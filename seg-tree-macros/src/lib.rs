use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parser},
    parse_macro_input, DeriveInput, ItemStruct,
};

/// Derive Macro for `PersistentNode`, use with persistent_node macro
#[proc_macro_derive(PersistentNode)]
pub fn test_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let output = quote! {
    impl #impl_generics PersistentNode for #ident #ty_generics #where_clause {
        fn left_child(&self)->usize {
            self._left_child
        }
        fn right_child(&self)->usize {
            self._right_child
        }
        fn set_children(&mut self, left: usize, right: usize){
            self._left_child = left;
            self._right_child = right;
        }
    }
    };

    output.into()
}

/// Attribute Macro for `PersistentNode`, use with derivo macro [PersistentNode]
#[proc_macro_attribute]
pub fn persistent_node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as ItemStruct);
    let _ = parse_macro_input!(attr as parse::Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        let items = ["left_child", "right_child"];
        for name in items {
            let _name = format_ident!("_{}", &name);
            fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! { #_name : usize })
                    .expect(format!("{}", &name).as_ref()),
            );
        }
    }

    return quote! {
        #item_struct
    }
    .into();
}
