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
            let name = &field.ident.as_ref();
            let name_str: &str = &name.unwrap().to_string();
            quote!(self.#name.serialize(#name_str, serializer))
        });

        let struct_name = &input.ident;
        let name_str: &str = &struct_name.to_string();

        return quote!(
        impl Serialize for #struct_name {
            fn serialize(&self, _name: &'static str, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::Struct{
                    f_name: #name_str,
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
