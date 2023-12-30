use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput};

/// Merges the variants of two enums.
///
/// Adapted from DAO DAO:
/// https://github.com/DA0-DA0/dao-contracts/blob/74bd3881fdd86829e5e8b132b9952dd64f2d0737/packages/dao-macros/src/lib.rs#L9
fn merge_variants(metadata: TokenStream, left: TokenStream, right: TokenStream) -> TokenStream {
    use syn::Data::Enum;

    // parse metadata
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    // parse the left enum
    let mut left: DeriveInput = parse_macro_input!(left);
    let Enum(DataEnum { variants, .. }) = &mut left.data else {
        return syn::Error::new(left.ident.span(), "only enums can accept variants")
            .to_compile_error()
            .into();
    };

    // parse the right enum
    let right: DeriveInput = parse_macro_input!(right);
    let Enum(DataEnum {
        variants: to_add, ..
    }) = right.data
    else {
        return syn::Error::new(left.ident.span(), "only enums can provide variants")
            .to_compile_error()
            .into();
    };

    // insert variants from the right to the left
    variants.extend(to_add);

    quote! { #left }.into()
}

/// Append `incoming proxy`-related execute message variants to an enum.
///
/// For example, apply the `cw_incoming_proxy_execute` macro to the following enum:
///
/// ```rust
/// use cosmwasm_schema::cw_serde;
/// use cw_incoming_proxy::cw_incoming_proxy_execute;
///
/// #[cw_incoming_proxy_execute]
/// #[cw_serde]
/// enum ExecuteMsg {
///     Foo {},
///     Bar {},
/// }
/// ```
///
/// Is equivalent to:
///
/// ```rust
/// use cosmwasm_schema::cw_serde;
/// use cosmwasm_std::IbcPacket;
/// use ics721::ibc::NonFungibleTokenPacketData;
///
/// #[cw_serde]
/// enum ExecuteMsg {
///     Ics721ReceivePacketMsg {
///         packet: IbcPacket,
///         data: NonFungibleTokenPacketData,
///     },
///     Foo {},
///     Bar {},
/// }
/// ```
///
/// Note: `#[cw_ics721_config_execute]` must be applied _before_ `#[cw_serde]`.
/// Adapted from CW++:
/// https://github.com/larry0x/cw-plus-plus/tree/main/packages/ownable
#[proc_macro_attribute]
pub fn cw_incoming_proxy_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                Ics721ReceivePacketMsg {
                    packet: cosmwasm_std::IbcPacket,
                    data: ics721_types::ibc_types::NonFungibleTokenPacketData,
                },
            }
        }
        .into(),
    )
}

/// Append `incoming proxy`-related query message variant(s) to an enum.
///
/// For example, apply the `cw_incoming_proxy_query` macro to the following enum:
///
/// ```rust
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cw_ics721_config::cw_ics721_config_query;
///
/// #[cw_incoming_proxy_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {
///     #[returns(FooResponse)]
///     Foo {},
///     #[returns(BarResponse)]
///     Bar {},
/// }
/// ```
///
/// Is equivalent to:
///
/// ```rust
/// use cosmwasm_schema::cw_serde;
///
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {
///     #[returns(Option<Addr>)]
///     GetOrigin {},
///     #[returns(Vec<String>)]
///     GetSourceChannels {},
///     #[returns(FooResponse)]
///     Foo {},
///     #[returns(BarResponse)]
///     Bar {},
/// }
/// ```
///
/// Note: `#[cw_ics721_config_query]` must be applied _before_ `#[cw_serde]`.
/// Adapted from CW++:
/// https://github.com/larry0x/cw-plus-plus/tree/main/packages/ownable
#[proc_macro_attribute]
pub fn cw_incoming_proxy_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                #[returns(cosmwasm_std::Addr)]
                GetOrigin {},
                #[returns(Vec<String>)]
                GetSourceChannels {
                    start_after: Option<String>,
                    limit: Option<u32>,
                },
            }
        }
        .into(),
    )
}
