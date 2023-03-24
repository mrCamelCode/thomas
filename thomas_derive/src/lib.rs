use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Behaviour)]
pub fn behaviour_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_behaviour_macro(&ast)
}

fn impl_behaviour_macro(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let gen = quote! {
        impl Behaviour for #struct_name {
            fn name(&self) -> &'static str {
                stringify!(#struct_name)
            }
        }
    };

    gen.into()
}