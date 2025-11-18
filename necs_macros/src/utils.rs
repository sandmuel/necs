use syn::{GenericParam, Generics, Lifetime, Visibility, parse_quote};

/// Insert a lifetime parameter with the given name.
pub fn with_lifetime(mut generics: Generics, lifetime_name: &str) -> Generics {
    let lifetime = Lifetime::new(
        &format!("'{}", lifetime_name),
        proc_macro2::Span::call_site(),
    );
    generics.params.insert(0, parse_quote!(#lifetime));

    // Ensure the angled brackets.
    if generics.lt_token.is_none() && generics.gt_token.is_none() {
        generics.lt_token = Some(Default::default());
        generics.gt_token = Some(Default::default());
    }

    generics
}

/// Strip all bounds from the given [Generics].
pub fn only_generic_idents(generics: &Generics) -> Generics {
    let mut new_generics = Generics::default();

    for param in generics.params.iter() {
        match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                new_generics.params.push(parse_quote!(#ident));
            }
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                new_generics.params.push(parse_quote!(#lifetime));
            }
            GenericParam::Const(param) => {
                let ident = &param.ident;
                new_generics.params.push(parse_quote!(const #ident: _));
            }
        }
    }

    new_generics
}

/// Increase the visibility of an item by one level.
pub fn one_up_vis(vis: Visibility) -> Visibility {
    match vis {
        // Inherited generally means private, so we replace it with pub(super).
        Visibility::Inherited => parse_quote!(pub(super)),
        Visibility::Restricted(mut vis) => {
            vis.path.segments.push(parse_quote!(super));
            Visibility::Restricted(vis)
        }
        Visibility::Public(_) => parse_quote!(pub),
    }
}
