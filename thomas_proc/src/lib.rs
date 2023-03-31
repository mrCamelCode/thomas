use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn get_behaviour_name(input: TokenStream) -> TokenStream {
    
    TokenStream::from(input)

    // TokenStream::from(quote!(
        
    // ))
}
