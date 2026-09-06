use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};
// Doc: https://doc.rust-lang.org/reference/procedural-macros.html

#[proc_macro_derive(Info)]
pub fn derive_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let field_info = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let field_name = field.ident.as_ref().expect("named field has an identifier");
                    let field_type = &field.ty;

                    quote! {
                        println!(
                            "{}: {}",
                            stringify!(#field_name),
                            stringify!(#field_type)
                        );
                    }
                })
                .collect::<Vec<_>>(),
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    type_name,
                    "Info cannot be derived for unnamed structs",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(type_name, "Info cannot be derived for unit types")
                    .to_compile_error()
                    .into();
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return syn::Error::new_spanned(type_name, "Info cannot be derived for non-structs")
                .to_compile_error()
                .into();
        }
    };

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics Info for #type_name #type_generics #where_clause {
            fn info(&self) {
                #(#field_info)*
            }
        }
    }
    .into()
}
