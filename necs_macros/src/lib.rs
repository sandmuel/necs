mod utils;
use utils::{only_generic_idents, with_lifetime};

use crate::utils::one_up_vis;
use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Generics, Lifetime, PathArguments,
    Result, Type, TypeReference, TypeTuple, Visibility, parse_quote,
};

fn type_uses_world(ty: &Type) -> bool {
    match ty {
        Type::Reference(TypeReference {
            lifetime: Some(lifetime),
            ..
        }) => lifetime.ident == "world",

        // Check inside generic types like Option<&'world T>
        Type::Path(type_path) => type_path
            .path
            .segments
            .iter()
            .any(|seg| match &seg.arguments {
                PathArguments::AngleBracketed(ab) => ab.args.iter().any(|arg| match arg {
                    GenericArgument::Type(t) => type_uses_world(t),
                    GenericArgument::Lifetime(Lifetime { ident, .. }) => ident == "world",
                    _ => false,
                }),
                _ => false,
            }),

        // You can extend to arrays, tuples, etc., if needed
        _ => false,
    }
}

// Insert 'world if the struct uses it.
fn maybe_insert_generic(
    err_input: &DeriveInput,
    fields: Fields,
    generics: &mut Generics,
) -> Result<(bool, Fields)> {
    // Only inject the node lifetime if there is at least one field.
    let (empty, fields) = match fields {
        Fields::Named(fields) => (fields.named.is_empty(), Fields::Named(fields)),
        Fields::Unit => (true, Fields::Unit),
        _ => {
            return Err(syn::Error::new_spanned(
                err_input,
                "struct fields must be named",
            ));
        }
    };

    if !empty {
        let uses_world = fields.iter().any(|field| type_uses_world(&field.ty));

        if uses_world {
            generics.params.insert(0, syn::parse_quote!('world));

            // Ensure the angled brackets.
            if generics.lt_token.is_none() {
                generics.lt_token = Some(Default::default());
                generics.gt_token = Some(Default::default());
            }
        }
    }

    Ok((empty, fields))
}

struct GeneratedNodeBuilder {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: proc_macro2::Ident,
    generics: Generics,
    fields: Fields,
    node_ref: proc_macro2::Ident,
}

impl Parse for GeneratedNodeBuilder {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let err_input = input.clone();

        let attrs = input.attrs;
        let vis = one_up_vis(input.vis);
        let ident = format_ident!("{}Builder", input.ident);
        let mut generics = input.generics;
        let node_ref = input.ident;

        let fields = match input.data {
            Data::Struct(data) => maybe_insert_generic(&err_input, data.fields, &mut generics),
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "only structs are supported",
                ));
            }
        }?
        .1;

        Ok(Self {
            attrs,
            vis,
            ident,
            generics,
            fields,
            node_ref,
        })
    }
}

impl ToTokens for GeneratedNodeBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let generics = &self.generics;
        let generic_idents = only_generic_idents(generics);
        let static_and_generic_idents = with_lifetime(generic_idents.clone(), "static");
        let node_ref = &self.node_ref;

        // Generate struct fields.
        let struct_fields = match &self.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_ty = &field.ty;
                    let field_vis = one_up_vis(field.vis.clone());
                    // Filter out #[ext] attributes.
                    let attrs = field
                        .attrs
                        .iter()
                        .filter(|attr| !attr.path().is_ident("ext"));
                    quote! {
                        #(#attrs)*
                        #field_vis #field_name: #field_ty
                    }
                });
                quote! {
                    { #(#fields,)* }
                }
            }
            Fields::Unit => quote! {;},
            _ => unreachable!("struct fields should not be unnamed"),
        };

        // Generate field assignments for __move_to_storage().
        let field_assignments = match &self.fields {
            Fields::Named(fields) => {
                let assignments = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let has_ext = field.attrs.iter().any(|attr| attr.path().is_ident("ext"));

                    if has_ext {
                        quote! {
                            storage.components.insert(node_id.instance, self.#field_name);
                        }
                    } else {
                        quote! {}
                    }
                });

                let tuple_fields = fields.named.iter().filter_map(|field| {
                    let field_name = &field.ident;
                    let has_ext = field.attrs.iter().any(|attr| attr.path().is_ident("ext"));

                    if has_ext {
                        None
                    } else {
                        Some(quote! { self.#field_name })
                    }
                });

                quote! {
                    let node_id = storage.nodes.spawn::<Self::AsNodeRef>((#(#tuple_fields,)*));
                    #(#assignments)*
                    node_id
                }
            }
            Fields::Unit => quote! {
                storage.nodes.spawn::<Self::AsNodeRef>(())
            },
            _ => unreachable!("struct fields should not be unnamed"),
        };

        // Add the <'static> lifetime annotation if the struct has at least one field.
        let as_node_ref = match &self.fields {
            Fields::Named(fields) if !fields.named.is_empty() => {
                quote!(#node_ref #static_and_generic_idents)
            }
            _ => quote!(#node_ref),
        };

        quote! {
            #(#attrs)*
            #vis struct #ident #generics #struct_fields

            #[doc(hidden)]
            impl #generics ::necs::NodeBuilder for #ident #generic_idents {
                type AsNodeRef = #as_node_ref;

                fn __move_to_storage(self, storage: &mut ::necs::storage::Storage) -> ::necs::NodeId {
                    #field_assignments
                }
            }
        }
        .to_tokens(tokens);
    }
}

struct FieldInfo {
    attrs: Vec<Attribute>,
    is_ext: bool,
    vis: Visibility,
    ident: syn::Ident,
    ty: Type,
}

struct GeneratedNodeRef {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: proc_macro2::Ident,
    generics: Generics,
    fields: Vec<FieldInfo>,
    recipe_tuple: TypeTuple,
}

impl Parse for GeneratedNodeRef {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let err_input = input.clone();

        let attrs = input.attrs;
        let vis = one_up_vis(input.vis);
        let ident = input.ident;
        let generics = input.generics;

        let mut data = match input.data {
            Data::Struct(data) => data,
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "only structs are supported",
                ));
            }
        };

        let mut field_infos = Vec::new();
        let mut tuple_types = Vec::new();

        match &mut data.fields {
            Fields::Named(named_fields) => {
                for original_field in &mut named_fields.named {
                    let mut field = original_field.clone();
                    let is_ext = field.attrs.iter().any(|attr| attr.path().is_ident("ext"));
                    let mut attrs = field.attrs;
                    attrs.retain(|attr| !attr.path().is_ident("ext"));

                    if !is_ext {
                        tuple_types.push(field.ty.clone());
                    };
                    let inner = &field.ty;
                    field.ty = parse_quote!(&'world mut #inner);
                    original_field.ty = field.ty.clone();

                    // Fine because we know fields are named.
                    let ident = field.ident.unwrap();
                    let vis = field.vis;
                    let ty = field.ty;
                    field_infos.push(FieldInfo {
                        attrs,
                        is_ext,
                        vis,
                        ident,
                        ty,
                    });
                }
            }
            Fields::Unit => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "struct fields must be named",
                ));
            }
        }

        let recipe_ty: Type = parse_quote!((#(#tuple_types),*));
        let recipe_tuple = match recipe_ty {
            Type::Tuple(tup) => tup,
            _ => return Err(syn::Error::new_spanned(recipe_ty, "expected tuple type")),
        };

        Ok(Self {
            attrs,
            vis,
            ident,
            generics,
            fields: field_infos,
            recipe_tuple,
        })
    }
}

impl ToTokens for GeneratedNodeRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            attrs,
            vis,
            ident,
            generics,
            fields,
            recipe_tuple,
        } = self;

        let borrowed_def = if fields.is_empty() {
            quote! {}
        } else {
            quote! {
                #[doc(hidden)]
                _borrowed: ::necs::BorrowDropper<'world>,
            }
        };

        let borrowed = if fields.is_empty() {
            quote! {}
        } else {
            quote! { _borrowed: borrowed, }
        };

        let struct_fields = fields.iter().map(|field| {
            let FieldInfo {
                attrs,
                vis,
                ident,
                ty,
                ..
            } = field;
            let field_vis = one_up_vis(vis.clone());
            quote! {
                #(#attrs)*
                #field_vis #ident: #ty,
            }
        });

        let mut field_extractions = Vec::new();
        let mut component_registrations = Vec::new();
        let mut tuple_idx = 0;
        let generic_idents = only_generic_idents(generics);
        let mut world_and_generics = generics.clone();
        let mut world_and_generic_idents = generic_idents.clone();
        let mut static_and_generic_idents = generic_idents.clone();
        if !self.fields.is_empty() {
            world_and_generics = with_lifetime(world_and_generics, "world");
            world_and_generic_idents = with_lifetime(world_and_generic_idents, "world");
            static_and_generic_idents = with_lifetime(static_and_generic_idents, "static");
        }

        for field in fields {
            let name = &field.ident;

            if field.is_ext {
                // For #ext fields, access them using the node's instance id.
                if let Type::Reference(type_ref) = &field.ty {
                    let inner_type = &type_ref.elem;
                    field_extractions.push(quote! {
                        let #name = storage.components.get_element_unchecked(&::necs::ComponentId::<#inner_type>::new(id.node_type, id.instance));
                    });
                }

                // Get the original type. (without the reference)
                if let Type::Reference(type_ref) = &field.ty {
                    let inner_type = &type_ref.elem;
                    component_registrations.push(quote! {
                        storage.components.register::<#inner_type>();
                    });
                }
            } else {
                let idx = syn::Index::from(tuple_idx);
                field_extractions.push(quote! {
                    let #name = &mut recipe_tuple.#idx;
                });
                tuple_idx += 1;
            }
        }

        let field_names = fields.iter().map(|f| &f.ident);

        // Generate match arms for Node::get implementation
        let get_match_arms = fields.iter().map(|field| {
            let name = &field.ident;
            let name_str = name.to_string();

            quote! {
                #name_str => self.#name,
            }
        });

        quote! {
            #(#attrs)*
            #vis struct #ident #world_and_generics {
                #borrowed_def
                #(#struct_fields)*
            }

            #[doc(hidden)]
            impl #world_and_generics ::necs::NodeTrait for #ident #world_and_generic_idents {
                fn get(&mut self, field_name: &str) -> &mut dyn ::necs::Field {
                    match field_name {
                        #(#get_match_arms)*
                        _ => panic!("field {} does not exist on {}", field_name, ::std::any::type_name::<Self>()),
                    }
                }
            }

            #[doc(hidden)]
            impl #generics ::necs::NodeRef for #ident #static_and_generic_idents {
                type Instance<'world> = #ident #world_and_generic_idents;
                type RecipeTuple = #recipe_tuple;

                unsafe fn __build_from_storage<'world>(recipe_tuple: &'world mut Self::RecipeTuple, borrowed: ::necs::BorrowDropper<'world>, storage: &'world ::necs::storage::Storage, id: ::necs::NodeId) -> #ident #world_and_generic_idents {
                    #(#field_extractions)*
                    #ident {
                        #borrowed
                        #(#field_names,)*
                    }
                }

                fn __register_node(storage: &mut ::necs::storage::Storage) {
                    // Register the node itself
                    storage.nodes.register::<Self>();

                    // Register every #[ext] field with component storage
                    #(#component_registrations)*
                }
            }
        }.to_tokens(tokens);
    }
}

/// This macro generates various types and traits used by necs to manage nodes.
///
/// # Example
/// ```
/// # use necs::node;
/// # struct Transform;
/// # fn main() {
///
/// #[node]
/// pub struct MyNode {
///     // This field has no attributes and is stored together with the rest of
///     // MyNode's data.
///     foo: u32,
///     bar: i32,
///     // This field is external; it is stored in memory alongside other fields of
///     // the same type, rather than with the rest of MyNode, useful for fields
///     // which are usually accessed separately.
///     #[ext]
///     transform: Transform,
/// }
/// # }
/// ```
///
/// This will generate additional builder and reference code associated with
/// `MyNode` to enable advanced functionality.
#[proc_macro_attribute]
pub fn node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let node_builder = item.clone();
    let node_ref = item;
    let node_builder = syn::parse_macro_input!(node_builder as GeneratedNodeBuilder);
    let node_ref = syn::parse_macro_input!(node_ref as GeneratedNodeRef);
    let mod_name = format_ident!("__necs_macro_{}", node_ref.ident.to_string().to_lowercase());
    quote! {
        mod #mod_name {
            use super::*;
            #node_builder #node_ref
        }
        pub use #mod_name::*;
    }
    .into()
}
