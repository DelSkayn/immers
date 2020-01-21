extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Ident, Index, Member, Visibility};

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

fn to_camel_case(ident: &Ident) -> Ident {
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
    println!("{}", res);
    format_ident!("{}", res)
}

fn to_snake_case(ident: &Ident) -> Ident {
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
    println!("{}", res);
    format_ident!("{}", res)
}

fn to_tuple_index(count: u32) -> Member {
    Member::Unnamed(Index {
        index: count,
        span: Span::call_site(),
    })
}

fn impl_struct(name: Ident, vis: Visibility, data: DataStruct) -> TokenStream {
    let mut field_names = Vec::new();
    let mut member_names = Vec::new();
    let mut field_types = Vec::new();
    let mut unnamed_count: u32 = 0;
    for field in data.fields.iter() {
        if let Some(ref x) = field.ident {
            field_names.push(to_camel_case(x));
            member_names.push(Member::Named(x.clone()));
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

    let stream = quote! {
        #[derive(Clone)]
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
    print!("{}", stream);
    proc_macro::TokenStream::from(stream)
}

#[proc_macro_derive(Patchable)]
pub fn derive_patchable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match input.data {
        Data::Struct(x) => impl_struct(input.ident, input.vis, x),
        _ => todo!(),
    }
}
