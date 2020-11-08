use cosmwasm_std::{coin, Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, QueryResponse, log, to_binary, Uint128, HandleResult, Coin, CosmosMsg, BankMsg};
use crate::rand::Prng;
use crate::msg::{HandleMsg, InitMsg, QueryMsg, WinnerResponse};
use crate::state::{config, config_read, State, balance_read, balance};
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use rand::{RngCore, SeedableRng};
use rand::distributions::WeightedIndex;
use crate::coin_helpers::{assert_sent_sufficient_coin};

use sha2::{Digest, Sha256};
const MIN_STAKE_AMOUNT: u128 = 1;
pub const DENOM: &str = "uscrt";


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    // Init msg.item_count items
    let items_weighted: Vec<(CanonicalAddr, Uint128)> = Vec::default();

    //Create state
    let state = State {
        entries: items_weighted,
        contract_owner: deps.api.canonical_address(&env.message.sender)?,
        seed: msg.seed.as_bytes().to_vec(),
        entropy: Vec::default(),
        start_time: env.block.time,
        winner1: CanonicalAddr::default(),
        winner2: CanonicalAddr::default(),
        winner3: CanonicalAddr::default(),
        staked_tokens: Uint128::zero()
    };

    // Save to state
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::EndLottery { winner_to_select } => end_lottery(deps, env, winner_to_select),
        HandleMsg::Join { phrase } => { try_join(deps, env, phrase)},
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Joined { address } => query_joined(deps, address),
        QueryMsg::Winner {} => query_winner(deps),
        QueryMsg::Config {} => to_binary(&config_read(&deps.storage).load()?),
    }
}

fn throw_gen_err(msg: String) -> StdError {
    StdError::GenericErr {
        msg,
        backtrace: None,
    }
}

// Join the raffle by depositing funds into the contract
// if the user already joined, their stake/weight is increased
fn try_join<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    phrase: String
) -> StdResult<HandleResponse> {

    assert_sent_sufficient_coin(
        &env.message.sent_funds,
        Some(coin(MIN_STAKE_AMOUNT, DENOM)),
    )?;
    let sent_funds = env
        .message
        .sent_funds
        .iter()
        .find(|coin| coin.denom.eq(DENOM))
        .unwrap();

    let mut state = config(&mut deps.storage).load()?;

    let raw_address = deps.api.canonical_address(&env.message.sender)?;

    &state.entries.retain(|(k, _)| k != &raw_address);
    let key = &raw_address.as_slice();
    let mut current_balance = balance_read(&deps.storage).may_load(key)?.unwrap_or_default();

    current_balance += sent_funds.amount;
    balance(&mut deps.storage).save(key, &current_balance)?;

    let staked_tokens = state.staked_tokens.u128() + sent_funds.amount.u128();
    state.staked_tokens = Uint128::from(staked_tokens);
    config(&mut deps.storage).save(&state)?;

    state.entries.push((raw_address.clone(), current_balance));

    state.entropy.extend(phrase.as_bytes());
    state.entropy.extend(&raw_address.as_slice().to_vec());
    state.entropy.extend(env.block.chain_id.as_bytes().to_vec());
    state.entropy.extend(&env.block.height.to_be_bytes());
    state.entropy.extend(&env.block.time.to_be_bytes());

    state.entropy = Sha256::digest(&state.entropy).as_slice().to_vec();

    // Save state
    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(Binary(Vec::from("Joined successfully!")))
    })
}


fn send_tokens<A: Api>(
    api: &A,
    from_address: &CanonicalAddr,
    to_address: &CanonicalAddr,
    amount: Vec<Coin>,
    action: &str,
) -> HandleResult {
    let from_human = api.human_address(from_address)?;
    let to_human = api.human_address(to_address)?;
    let log = vec![log("action", action), log("to", to_human.as_str())];

    let r = HandleResponse {
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: from_human,
            to_address: to_human,
            amount,
        })],
        log,
        data: None,
    };
    Ok(r)
}


fn query_joined<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<QueryResponse> {
    let state = config_read(&deps.storage).load()?;

    let addr = deps.api.canonical_address(&address)?;

    if state.entries.iter().any(|i| i.0 == addr) {
        Ok(Binary(Vec::from(format!("{} joined", address))))
    } else {
        Ok(Binary(Vec::from(format!("{} has not joined", address))))
    }
}

fn query_winner<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;

    let w1 = if state.winner1 != CanonicalAddr::default() {
        deps.api.human_address(&state.winner1)?.to_string()
    } else {
        "not selected".to_string()
    };

    let w2 = if state.winner2 != CanonicalAddr::default() {
        deps.api.human_address(&state.winner2)?.to_string()
    } else {
        "not selected".to_string()
    };

    let w3 = if state.winner3 != CanonicalAddr::default() {
        deps.api.human_address(&state.winner3)?.to_string()
    } else {
        "not selected".to_string()
    };

    if state.winner1 != CanonicalAddr::default() || state.winner2 != CanonicalAddr::default() || state.winner3 != CanonicalAddr::default() {

        let resp = WinnerResponse {
            winner: format!("1st place: {}\n2nd place: {}\n3rd place: {}", w1, w2, w3)
        };
        to_binary(&resp)
    } else {

        let resp = WinnerResponse {
            winner: "Winner not selected yet!".to_string()
        };
        to_binary(&resp)
    }
}

fn end_lottery<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    winner_to_select: u8,
) -> HandleResult {
    // TODO Check if contract has expired
    let mut state = config(&mut deps.storage).load()?;

    // add this if you don't want to allow choosing an alternative winner
    if state.winner1 != CanonicalAddr::default() {
        // game already ended
        return Ok(HandleResponse::default());
    }

    if deps.api.canonical_address(&env.message.sender)? != state.contract_owner {
        return Err(throw_gen_err("You cannot trigger lottery end unless you're the owner!".to_string()));
    }
    // let contract_addr: HumanAddr = deps.api.human_address(&env.contract.address)?;

    // this way every time we call the end_lottery function we will get a different result. Plus it's going to be pretty hard to
    // predict the exact time of the block, so less chance of cheating
    state.entropy.extend_from_slice(&env.block.time.to_be_bytes());

    let entry_iter = &state.entries.clone();
    let weight_iter = &state.entries.clone();
    let entries: Vec<_> = entry_iter.into_iter().map(|(k, _)| k).collect();
    let weights: Vec<_> = weight_iter.into_iter().map(|(_, v)| v.u128()).collect();

    let mut hasher = Sha256::new();

    // write input message
    hasher.update(&state.seed);
    hasher.update(&state.entropy);
    let hash = hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_slice());

    let mut rng: ChaChaRng = ChaChaRng::from_seed(result);

    let dist = WeightedIndex::new(&weights).unwrap();
    let sample = dist.sample(&mut rng).clone();
    let winner = entries[sample];

    match winner_to_select {
        1 => {
            state.winner1 =  winner.clone();
        },
        2 => {
            state.winner2 =  winner.clone();
        },
        3 => {
            state.winner3 =  winner.clone();
        },
        _ => {
            return Err(throw_gen_err(format!("bad winner selection")));
        }
    }

    config(&mut deps.storage).save(&state)?;

    let contract_address_raw = deps.api.canonical_address(&env.contract.address)?;
    send_tokens(
        &deps.api,
        &contract_address_raw,
        &winner.clone(),
        vec![coin(state.staked_tokens.u128(), DENOM)],
        "winner winner chicken dinner",
    )
}

