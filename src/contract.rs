#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cosmwasm_std::entry_point;

use cw2::set_contract_version;
use cw20::{Balance, Cw20ExecuteMsg, Cw20ReceiveMsg, Cw20CoinVerified};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InstantiateMsg, QueryMsg, ReceiveMsg};
use crate::state::Recipient;

// version info for migration info
const CONTRACT_NAME: &str = "empty-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // no setup
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<Response, ContractError> {
    match msg {
        HandleMsg::Receive(msg) => execute_receive(deps, info, msg),
        HandleMsg::Send { recipients } => execute_send(deps, recipients),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: wrapper.amount,
    });
    match msg {
        ReceiveMsg::Send { recipients } => execute_send(deps, recipients),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute_send(
    deps: DepsMut,
    recipients: Vec<Recipient>,
) -> Result<Response, ContractError> {
    let messages = send_tokens(deps.api, recipients)?;
    let attributes = vec![attr("action", "send")];
    Ok(Response::default())
}

fn send_tokens(
    api: &dyn Api,
    recipients: Vec<Recipient>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    for recipient in recipients {
        let native_balance = &recipient.clone().amount.native;
        let mut native_msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
            vec![]
        } else {
            vec![BankMsg::Send {
                to_address: recipient.clone().address.into(),
                amount: native_balance.to_vec(),
            }
            .into()]
        };

        let cw20_balance = &recipient.amount.cw20;
        let cw20_msgs: StdResult<Vec<_>> = cw20_balance
            .iter()
            .map(|c| {
                let msg = Cw20ExecuteMsg::Transfer {
                    recipient: recipient.clone().address.into(),
                    amount: c.amount,
                };
                let exec = WasmMsg::Execute {
                    contract_addr: c.clone().address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                };
                Ok(exec.into())
            })
            .collect();

        native_msgs.append(&mut cw20_msgs?);
        msgs.append(&mut native_msgs);
    }

    // assert balance
    Ok(msgs)
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}