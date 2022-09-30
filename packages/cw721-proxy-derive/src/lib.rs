use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the nesecary fields to a enum such that it implements the
/// cw721-proxy-receiver interface. You must have the cw721 Cargo
/// package avaliable as a dependency to use this macro.
///
/// For example:
///
/// ```
/// use cw721_proxy_derive::cw721_proxy;
///
/// #[cw721_proxy]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum ExecuteMsg {
///     ReceiveProxyNft {
///         eyeball: String,
///         msg: cw721::Cw721ReceiveMsg,
///     }
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cw721_proxy_derive::cw721_proxy;
///
/// #[derive(Clone)]
/// #[cw721_proxy]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn cw721_proxy(metadata: TokenStream, input: TokenStream) -> TokenStream {
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
                ReceiveProxyNft {
                    eyeball: String,
                    msg: ::cw721::Cw721ReceiveMsg,
                }
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
