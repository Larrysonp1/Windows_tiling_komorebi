#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![no_implicit_prelude]

use ::std::clone::Clone;
use ::std::convert::From;
use ::std::convert::Into;
use ::std::iter::Extend;
use ::std::iter::Iterator;
use ::std::matches;
use ::std::string::ToString;
use ::std::unreachable;

use ::quote::quote;
use ::std::option::Option::Some;
use ::syn::parse_macro_input;
use ::syn::Data;
use ::syn::DataEnum;
use ::syn::DeriveInput;
use ::syn::Fields;
use ::syn::FieldsNamed;
use ::syn::FieldsUnnamed;

#[proc_macro_derive(AhkFunction)]
pub fn ahk_function(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let idents = named.iter().map(|f| &f.ident);
                let arguments = quote! {#(#idents), *}.to_string();

                let idents = named.iter().map(|f| &f.ident);
                let called_arguments = quote! {#(%#idents%) *}
                    .to_string()
                    .replace(" %", "%")
                    .replace("% ", "%")
                    .replace("%%", "% %");

                quote! {
                    impl AhkFunction for #name {
                        fn generate_ahk_function() -> String {
                            ::std::format!(r#"
{}({}) {{
    Run, komorebic.exe {} {}, , Hide
}}"#, 
                                ::std::stringify!(#name),
                                #arguments,
                                stringify!(#name).to_kebab_case(),
                                #called_arguments
                            )
                       }
                    }
                }
            }
            _ => unreachable!("only to be used on structs with named fields"),
        },
        _ => unreachable!("only to be used on structs"),
    }
    .into()
}

#[proc_macro_derive(AhkLibrary)]
pub fn ahk_library(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let enums = variants.iter().filter(|&v| {
                matches!(v.fields, Fields::Unit) || matches!(v.fields, Fields::Unnamed(..))
            });

            let mut stream = ::proc_macro2::TokenStream::new();

            for variant in enums.clone() {
                match &variant.fields {
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        for field in unnamed {
                            stream.extend(quote! {
                                v.push(#field::generate_ahk_function());
                            });
                        }
                    }
                    Fields::Unit => {
                        let name = &variant.ident;
                        stream.extend(quote! {
                            v.push(::std::format!(r#"
{}() {{
    Run, komorebic.exe {}, , Hide
}}"#, 
                                ::std::stringify!(#name),
                                ::std::stringify!(#name).to_kebab_case()
                            ));
                        });
                    }
                    Fields::Named(_) => {
                        unreachable!("only to be used with unnamed and unit fields");
                    }
                }
            }

            quote! {
                impl #name {
                    fn generate_ahk_library() -> String {
                        let mut v: Vec<String> = vec![String::from("; Generated by komorebic.exe")];

                        #stream

                        v.join("\n")
                    }
               }
            }
        }
        _ => unreachable!("only to be used on enums"),
    }
    .into()
}
