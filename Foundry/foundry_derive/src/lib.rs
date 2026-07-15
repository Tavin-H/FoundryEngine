extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(SerializeComponent)]
pub fn serialize_component_macro(input: TokenStream) -> TokenStream {
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
            quote!((#name_str, self.#name.serialize(serializer)))
        });

        let struct_name = &input.ident;
        let name_str: &str = &struct_name.to_string();

        return quote!(
        impl serializer::Serialize for #struct_name {
            fn serialize(&self, serializer: &mut impl serializer::Serializer) -> serializer::SerializerNode {
                let node = serializer::SerializerNode::Component{
                    f_name: #name_str,
                    foundry_type_id: 1,
                    local_file_id: uuid::Uuid::new_v4(),
                    data_fields: vec![#(#field_vals),*],
                };
                serializer.serialize_component(&node);
                node
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

#[proc_macro_derive(Component)]
pub fn derive_component_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(ref data) = input.data {
        //Filter out only named field structs
        let Fields::Named(_) = data.fields else {
            return TokenStream::from(
                syn::Error::new(
                    input.ident.span(),
                    "Only structs with named fields can derive `Component`",
                )
                .to_compile_error(),
            );
        };

        let struct_name = &input.ident;
        return quote!(
            impl Component for #struct_name {}
        )
        .into();
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Enums, and Unions cannot derive `Component`",
        )
        .to_compile_error(),
    )
}
