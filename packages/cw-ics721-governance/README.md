# ICS721 Governance

Utility for controlling governance of [ICS721 proxy](https://github.com/arkprotocol/cw721-proxy) smart contracts.

## How to use

Initialize the governed proxy during instantiation using the `instantiate` method provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_ics721_governance::GovernanceError;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, GovernanceError> {
    cw_ics721_governance::instantiate(deps, info, msg.owner, msg.origin, msg.transfer_fee)?;
    Ok(Response::new())
}
```

Use the `#[cw_ics721_governance_execute]` macro to extend your execute message:

```rust
use cosmwasm_schema::cw_serde;
use cw_ics721_governance::cw_ics721_governance_execute;

#[cw_ics721_governance_execute]
#[cw_serde]
enum ExecuteMsg {
    Foo {},
    Bar {},
}
```

The macro inserts 2 new variants, `Governance` and `ReceiveNft`, to the enum:

```rust
#[cw_serde]
enum ExecuteMsg {
    Governance(cw_ics721_governance::Action),
    ReceiveNft(cw721::Cw721ReceiveMsg),
    Foo {},
    Bar {},
}
```

Where `Action` can be one of these:

- Owner: changing owner of proxy. Once set, it can't be set to None - except via migration!
- Origin: ICS721 contract where Cw721ReceiveMsg is forwarded to.
- Transfer Fee: optional transfer fee, if provided it will be checked whether funds have been send on `receive_nft` and `bridge_nft` is called. This means pratically only `bridge_nft` is eligible to call ics721, since `send_nft` is called by collection - and in case of base cw721 it doesn't send funds!
- Send Funds: from proxy to specified address.
- Bridge NFT: analogous to `cw721::Cw721ExecuteMsg::SendNft`, where NFT is transferred to ICS721 (escrow) and forwards `Cw721ReceiveMsg` to ICS721.

Handle the messages using the `execute` function provided by this crate:

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Governance(action) => {
            Ok(cw_ics721_governance::execute(deps, env, &info, action)?)
        }
        ExecuteMsg::ReceiveNft(msg) => {
            Ok(cw_ics721_governance::execute_receive_nft(deps, info, msg)?)
        }
    }
}
```

Use the `#[cw_ics721_governance_query]` macro to extend your query message:

```rust
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ics721_governance::cw_ics721_governance_query;

#[cw_ics721_governance_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}
```

The macro inserts a new variant, `Governance`:

```rust
#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg {
    #[returns(Governance)]
    Governance {},
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}
```

Handle the message using the `query_governance` function provided by this crate:

```rust
use cosmwasm_std::{entry_point, Deps, Env, Binary};
use cw_ics721_governance::query_governance;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Governance() => query_governance(deps.storage),
    }
}
```

# Kudos

This work has been adapted on the great work by Jake and Larry:
- DAO DAO: https://github.com/DA0-DA0/dao-contracts/blob/74bd3881fdd86829e5e8b132b9952dd64f2d0737/packages/dao-macros/src/lib.rs#L9
- CW++: https://github.com/larry0x/cw-plus-plus/tree/main/packages/ownable