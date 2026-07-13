extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Serialize)]
pub fn serialize_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(ref data) = input.data {
        //Filter out only named field structs
        let Fields::Named(ref fields) = data.fields else {
            return TokenStream::from(
                syn::Error::new(
                    input.ident.span(),
                    "Only structs with named fields can derive `Serialize`",
                )
                .to_compile_error(),
            );
        };

        let field_vals = fields.named.iter().map(|field| {
            // grab the name of the field
            let name = &field.ident;
            quote!(self.#name.serialize(serializer))
        });

        let name = input.ident;

        return quote!(
        impl Serialize for #name {
            fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::Struct{
                    data: vec![#(#field_vals),*]
                }
            }
        })
        .into();
    }

    // Catchall if we don't match on the structure we don't want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Enums, and Unions cannot derive `Serialize`",
        )
        .to_compile_error(),
    )
}
