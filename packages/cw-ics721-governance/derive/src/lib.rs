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
    let Enum(DataEnum {
        variants,
        ..
    }) = &mut left.data else {
        return syn::Error::new(left.ident.span(), "only enums can accept variants")
            .to_compile_error()
            .into();
    };

    // parse the right enum
    let right: DeriveInput = parse_macro_input!(right);
    let Enum(DataEnum {
        variants: to_add,
        ..
    }) = right.data else {
        return syn::Error::new(left.ident.span(), "only enums can provide variants")
            .to_compile_error()
            .into();
    };

    // insert variants from the right to the left
    variants.extend(to_add.into_iter());

    quote! { #left }.into()
}

/// Append governance-related execute message variant(s) to an enum.
///
/// For example, apply the `cw_ics721_governance_execute` macro to the following enum:
///
/// ```rust
/// use cosmwasm_schema::cw_serde;
/// use cw_ics721_governance::cw_ics721_governance_exeucte;
///
/// #[cw_ics721_governance_execute]
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
/// use cw_ics721_governance::Action;
///
/// #[cw_serde]
/// enum ExecuteMsg {
///     Governance(Action),
///     ReceiveNft(cw721::Cw721ReceiveMsg),
///     Foo {},
///     Bar {},
/// }
/// ```
///
/// Note: `#[cw_ics721_governance_execute]` must be applied _before_ `#[cw_serde]`.
#[proc_macro_attribute]
pub fn cw_ics721_governance_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                /// Actions that can be taken to alter the proxy contract's governance like in `execute`'s entry point:
                /// ```rust
                /// use cosmwasm_std::{from_binary, Binary, DepsMut, Env, MessageInfo, Response, Storage};
                /// use cosmwasm_schema::cw_serde;
                /// use cosmwasm_std::entry_point;
                ///
                /// #[cw_serde]
                /// pub enum ExecuteMsg {
                ///     Governance(cw_ics721_governance::Action),
                ///     ReceiveNft(cw721::Cw721ReceiveMsg)
                /// }
                ///
                /// #[cfg_attr(not(feature = "library"), entry_point)]
                /// pub fn execute(
                ///     deps: DepsMut,
                ///     env: Env,
                ///     info: MessageInfo,
                ///     msg: ExecuteMsg,
                /// ) -> Result<Response, cw_ics721_governance::GovernanceError> {
                ///     match msg {
                ///         ExecuteMsg::Governance(action) => {
                ///             Ok(cw_ics721_governance::execute(deps, env, &info, action)?)
                ///         }
                ///         ExecuteMsg::ReceiveNft(msg) => {
                ///             Ok(cw_ics721_governance::execute_receive_nft(deps, info, msg)?)
                ///         }
                ///     }
                /// }
                /// ```
                Governance(::cw_ics721_governance::Action),
                /// Cw721ReceiveMsg to be forwared to ICS721 (origin).
                /// NOTE: this is NOT part of governance, since it is send by cw721 contract diretly to proxy
                ReceiveNft(cw721::Cw721ReceiveMsg),
            }
        }
        .into(),
    )
}

/// Append governance-related query message variant(s) to an enum.
///
/// For example, apply the `cw_ics721_governance_query` macro to the following enum:
///
/// ```rust
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cw_ics721_governance::cw_ics721_governance_query;
///
/// #[cw_ics721_governance_query]
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
/// use cw_ics721_governance::Governance;
///
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum ExecuteMsg {
///     #[returns(Governance)]
///     Governance {},
///     #[returns(FooResponse)]
///     Foo {},
///     #[returns(BarResponse)]
///     Bar {},
/// }
/// ```
///
/// Note: `#[cw_ics721_governance_query]` must be applied _before_ `#[cw_serde]`.
#[proc_macro_attribute]
pub fn cw_ics721_governance_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                /// Query the contract's governance information
                #[returns(::cw_ics721_governance::Governance)]
                Governance(),
            }
        }
        .into(),
    )
}
