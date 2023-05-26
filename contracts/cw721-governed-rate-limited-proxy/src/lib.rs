use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw721::Cw721ReceiveMsg;
use cw_rate_limiter::Rate;
use error::ContractError;
use state::RATE_LIMITER;

pub mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "library"))]
pub mod entry {
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
    use crate::state::{CONTRACT_NAME, CONTRACT_VERSION, RATE_LIMITER};
    use crate::{execute_bridge_nft, execute_rate_limit, execute_receive_nft};

    use cosmwasm_std::{entry_point, to_binary};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw2::set_contract_version;
    use cw_ics721_governance::Action;
    use cw_rate_limiter::Rate;

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        if msg.rate_limit.is_zero() {
            Err(ContractError::ZeroRate {})
        } else {
            let (rate, units) = match msg.rate_limit {
                Rate::PerBlock(rate) => (rate, "nfts_per_block"),
                Rate::Blocks(rate) => (rate, "blocks_per_nft"),
            };
            RATE_LIMITER.init(deps.storage, &msg.rate_limit)?;
            let res = cw_ics721_governance::instantiate(
                deps,
                info,
                msg.owner,
                msg.origin,
                msg.transfer_fee,
            )?;
            Ok(res
                .add_attribute("rate".to_string(), rate.to_string())
                .add_attribute("units", units))
        }
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Governance(Action::BridgeNft {
                collection,
                token_id,
                msg,
            }) => execute_bridge_nft(deps, env, info, collection, token_id, msg),
            ExecuteMsg::Governance(action) => {
                Ok(cw_ics721_governance::execute(deps, env, &info, action)?)
            }
            ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
            ExecuteMsg::RateLimit(rate) => execute_rate_limit(deps, env, info, rate),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Governance() => cw_ics721_governance::query_governance(deps.storage),
            QueryMsg::RateLimit {} => to_binary(&RATE_LIMITER.query_limit(deps.storage)?),
        }
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
        // Set contract to version to latest
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        match msg {
            MigrateMsg::WithUpdate {
                origin,
                owner,
                transfer_fee,
                rate_limit,
            } => {
                if let Some(rate) = rate_limit {
                    if rate.is_zero() {
                        return Err(ContractError::ZeroRate {});
                    } else {
                        RATE_LIMITER.init(deps.storage, &rate)?;
                    }
                }
                let res = cw_ics721_governance::migrate(deps, owner, origin, transfer_fee)?;
                Ok(res.add_attribute("rate_limit", format!("{:?}", rate_limit)))
            }
        }
    }
}

pub fn execute_rate_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    rate_limit: Rate,
) -> Result<Response, ContractError> {
    cw_ics721_governance::assert_owner(deps.storage, &info.sender)?;
    if rate_limit.is_zero() {
        Err(ContractError::ZeroRate {})
    } else {
        RATE_LIMITER.init(deps.storage, &rate_limit)?;
        let (rate, units) = match rate_limit {
            Rate::PerBlock(rate) => (rate, "nfts_per_block"),
            Rate::Blocks(rate) => (rate, "blocks_per_nft"),
        };
        Ok(Response::default()
            .add_attribute("method", "execute_rate_limit")
            .add_attribute("rate", rate.to_string())
            .add_attribute("units", units))
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    RATE_LIMITER.limit(deps.storage, &env, info.sender.as_str())?;
    Ok(cw_ics721_governance::execute_receive_nft(deps, info, msg)?)
}

pub fn execute_bridge_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    RATE_LIMITER.limit(deps.storage, &env, info.sender.as_str())?;
    Ok(cw_ics721_governance::execute_bridge_nft(
        deps, env, &info, collection, token_id, msg,
    )?)
}
