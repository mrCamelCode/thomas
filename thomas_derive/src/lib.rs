use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_component_macro(&ast)
}

fn impl_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let gen = quote! {
        impl Component for #struct_name {
            fn name() -> &'static str {
                stringify!(#struct_name)
            }

            fn component_name(&self) -> &'static str {
                stringify!(#struct_name)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn is_component_type(comp: &Box<dyn Component>) -> bool where Self: Sized{
                comp.component_name() == Self::name()
            }

            fn coerce(comp: &Box<dyn Component>) -> Option<&Self> where Self: Sized {
                comp.as_any().downcast_ref::<Self>()
            }

            fn coerce_mut(comp: &mut Box<dyn Component>) -> Option<&mut Self> where Self: Sized {
                comp.as_any_mut().downcast_mut::<Self>()
            }
        }
    };

    gen.into()
}
