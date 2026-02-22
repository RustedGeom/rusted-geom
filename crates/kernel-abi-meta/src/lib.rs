use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn rgm_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn rgm_ffi_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
