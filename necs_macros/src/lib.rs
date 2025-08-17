use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Data, DeriveInput, Fields, Generics, TypeTuple, Visibility, parse_quote};

struct GeneratedNodeBuilder {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: proc_macro2::Ident,
    generics: Generics,
    fields: Fields,
    node_ref: proc_macro2::Ident,
}

impl Parse for GeneratedNodeBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = input.parse()?;
        let err_input = input.clone();

        let attrs = input.attrs;
        let vis = input.vis;
        let ident = format_ident!("{}Builder", input.ident);
        let generics = input.generics;
        let node_ref = input.ident;

        let fields = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => Fields::Named(fields),
                Fields::Unit => Fields::Unit,
                _ => {
                    return Err(syn::Error::new_spanned(
                        err_input,
                        "struct fields must be named",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "only structs are supported",
                ));
            }
        };

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
        let node_ref = &self.node_ref;

        // Generate struct fields.
        let struct_fields = match &self.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_ty = &field.ty;
                    let field_vis = &field.vis;
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
                            let #field_name = storage.components.spawn(self.#field_name);
                        }
                    } else {
                        quote! {
                            let #field_name = self.#field_name;
                        }
                    }
                });

                let tuple_fields = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name }
                });

                quote! {
                    #(#assignments)*
                    storage.nodes.spawn((#(#tuple_fields,)*))
                }
            }
            Fields::Unit => quote! {
                storage.nodes.spawn(())
            },
            _ => unreachable!("struct fields should not be unnamed"),
        };

        // Add the <'static> lifetime annotation if the struct has at least one field.
        let as_node_ref = match &self.fields {
            Fields::Named(fields) if !fields.named.is_empty() => quote!(#node_ref<'static>),
            _ => quote!(#node_ref),
        };

        quote! {
            #(#attrs)*
            #vis struct #ident #generics #struct_fields

            #[doc(hidden)]
            impl ::necs::NodeBuilder for #ident #generics {
                type AsNodeRef = #as_node_ref;

                unsafe fn __move_to_storage(self, storage: &mut ::necs::storage::Storage) -> ::necs::NodeId {
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
    ty: syn::Type,
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
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = input.parse()?;
        let err_input = input.clone();

        let attrs = input.attrs;
        let vis = input.vis;
        let ident = input.ident;
        let mut generics = input.generics;

        let data = match input.data {
            Data::Struct(data) => data,
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "only structs are supported",
                ));
            }
        };

        // Only inject the node lifetime if there is at least one field.
        let empty = match &data.fields {
            Fields::Named(named) => named.named.is_empty(),
            Fields::Unit => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    err_input,
                    "struct fields must be named",
                ));
            }
        };

        if !empty {
            generics.params.insert(0, syn::parse_quote!('node));

            // Ensure the angled brackets.
            if generics.lt_token.is_none() {
                generics.lt_token = Some(Default::default());
                generics.gt_token = Some(Default::default());
            }
        }

        let mut field_infos = Vec::new();
        let mut tuple_types = Vec::new();

        match data.fields {
            Fields::Named(named_fields) => {
                for field in named_fields.named {
                    let is_ext = field.attrs.iter().any(|attr| attr.path().is_ident("ext"));
                    let mut attrs = field.attrs;
                    attrs.retain(|attr| !attr.path().is_ident("ext"));

                    let ty = if is_ext {
                        let inner = &field.ty;
                        tuple_types.push(parse_quote!(::necs::ComponentId<#inner>));
                        parse_quote!(&'node mut #inner)
                    } else {
                        let inner = &field.ty;
                        tuple_types.push(field.ty.clone());
                        parse_quote!(&'node mut #inner)
                    };

                    // Safe because fields are named.
                    let ident = field.ident.unwrap();
                    let vis = field.vis;
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

        let recipe_ty: syn::Type = parse_quote!((#(#tuple_types),*));
        let recipe_tuple = match recipe_ty {
            syn::Type::Tuple(tup) => tup,
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

        // Emit struct fields
        let struct_fields = fields.iter().map(|field| {
            let FieldInfo {
                attrs,
                vis,
                ident,
                ty,
                ..
            } = field;
            quote! {
                #(#attrs)*
                #vis #ident: #ty,
            }
        });

        // Field extraction logic
        let mut field_extractions = Vec::new();
        let mut component_registrations = Vec::new();
        let mut idx = 0;

        for field in fields {
            let tuple_idx = syn::Index::from(idx);
            let name = &field.ident;

            if field.is_ext {
                field_extractions.push(quote! {
                    let #name = &mut storage.components[&recipe_tuple.#tuple_idx];
                });

                // Get the original type (without the reference)
                if let syn::Type::Reference(type_ref) = &field.ty {
                    let inner_type = &type_ref.elem;
                    component_registrations.push(quote! {
                        storage.components.register::<#inner_type>();
                    });
                }
            } else {
                field_extractions.push(quote! {
                    let #name = &mut recipe_tuple.#tuple_idx;
                });
            }
            idx += 1;
        }

        let field_names = fields.iter().map(|f| &f.ident);

        // Generate match arms for Node::get implementation
        let get_match_arms = fields.iter().map(|field| {
            let name = &field.ident;
            let mut ty = &field.ty;
            if let syn::Type::Reference(type_ref) = &field.ty {
                ty = &type_ref.elem;
            }
            let name_str = name.to_string();

            quote! {
                #name_str => unsafe {
                    ::std::mem::transmute::<&mut #ty, &'static mut #ty>(self.#name)
                },
            }
        });

        quote! {
            #(#attrs)*
            #vis struct #ident #generics {
                #(#struct_fields)*
            }

            #[doc(hidden)]
            impl #generics ::necs::NodeTrait for #ident #generics {
                fn get(&mut self, field_name: &str) -> &mut dyn ::necs::Field {
                    match field_name {
                        #(#get_match_arms)*
                        _ => panic!("field {} does not exist on {}", field_name, ::std::any::type_name::<Self>()),
                    }
                }
            }

            #[doc(hidden)]
            impl #generics ::necs::NodeRef for #ident #generics {
                type RecipeTuple = #recipe_tuple;

                unsafe fn __build_from_storage(storage: &mut ::necs::storage::Storage, id: ::necs::NodeId) -> Self {
                    let storage = unsafe {
                        ::std::mem::transmute::<_, &'static mut ::necs::storage::Storage>(storage)
                    };
                    let recipe_tuple = unsafe {
                        storage
                            .nodes[id.node_type]
                            .downcast_mut_unchecked::<::necs::SubStorage<Self::RecipeTuple>>()
                            .get_mut(id.instance).unwrap()
                    };
                    #(#field_extractions)*
                    Self {
                        #(#field_names,)*
                    }
                }

                fn __register_node(storage: &mut ::necs::storage::Storage) {
                    // Register the node itself
                    storage.nodes.register::<Self::RecipeTuple>();

                    // Register every #[ext] field with component storage
                    #(#component_registrations)*
                }
            }
        }
            .to_tokens(tokens);
    }
}

/// This macro generates various types and traits used by necs to manage nodes.
///
/// # Example
/// ```ignore
/// #[node]
/// pub struct MyNode {
///     // This field has no attributes, and is stored together with the rest of
///     // MyNode's data.
///     foo: u32,
///     // This field is external, it is stored in memory alongside other fields of
///     // the same type, rather than with the rest of MyNode, useful for fields
///     // which are usually accessed individually.
///     #[ext]
///     transform: Transform,
/// }
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
    quote!(#node_builder #node_ref).into()
}
