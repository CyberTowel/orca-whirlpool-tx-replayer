use anchor_client::anchor_lang::prelude::borsh::de;
use num::ToPrimitive;
use num_bigfloat::{BigFloat, RoundingMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionTokenBalance,
};
use std::{collections::HashMap, fmt::Binary, str::FromStr};

use crate::{pool_state::LiquidityStateLayoutV4, transaction::Transaction};
const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

#[derive(Debug)]
pub enum Error {}

#[derive(Default, Debug, Clone)]
pub struct PoolVars {
    // pub pool_coin_token_account: String,
    pub pool_id: String,
    // pub amm_target_orders: String,
    // pub token_a_address: String,
    // pub token_b_address: String,
}

#[derive(Default, Debug, Clone)]
pub struct BalanceChange {
    pub owner: String,
    pub mint: String,
    pub balance_pre: BigFloat,
    pub balance_post: BigFloat,
    pub difference: BigFloat,
}

pub fn get_token_amounts(
    rpc_transaction: &EncodedConfirmedTransactionWithStatusMeta,
    account_keys: &Vec<Value>,
    ubo: &str,
    token_a_address: &str,
    token_b_address: &str,
    token_a_taa: &str,
    token_b_taa: &str,
    token_a_decimals: u64,
    token_b_decimals: u64,
    // pool_state: &LiquidityStateLayoutV4,
) -> TokenAmountsSwap {
    let (token_changes_by_wallet, changes_by_token_account_address) =
        parse_balance_changes(rpc_transaction, account_keys);

    let token_changes_ubo = token_changes_by_wallet.get(ubo).unwrap();
    // let token_changes_pool_old = token_changes_by_wallet.get(RAYDIUM_AUTHORITY).unwrap();

    let token_changes_pool_new_a = changes_by_token_account_address.get(token_a_taa).unwrap();
    let token_changes_pool_new_b = changes_by_token_account_address.get(token_b_taa).unwrap();

    let token_changes_pool = merge_hashmap(
        token_changes_pool_new_a.clone(),
        token_changes_pool_new_b.clone(),
    );

    let token_amounts_a = parse_token_amounts(
        token_changes_ubo,
        &token_changes_pool,
        token_a_address,
        token_a_decimals,
    );

    let token_amounts_b = parse_token_amounts(
        token_changes_ubo,
        &token_changes_pool,
        token_b_address,
        token_b_decimals,
    );

    let token_b_price_rel = token_amounts_b.amount_total_pool / token_amounts_a.amount_total_pool;

    // let decimals_correct = token_b_decimals as i64 - token_a_decimals as i64;

    // let price_token_ref = token_b_price_rel
    //     * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(decimals_correct)));

    let token_amounts_swap = TokenAmountsSwap {
        token_a_address: token_a_address.to_string(),
        token_b_address: token_b_address.to_string(),
        token_amounts_a: token_amounts_a,
        token_amounts_b: token_amounts_b,
        price_usd_token_b: token_b_price_rel,
        price_token_b_rel: token_b_price_rel,
        price_usd_token_b_formatted: token_b_price_rel.to_string(),
    };

    return token_amounts_swap;
}

fn merge_hashmap(
    mut map1: HashMap<std::string::String, BalanceChange>,
    map2: HashMap<std::string::String, BalanceChange>,
) -> HashMap<std::string::String, BalanceChange> {
    for (item, change) in map2.clone().into_iter() {
        if map1.contains_key(&item) {
            let existing = map1.get(&item).unwrap();
            let new_balance = change.balance_post;
            let new_diff = change.difference;

            let new_balance_post = existing.balance_post + new_balance;
            let new_diff_post = existing.difference + new_diff;

            let new_change = BalanceChange {
                balance_pre: existing.balance_pre,
                balance_post: new_balance_post,
                difference: new_diff_post,
                mint: existing.mint.clone(),
                owner: existing.owner.clone(),
            };

            map1.insert(item, new_change);
        } else {
            map1.insert(item, change);
        }
    }

    return map1;

    // map1.into_iter().chain(map2).collect()
}

pub fn parse_balance_changes(
    transaction: &EncodedConfirmedTransactionWithStatusMeta,
    account_keys: &Vec<Value>,
) -> (
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>,
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>,
) {
    let post_balances = transaction.transaction.clone().meta.unwrap().post_balances;
    let pre_balances = transaction.transaction.clone().meta.unwrap().pre_balances;

    let post_token_balances: Option<Vec<UiTransactionTokenBalance>> = transaction
        .transaction
        .clone()
        .meta
        .unwrap()
        .post_token_balances
        .into();

    let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> = transaction
        .transaction
        .clone()
        .meta
        .unwrap()
        .pre_token_balances
        .into();

    let mut changes_by_owner: HashMap<String, HashMap<String, BalanceChange>> = HashMap::new();

    let mut changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChange>> =
        HashMap::new();

    for balance in post_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;
        let owner_address = owner.unwrap();

        let index_usize = balance.account_index.to_usize().unwrap();
        // println!(
        //     "post_balances account index: {:#?}, address at index {:#?}",
        //     balance``.account_index, account_keys[index_usize]
        // );

        let pub_key_token_address = account_keys[index_usize]["pubkey"].as_str().unwrap();

        let owner_entry = changes_by_owner.entry(owner_address.clone());
        let token_entry = owner_entry.or_default().entry(mint.clone());

        let token_account_address_entry =
            changes_by_token_account_address.entry(pub_key_token_address.to_string());

        let token_entry_token_account_address =
            token_account_address_entry.or_default().entry(mint.clone());

        let amount_bf = BigFloat::from_str(&amount).unwrap();

        *token_entry.or_default() = BalanceChange {
            balance_pre: BigFloat::from_f64(0.0),
            balance_post: amount_bf,
            difference: amount_bf,
            mint: mint.clone(),
            owner: owner_address.clone(),
        };

        *token_entry_token_account_address.or_default() = BalanceChange {
            balance_pre: BigFloat::from_f64(0.0),
            balance_post: amount_bf,
            difference: amount_bf,
            mint: mint.clone(),
            owner: owner_address.clone(),
        };
    }

    for balance in pre_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;
        let owner_address = owner.unwrap();

        let owner_address_c = owner_address.clone();

        let index_usize = balance.account_index.to_usize().unwrap();
        let pub_key_token_address = account_keys[index_usize]["pubkey"].as_str().unwrap();

        let owner_entry = changes_by_owner.entry(owner_address);
        let token_entry = owner_entry.or_default().entry(mint.clone());

        let token_account_address_entry =
            changes_by_token_account_address.entry(pub_key_token_address.to_string());

        let token_entry_token_account_address =
            token_account_address_entry.or_default().entry(mint.clone());

        let amount_bf = BigFloat::from_str(&amount).unwrap();

        let existing_entry = token_entry.or_default();

        let existing_entry_token_account = token_entry_token_account_address.or_default();

        existing_entry.balance_pre = amount_bf;
        existing_entry.difference = existing_entry.balance_post - amount_bf;

        existing_entry_token_account.balance_pre = amount_bf;
        existing_entry_token_account.difference =
            existing_entry_token_account.balance_post - amount_bf;
    }

    for (index, account_key) in account_keys.iter().enumerate() {
        let pubkey = account_key["pubkey"].as_str().unwrap();

        let pre = BigFloat::from_u64(pre_balances[index]);
        let post = BigFloat::from_u64(post_balances[index]);

        let item = BalanceChange {
            balance_pre: pre,
            balance_post: post,
            difference: post - pre,
            mint: "sol".to_string(),
            owner: pubkey.to_string(),
        };

        let owner_entry = changes_by_owner.entry(pubkey.to_string());
        let token_entry = owner_entry.or_default().entry("sol".to_string());

        let token_account_address_entry =
            changes_by_token_account_address.entry(pubkey.to_string());

        let token_entry_token_account_address = token_account_address_entry
            .or_default()
            .entry("sol".to_string());

        *token_entry.or_default() = item.clone();

        *token_entry_token_account_address.or_default() = item.clone();
    }

    return (changes_by_owner, changes_by_token_account_address);
}

#[derive(Default, Debug, Clone)]
pub struct TokenPriceResult {
    pub token_price_rel_to_ref: BigFloat,
    pub token_price_usd_18: BigFloat,
    pub token_price_usd_fixed: BigFloat,
    pub token_ref_price_usd_18: BigFloat,
    pub token_ref_price_usd_fixed: BigFloat,
}

pub fn get_price(
    token_price_rel_a_to_b: &String,
    token_ref: &String,
    sol_price_db: &String,
    decimals_correct: i64,
) -> Result<TokenPriceResult, Error> {
    let stable_coin_ref = token_ref == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let token_price_rel_to_ref = BigFloat::from_str(token_price_rel_a_to_b).unwrap();

    let token_ref_price_usd_18 = if stable_coin_ref {
        (BigFloat::from_i16(1) * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18))))
    } else {
        BigFloat::from_str(sol_price_db).unwrap()
    };

    let token_price_usd_18 = if stable_coin_ref {
        let token_rel_to_ref = BigFloat::from_str(token_price_rel_a_to_b).unwrap();

        let token_price_base = token_ref_price_usd_18 / token_rel_to_ref;

        let token_price_18 = token_price_base
            * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-decimals_correct)));

        println!(
            "decimals_correct: {:#?}, token_price_base {:#?} token_price_18: {:#?}",
            decimals_correct,
            token_price_base.to_f64().to_string(),
            token_price_18.to_f64().to_string()
        );
        let _token_price_fixed =
            token_price_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

        token_price_18
    } else {
        let usd_price_token_b = token_price_rel_to_ref * BigFloat::from_str(sol_price_db).unwrap();

        usd_price_token_b
    };

    let token_price_usd_fixed =
        token_price_usd_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let token_ref_price_usd_fixed =
        token_ref_price_usd_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let result = TokenPriceResult {
        token_price_rel_to_ref: token_price_rel_to_ref,
        token_price_usd_18: token_price_usd_18,
        token_price_usd_fixed: token_price_usd_fixed,
        token_ref_price_usd_18: token_ref_price_usd_18,
        token_ref_price_usd_fixed: token_ref_price_usd_fixed,
    };

    Ok(result)
}

fn parse_token_amounts(
    token_changes_ubo: &HashMap<String, BalanceChange>,
    token_changes_pool: &HashMap<String, BalanceChange>,
    token_address: &str,
    token_decimals: u64,
) -> TokenAmounts {
    let mut token_amount_ubo = match token_changes_ubo.get(token_address) {
        Some(x) => x.balance_post,
        None => BigFloat::from_i64(0),
    };

    let mut token_amount_pool = match token_changes_pool.get(token_address) {
        Some(x) => x.balance_post,
        None => BigFloat::from_i64(0),
    };

    let mut amount_diff_ubo = match token_changes_ubo.get(token_address) {
        Some(x) => x.difference,
        None => BigFloat::from_i64(0),
    };

    let mut amount_diff_pool = match token_changes_pool.get(token_address) {
        Some(x) => x.difference,
        None => BigFloat::from_i64(0),
    };

    if token_address == "So11111111111111111111111111111111111111112" {
        let native_sol_amount = token_changes_ubo.get("sol");
        let amount = match native_sol_amount {
            Some(x) => x.balance_post,
            None => BigFloat::from_i64(0),
        };

        token_amount_ubo += amount;

        let amount_diff = match native_sol_amount {
            Some(x) => x.difference,
            None => BigFloat::from_i64(0),
        };

        amount_diff_ubo += amount_diff;

        //
        //

        // let native_sol_amount_pool = token_changes_pool.get("sol");
        // let amount_pool = match native_sol_amount_pool {
        //     Some(x) => x.balance_post,
        //     None => 0,
        // };

        // token_amount_pool += amount_pool;

        // let amount_diff_pool_ne = match native_sol_amount_pool {
        //     Some(x) => x.difference,
        //     None => 0,
        // };

        // amount_diff_pool += amount_diff_pool_ne;
    }

    let token_perc_ubo = parse_token_amount_rounded(token_amount_ubo, token_amount_pool);

    let amount_total_pool_bf = token_amount_pool
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18 - (token_decimals as i64))));

    let token_amount_ubo_bf = token_amount_ubo
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18 - (token_decimals as i64))));

    let amount_diff_pool_bf = amount_diff_pool
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18 - (token_decimals as i64))));

    let amount_diff_ubo_bf = amount_diff_ubo
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18 - (token_decimals as i64))));

    let token_amount_pool_18 = amount_total_pool_bf;

    let token_amount_ubo_18 = token_amount_ubo_bf;

    let amount_diff_pool_18 = amount_diff_pool_bf;

    let amount_diff_ubo_18 = amount_diff_ubo_bf;

    return TokenAmounts {
        amount_total_pool: token_amount_pool,
        amount_total_ubo: token_amount_ubo,
        amount_diff_pool: amount_diff_pool,
        amount_diff_ubo: amount_diff_ubo,
        token_address: token_address.to_string(),
        amount_total_pool_18: token_amount_pool_18,
        amount_total_ubo_18: token_amount_ubo_18,
        amount_diff_pool_18: amount_diff_pool_18,
        amount_diff_ubo_18: amount_diff_ubo_18,
    };
}

// fn parse_swap_token_amounts(
//     token_changes_ubo: &HashMap<String, BalanceChange>,
//     token_changes_pool: &HashMap<String, BalanceChange>,
//     token_a_address: &str,
//     token_b_address: &str,
// ) -> TokenAmounts {
//     let mut token_amount_ubo_a = match token_changes_ubo.get(token_a_address) {
//         Some(x) => x.balance_post,
//         None => 0,
//     };

//     let token_amount_ubo_b = match token_changes_ubo.get(token_b_address) {
//         Some(x) => x.balance_post,
//         None => 0,
//     };

//     let token_amount_pool_a = match token_changes_pool.get(token_a_address) {
//         Some(x) => x.balance_post,
//         None => 0,
//     };

//     let token_amount_pool_b = match token_changes_pool.get(token_b_address) {
//         Some(x) => x.balance_post,
//         None => 0,
//     };

//     let mut amount_diff_ubo_a = match token_changes_ubo.get(token_a_address) {
//         Some(x) => x.difference,
//         None => 0,
//     };

//     let amount_diff_ubo_b = match token_changes_ubo.get(token_b_address) {
//         Some(x) => x.difference,
//         None => 0,
//     };

//     if token_a_address == "So11111111111111111111111111111111111111112" {
//         let native_sol_amount = token_changes_ubo.get("sol");
//         let amount = match native_sol_amount {
//             Some(x) => x.balance_post,
//             None => 0,
//         };

//         token_amount_ubo_a += amount;

//         let amount_diff = match native_sol_amount {
//             Some(x) => x.difference,
//             None => 0,
//         };

//         amount_diff_ubo_a += amount_diff;
//     }

//     let token_perc_ubo_a = parse_token_amount_rounded(token_amount_ubo_a, token_amount_pool_a);
//     let token_perc_ubo_b = parse_token_amount_rounded(token_amount_ubo_b, token_amount_pool_b);

//     let token_b_price_rel =
//         BigFloat::from_i64(token_amount_pool_a) / BigFloat::from_i64(token_amount_pool_b);

//     let price_token_ref =
//         token_b_price_rel * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-6)));

//     return TokenAmounts {
//         amount_total_pool_a: token_amount_pool_a,
//         amount_total_pool_b: token_amount_pool_b,
//         amount_total_ubo_a: token_amount_ubo_a,
//         amount_total_ubo_b: token_amount_ubo_b,
//         amount_diff_ubo_a: amount_diff_ubo_a,
//         amount_diff_ubo_b: amount_diff_ubo_b,
//         perc_ubo_a: token_perc_ubo_a,
//         perc_ubo_b: token_perc_ubo_b,
//         perc_ubo_a_formatted: token_perc_ubo_a.to_string(),
//         perc_ubo_b_formatted: token_perc_ubo_b.to_string(),
//         token_b_price_rel: price_token_ref.to_f64(),
//     };
// }

fn parse_token_amount_rounded(token_amount: BigFloat, token_amount_pool: BigFloat) -> BigFloat {
    let ubo_token_a_perc_f = token_amount / token_amount_pool * BigFloat::from_f64(100.0);

    // let parse_token_amount_rounded = ubo_token_a_perc_f;

    return ubo_token_a_perc_f;
}

#[derive(Debug)]
pub struct TokenAmountsSwap {
    pub token_amounts_a: TokenAmounts,
    pub token_amounts_b: TokenAmounts,
    pub price_token_b_rel: BigFloat,
    pub price_usd_token_b: BigFloat,
    pub price_usd_token_b_formatted: String,
    pub token_a_address: String,
    pub token_b_address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenAmounts {
    pub token_address: String,
    pub amount_total_pool: BigFloat,
    pub amount_diff_pool: BigFloat,
    pub amount_total_ubo: BigFloat,
    pub amount_diff_ubo: BigFloat,
    pub amount_total_pool_18: BigFloat,
    pub amount_diff_pool_18: BigFloat,
    pub amount_total_ubo_18: BigFloat,
    pub amount_diff_ubo_18: BigFloat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmountsPriced {
    usd_total_pool_18_rounded: BigFloat,
    usd_diff_pool_18_rounded: BigFloat,
    usd_total_ubo_18_rounded: BigFloat,
    usd_diff_ubo_18_rounded: BigFloat,

    usd_total_pool_18: BigFloat,
    usd_diff_pool_18: BigFloat,
    usd_total_ubo_18: BigFloat,
    usd_diff_ubo_18: BigFloat,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTokenAmountsPriced {
    pub token_amounts_priced_a: TokenAmountsPriced,
    pub token_amounts_priced_b: TokenAmountsPriced,
    pub usd_total_pool_18: BigFloat,
    pub usd_total_pool_18_rounded: BigFloat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPriceOracleValues {
    pub ubo: String,
    pub signer: String,
    pub pool_address: String,
    pub token_address: String,
    pub signature: String,

    pub usd_total_pool: BigFloat,
    pub usd_total_ubo: BigFloat,
    pub usd_diff_ubo: BigFloat,
    pub usd_diff_pool: BigFloat,

    pub amount_total_pool: BigFloat,
    pub amount_diff_pool: BigFloat,
    pub amount_total_ubo: BigFloat,
    pub amount_diff_ubo: BigFloat,
}

pub struct TokenAmountsPriced18 {}

pub fn parse_token_price_oracle_values(
    ubo: String,
    signer: String,
    pool_id: String,
    token_address: String,
    token_amounts: &TokenAmounts,
    token_amounts_usd: &TokenAmountsPriced,
    signature: &String,
) -> TokenPriceOracleValues {
    let tpo_values_a = TokenPriceOracleValues {
        ubo: ubo,
        signer: signer,
        pool_address: pool_id,
        token_address: token_address,
        signature: signature.to_string(),

        usd_total_pool: token_amounts_usd.usd_total_pool_18_rounded,
        usd_total_ubo: token_amounts_usd.usd_total_ubo_18_rounded,
        usd_diff_ubo: token_amounts_usd.usd_diff_ubo_18_rounded,
        usd_diff_pool: token_amounts_usd.usd_diff_pool_18_rounded,

        amount_total_pool: token_amounts.amount_total_pool,
        amount_diff_pool: token_amounts.amount_diff_pool,
        amount_total_ubo: token_amounts.amount_total_ubo,
        amount_diff_ubo: token_amounts.amount_diff_ubo,
    };

    return tpo_values_a;
}

pub fn parse_token_amounts_new(
    token_amounts: &TokenAmountsSwap,
    token_prices: &TokenPriceResult,
    // token_a_decimals: u64,
    // token_b_decimals: u64,
) -> SwapTokenAmountsPriced {
    let token_usd_a = multiply_token_amounts_to_usd(
        &token_amounts.token_amounts_a,
        token_prices.token_ref_price_usd_18,
    );

    let token_usd_b = multiply_token_amounts_to_usd(
        &token_amounts.token_amounts_b,
        token_prices.token_price_usd_18,
    );

    let usd_total_pool_18 = token_usd_a.usd_total_pool_18 + token_usd_b.usd_total_pool_18;

    let usd_total_pool_18_rounded = usd_total_pool_18.round(0, RoundingMode::ToOdd);

    let values = SwapTokenAmountsPriced {
        token_amounts_priced_a: token_usd_a,
        token_amounts_priced_b: token_usd_b,
        usd_total_pool_18: usd_total_pool_18,
        usd_total_pool_18_rounded,
    };

    return values;
}

pub struct UsdMultiplierResult {
    pub amount_18: BigFloat,
    pub amount_18_rounded: BigFloat,
}

fn multiply_amounts(token_price_usd_18: BigFloat, amount_18: BigFloat) -> UsdMultiplierResult {
    let token_price_usd_fixed =
        token_price_usd_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let amount_fixed = amount_18;

    let total_amount_usd = token_price_usd_fixed * amount_fixed;

    let total_amount_usd_rounded = total_amount_usd.round(0, RoundingMode::ToOdd);
    // .unwrap();

    // let rounded = total_amount_usd_rounded.round();

    return UsdMultiplierResult {
        amount_18: total_amount_usd,
        amount_18_rounded: total_amount_usd_rounded,
    };
}

fn multiply_token_amounts_to_usd(
    token_amounts: &TokenAmounts,
    token_price_18: BigFloat,
) -> TokenAmountsPriced {
    let usd_total_pool_18 = multiply_amounts(token_price_18, token_amounts.amount_total_pool_18);

    let usd_diff_pool_18 = multiply_amounts(token_price_18, token_amounts.amount_diff_pool_18);

    let usd_total_ubo_18 = multiply_amounts(token_price_18, token_amounts.amount_total_ubo_18);

    let usd_diff_ubo_18 = multiply_amounts(token_price_18, token_amounts.amount_diff_ubo_18);

    let token_amounts_priced = TokenAmountsPriced {
        usd_total_pool_18: usd_total_pool_18.amount_18,
        usd_total_pool_18_rounded: usd_total_pool_18.amount_18_rounded,

        usd_diff_pool_18: usd_diff_pool_18.amount_18,
        usd_diff_pool_18_rounded: usd_diff_pool_18.amount_18_rounded,

        usd_total_ubo_18: usd_total_ubo_18.amount_18,
        usd_total_ubo_18_rounded: usd_total_ubo_18.amount_18_rounded,

        usd_diff_ubo_18: usd_diff_ubo_18.amount_18,
        usd_diff_ubo_18_rounded: usd_diff_ubo_18.amount_18_rounded,
    };

    return token_amounts_priced;
}
