use proc_macro::TokenStream;
use quote::quote;

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

            fn is_component_type(comp: &dyn Component) -> bool where Self: Sized{
                comp.component_name() == Self::name()
            }

            fn coerce(comp: &dyn Component) -> Option<&Self> where Self: Sized {
                comp.as_any().downcast_ref::<Self>()
            }

            fn coerce_mut(comp: &mut dyn Component) -> Option<&mut Self> where Self: Sized {
                comp.as_any_mut().downcast_mut::<Self>()
            }
        }
    };

    gen.into()
}

// #[derive(Parse)]
// struct Test {
//     query_result: syn::Ident,
//     comma: Punct,
//     component_type: syn::Ident,
// }

#[proc_macro]
pub fn get_component(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let query_result =
        syn::parse_str::<syn::Ident>(&iter.next().unwrap().span().source_text().unwrap()).unwrap();

    // Ignore comma
    iter.next();

    let component_type =
        syn::parse_str::<syn::Ident>(&iter.next().unwrap().span().source_text().unwrap()).unwrap();

    // TODO: This currently doesn't work because of Rust complaining about trying to use a temp value or some such.
    let gen = quote! {
        #component_type::coerce(
            &#query_result
                .components
                .iter()
                .find(|comp| comp.borrow().component_name() == #component_type::name())
                .expect("get_component: Provided component type is present in query results.")
                .borrow(),
        )
        .unwrap()

    };

    gen.into()
}
