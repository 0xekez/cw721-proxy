use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the nesecary fields to a enum such that it implements the
/// cw721 ReceiveNft interface. You must have the cw721 Cargo
/// package avaliable as a dependency to use this macro.
///
/// For example:
///
/// ```
/// use cw_ics721_outgoing_proxy_derive::cw721_receive_nft;
///
/// #[cw721_receive_nft]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum ExecuteMsg {
///     ReceiveNft(cw721::Cw721ReceiveMsg)
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cw721_outgoing_proxy_derive::cw721_receive_nft;
///
/// #[derive(Clone)]
/// #[cw721_receive_nft]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn cw721_receive_nft(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let receive: Variant = syn::parse2(quote! {
                ReceiveNft (::cw721::Cw721ReceiveMsg)
            })
            .unwrap();

            variants.push(receive);
        }
        _ => {
            return syn::Error::new(ast.ident.span(), "macro may only be derived on enums")
                .to_compile_error()
                .into()
        }
    };

    quote! {
    #ast
    }
    .into()
}
