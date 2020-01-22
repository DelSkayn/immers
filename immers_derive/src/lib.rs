extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};

fn get_name(mut count: u32) -> String {
    let letters = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    let mut res = String::new();
    while count >= 26 {
        res.push(letters[(count % 26) as usize]);
        count -= 26;
    }
    res.push(letters[count as usize]);
    return res;
}

fn to_camel_case(ident: &syn::Ident) -> syn::Ident {
    let text = format!("{}", ident);
    let mut res = String::new();
    res.reserve(text.len());
    let mut should_uppercase = true;
    for c in text.chars() {
        if c == '_' {
            should_uppercase = true;
            continue;
        }
        if should_uppercase {
            for u in c.to_uppercase() {
                res.push(u);
            }
            should_uppercase = false;
        } else {
            res.push(c);
        }
    }
    format_ident!("{}", res)
}

fn to_snake_case(ident: &syn::Ident) -> syn::Ident {
    let text = format!("{}", ident);
    let mut res = String::new();
    res.reserve(text.len());
    let mut first = true;
    for c in text.chars() {
        if c.is_uppercase() {
            if !first {
                res.push('_');
            }

            for l in c.to_lowercase() {
                res.push(l)
            }
        } else {
            res.push(c);
        }
        first = false;
    }
    format_ident!("{}", res)
}

fn to_tuple_index(count: u32) -> syn::Member {
    syn::Member::Unnamed(syn::Index {
        index: count,
        span: Span::call_site(),
    })
}

fn impl_struct(
    name: syn::Ident,
    vis: syn::Visibility,
    data: syn::DataStruct,
    derive: Vec<syn::Ident>,
) -> TokenStream {
    let mut field_names = Vec::new();
    let mut member_names = Vec::new();
    let mut field_types = Vec::new();
    let mut unnamed_count: u32 = 0;
    for field in data.fields.iter() {
        if let Some(ref x) = field.ident {
            field_names.push(to_camel_case(x));
            member_names.push(syn::Member::Named(x.clone()));
            field_types.push(field.ty.clone());
        } else {
            let ident = format_ident!("{}", get_name(unnamed_count));
            field_names.push(ident);
            member_names.push(to_tuple_index(unnamed_count));
            unnamed_count += 1;
            field_types.push(field.ty.clone());
        }
    }
    let patch_name = format_ident!("{}Patch", name);
    let patch_error_name = format_ident!("{}PatchError", name);
    let mod_name = format_ident!("__{}_mod", to_snake_case(&name));

    let derive_tag = quote! {
        #[derive(#(#derive,)*)]
    };

    let stream = quote! {
        #derive_tag
        #vis enum #patch_name {
            #(#field_names (<#field_types as Patchable>::Patch),)*
        }
        #vis enum #patch_error_name {
            #(#field_names (<#field_types as Patchable>::Error),)*
        }

        mod #mod_name{
            use std::fmt;
            use super::#patch_error_name;

            impl fmt::Display for #patch_error_name{
                fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result{
                    match *self{
                        #(#patch_error_name::#field_names(ref x) => {
                            write!(fmt, "within `#name.#field_names` => {}",x)
                        })*
                    }
                }
            }
        }

        impl Patchable for #name{
            type Patch = Vec<#patch_name>;
            type Error = #patch_error_name;

            fn produce(&self, other: &Self) -> Option<Self::Patch>{
                let mut res = Vec::new();
                #(if let Some(x) = self.#member_names.produce(&other.#member_names) {
                    res.push(#patch_name::#field_names(x));
                })*
                if res.len() == 0{
                    None
                }else{
                    Some(res)
                }
            }

            fn apply(&mut self, patch: Self::Patch) -> Result<(),Self::Error>{
                for p in patch {
                    match p{
                        #(#patch_name::#field_names(x) => {
                            self.#member_names.apply(x).map_err(#patch_error_name::#field_names)?;
                        })*
                    }
                }
                Ok(())
            }
        }
    };
    proc_macro::TokenStream::from(stream)
}

/// Specifically for working with attributes like #[shrinkwrap(..)], where
/// a name is combined with a list of attributes. Get the list of attributes
/// matching the tag.
fn tagged_attrs<'a, A: 'a>(tag: &str, attrs: A) -> Vec<syn::NestedMeta>
where
    A: Iterator<Item = &'a syn::Attribute>,
{
    use syn::{Meta, MetaList};

    let mut result = vec![];

    for attr in attrs {
        let meta = attr.parse_meta();

        if let Ok(Meta::List(MetaList { path, nested, .. })) = meta {
            if path.is_ident(tag) {
                result.extend(nested);
            }
        }
    }

    result
}

fn patchable_options(attrs: &[syn::Attribute]) -> Vec<syn::Ident> {
    use syn::{Meta, MetaList, NestedMeta};
    let meta = tagged_attrs("patchable", attrs.iter());

    let mut res = vec![format_ident!("Clone")];
    for m in meta.iter() {
        if let NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) = m {
            if !path.is_ident("derive") {
                continue;
            }
            for d in nested.iter() {
                if let NestedMeta::Meta(Meta::Path(x)) = d {
                    x.get_ident().map(|x| {
                        if x != "Clone" {
                            res.push(x.clone())
                        }
                    });
                }
                //TODO handle invalid attrs.
            }
        }
    }
    res
}

#[proc_macro_derive(Patchable, attributes(patchable))]
pub fn derive_patchable(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let derive = patchable_options(&input.attrs);
    match input.data {
        syn::Data::Struct(x) => impl_struct(input.ident, input.vis, x, derive),
        _ => todo!(),
    }
}
