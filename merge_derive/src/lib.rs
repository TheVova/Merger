extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam};

#[proc_macro_derive(MergeFrom)]
pub fn derive_merge_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut generics = input.generics;
    // Add a bound `T: MergeFrom` to every type parameter T.
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(MergeFrom));
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Generate a statement that merges each param.
    let merges = make_merge(&input.data);

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics MergeFrom for #name #ty_generics #where_clause {
            fn merge_from(&mut self,other: &Self) {
                #merges
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn make_merge(data: &Data) -> TokenStream {
    match &data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let merges = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote_spanned! { f.span() =>
                        MergeFrom::merge_from(&mut self.#field_name, &other.#field_name)
                    }
                });
                quote! { #(#merges;)* }
            }
            Fields::Unnamed(fields) => {
                let merges = fields.unnamed.iter().enumerate().map(|(s, f)| {
                    let idx = syn::Index::from(s);
                    quote_spanned! { f.span() =>
                        MergeFrom::merge_from(&mut self.#idx, &other.#idx)
                    }
                });
                quote! { #(#merges;)* }
            }
            Fields::Unit => quote! { () },
        },
        Data::Enum(_) => { unimplemented!() }
        Data::Union(_) => { unimplemented!() }
    }
}