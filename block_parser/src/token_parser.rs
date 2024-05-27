// use anchor_client::anchor_lang::prelude::borsh::de;
// use num::ToPrimitive;
use num::ToPrimitive;
use num_bigfloat::{BigFloat, RoundingMode};
use serde::{de::value, Deserialize, Serialize};

use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionTokenBalance};
use std::{collections::HashMap, str::FromStr};

use crate::interfaces::{BalanceChange, CtTransaction, TransactionFees};

#[derive(Debug)]
pub enum Error {}

// #[derive(Default, Debug, Clone)]
// pub struct BalanceChange {
//     pub owner: String,
//     pub mint: String,
//     pub balance_pre: BigFloat,
//     pub balance_post: BigFloat,
//     pub difference: BigFloat,
// }

#[derive(Default, Debug, Clone)]
pub struct TokenPriceResult {
    pub token_quote_price_18: BigFloat,
    pub token_new_price_18: BigFloat,
    pub token_new_price_fixed: BigFloat,
    pub token_new_price_in_token_quote_18: BigFloat,
    pub token_new_price_in_token_quote_fixed: BigFloat,

    pub token_trade_price_18: BigFloat,
    pub token_trade_price_fixed: BigFloat,
    pub token_trade_price_in_token_quote_18: BigFloat,
    pub token_trade_price_in_token_quote_fixed: BigFloat,
}

#[derive(Debug)]
pub struct TokenAmountsSwap {
    pub token_amounts_quote: TokenAmounts,
    pub token_amounts_base: TokenAmounts,
    // pub token_b_price_fixed: BigFloat,
    // pub price_usd_token_base: BigFloat,
    pub token_new_price_in_token_quote_18: BigFloat,
    pub token_trade_price_in_token_quote_18: BigFloat,
    // pub token_quote_address: String,
    // pub token_base_address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenAmounts {
    pub token_address: String,
    // pub amount_total_pool: BigFloat,
    // pub amount_total_pool_pre: BigFloat,
    // pub amount_diff_pool: BigFloat,
    // pub amount_total_ubo: BigFloat,
    // pub amount_diff_ubo: BigFloat,
    pub amount_total_pool_18: BigFloat,
    pub amount_diff_pool_18: BigFloat,
    pub amount_total_ubo_18: BigFloat,
    pub amount_diff_ubo_18: BigFloat,
    pub amount_total_pool_pre_18: BigFloat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTokenAmountsPriced {
    pub token_amounts_priced_a: TokenAmountsPriced,
    pub token_amounts_priced_b: TokenAmountsPriced,
    pub usd_total_pool_18: BigFloat,
    pub usd_total_pool_18_rounded: BigFloat,
}

pub struct UsdMultiplierResult {
    pub amount_18: BigFloat,
    pub amount_18_rounded: BigFloat,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMeta {
    pub base_decimal: u64,
    pub base_lot_size: u64,
    pub base_need_take_pnl: u64,
    pub base_total_pnl: u64,
    pub base_total_deposited: u128,
    pub base_vault: Pubkey,
    pub base_mint: Pubkey,

    pub quote_decimal: u64,
    pub quote_lot_size: u64,
    pub quote_need_take_pnl: u64,
    pub quote_total_pnl: u64,
    pub quote_total_deposited: u128,
    pub quote_vault: Pubkey,
    pub quote_mint: Pubkey,
}

pub fn get_token_amounts(
    transaction_base: &CtTransaction,
    _account_keys: &Vec<String>,
    ubo: &str,
    quote_mint_address: &str,
    base_mint_address: &str,
    quote_vault_address: &str,
    base_vault_address: &str,
    quote_decimal: u64,
    base_decimal: u64,
) -> Option<TokenAmountsSwap> {
    // let (token_changes_by_wallet, changes_by_token_account_address) =
    //     parse_balance_changes(rpc_transaction, account_keys);

    // transaction_base.changes_by_token_account_address

    let token_changes_ubo = transaction_base
        .token_changes_owner
        .values
        .get(ubo)
        .unwrap();

    let token_changes_pool_new_a_req = transaction_base
        .token_changes_token_account
        .values
        .get(quote_vault_address);

    let token_changes_pool_new_b_req = transaction_base
        .token_changes_token_account
        .values
        .get(base_vault_address);

    if token_changes_pool_new_a_req.is_none() {
        return None;
        // return TokenAmountsSwap {
        //     token_amounts_quote: TokenAmounts {
        //         token_address: quote_mint_address.to_string(),
        //         amount_total_pool_18: BigFloat::from_i64(0),
        //         amount_diff_pool_18: BigFloat::from_i64(0),
        //         amount_total_ubo_18: BigFloat::from_i64(0),
        //         amount_diff_ubo_18: BigFloat::from_i64(0),
        //         amount_total_pool_pre_18: BigFloat::from_i64(0),
        //     },
        //     token_amounts_base: TokenAmounts {
        //         token_address: base_mint_address.to_string(),
        //         amount_total_pool_18: BigFloat::from_i64(0),
        //         amount_diff_pool_18: BigFloat::from_i64(0),
        //         amount_total_ubo_18: BigFloat::from_i64(0),
        //         amount_diff_ubo_18: BigFloat::from_i64(0),
        //         amount_total_pool_pre_18: BigFloat::from_i64(0),
        //     },
        //     token_new_price_in_token_quote_18: BigFloat::from_i64(0),
        //     token_trade_price_in_token_quote_18: BigFloat::from_i64(0),
        // };
    }
    // .unwrap();

    let token_changes_pool_new_a = token_changes_pool_new_a_req.unwrap();
    let token_changes_pool_new_b = token_changes_pool_new_b_req.unwrap();

    let token_changes_pool = merge_hashmap(
        token_changes_pool_new_a.clone(),
        token_changes_pool_new_b.clone(),
    );

    let token_amounts_quote = parse_token_amounts(
        token_changes_ubo,
        &token_changes_pool,
        quote_mint_address,
        quote_decimal,
    );

    let token_amounts_base = parse_token_amounts(
        token_changes_ubo,
        &token_changes_pool,
        base_mint_address,
        base_decimal,
    );

    // let token_b_price_rel =
    //     token_amounts_base.amount_total_pool_18 / token_amounts_quote.amount_total_pool_18;

    let token_base_price_in_token_quote =
        token_amounts_quote.amount_total_pool_18 / token_amounts_base.amount_total_pool_18;

    let token_buy_price_in_token_qoute =
        token_amounts_quote.amount_total_pool_pre_18 / token_amounts_base.amount_total_pool_pre_18;

    // let token_b_price_fixed = token_b_price_rel_ref
    //     * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(decimals_correct)));

    let token_new_price_in_token_quote_18 = token_base_price_in_token_quote
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let token_trade_price_in_token_qoute_18 = token_buy_price_in_token_qoute
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    // let token_b_price_rel_new =
    //     token_amounts_base.amount_total_pool_18_pre / token_amounts_quote.amount_total_pool_18_pre;

    let token_amounts_swap = TokenAmountsSwap {
        // token_quote_address: quote_mint_address.to_string(),
        // token_base_address: base_mint_address.to_string(),
        token_amounts_quote,
        token_amounts_base,
        // price_usd_token_base: token_b_price_rel,
        // token_b_price_fixed: token_b_price_fixed,
        token_new_price_in_token_quote_18,
        token_trade_price_in_token_quote_18: token_trade_price_in_token_qoute_18,
    };

    return Some(token_amounts_swap);
}

pub fn parse_balance_changes(
    transaction: &EncodedTransactionWithStatusMeta,
    account_keys: &Vec<String>,
) -> (
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>,
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>,
) {
    let post_balances = transaction.clone().meta.unwrap().post_balances;
    let pre_balances = transaction.clone().meta.unwrap().pre_balances;

    let post_token_balances: Option<Vec<UiTransactionTokenBalance>> =
        transaction.clone().meta.unwrap().post_token_balances.into();

    let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> =
        transaction.clone().meta.unwrap().pre_token_balances.into();

    let mut changes_by_owner: HashMap<String, HashMap<String, BalanceChange>> = HashMap::new();

    let mut changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChange>> =
        HashMap::new();

    for balance in post_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;

        let owner_address = owner.unwrap();

        // print!("\n amount post: {:#?}", amount);

        let index_usize = balance.account_index.to_usize().unwrap();

        let pub_key_token_address = account_keys[index_usize].clone();

        let owner_entry = changes_by_owner.entry(owner_address.clone());
        let token_entry = owner_entry.or_default().entry(mint.clone());
        let decimals = balance.ui_token_amount.decimals;

        let token_account_address_entry =
            changes_by_token_account_address.entry(pub_key_token_address);

        let token_entry_token_account_address =
            token_account_address_entry.or_default().entry(mint.clone());

        let amount_bf = BigFloat::from_str(&amount).unwrap();

        *token_entry.or_default() = BalanceChange {
            balance_pre: BigFloat::from_f64(0.0),
            balance_pre_usd: None,
            balance_post_usd: None,
            balance_post: amount_bf,
            difference: amount_bf,
            value_transferred: amount_bf,
            difference_usd: None,
            value_transferred_usd: None,
            decimals: decimals,
            mint: mint.clone(),
            owner: owner_address.clone(),
            fees: None,
        };

        *token_entry_token_account_address.or_default() = BalanceChange {
            balance_pre: BigFloat::from_f64(0.0),
            decimals,
            balance_pre_usd: None,
            balance_post_usd: None,
            balance_post: amount_bf,
            difference: amount_bf,
            value_transferred: amount_bf,
            value_transferred_usd: None,
            difference_usd: None,
            mint: mint.clone(),
            owner: owner_address.clone(),
            fees: None,
        };
    }

    for balance in pre_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;

        // print!("\n amount pre: {:#?}", amount);

        let owner_address = owner.unwrap();

        let _owner_address_c = owner_address.clone();

        let index_usize = balance.account_index.to_usize().unwrap();
        let pub_key_token_address = account_keys[index_usize].clone();

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

    for (index, pubkey) in account_keys.iter().enumerate() {
        // let pubkey = account_key["pubkey"].as_str().unwrap();

        let pre = BigFloat::from_u64(pre_balances[index]);
        let post = BigFloat::from_u64(post_balances[index]);

        let item = BalanceChange {
            balance_pre: pre,
            balance_pre_usd: None,
            balance_post_usd: None,
            difference_usd: None,
            decimals: 9,
            balance_post: post,
            difference: post - pre,
            value_transferred: post - pre,
            value_transferred_usd: None,
            mint: "sol".to_string(),
            owner: pubkey.to_string(),
            fees: None,
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

    let token_amount_pool = match token_changes_pool.get(token_address) {
        Some(x) => x.balance_post,
        None => BigFloat::from_i64(0),
    };

    let token_amount_pool_pre = match token_changes_pool.get(token_address) {
        Some(x) => x.balance_pre,
        None => BigFloat::from_i64(0),
    };

    let mut amount_diff_ubo = match token_changes_ubo.get(token_address) {
        Some(x) => x.difference,
        None => BigFloat::from_i64(0),
    };

    let amount_diff_pool = match token_changes_pool.get(token_address) {
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
    }

    let _token_perc_ubo = parse_token_amount_rounded(token_amount_ubo, token_amount_pool);

    let amount_total_pool_bf = token_amount_pool
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18 - (token_decimals as i64))));

    let amount_total_pool_bf_pre = token_amount_pool_pre
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
        // amount_total_pool: token_amount_pool,
        // amount_total_pool_pre: token_amount_pool_pre,
        // amount_total_ubo: token_amount_ubo,
        // amount_diff_pool: amount_diff_pool,
        // amount_diff_ubo: amount_diff_ubo,
        token_address: token_address.to_string(),
        amount_total_pool_18: token_amount_pool_18,
        amount_total_ubo_18: token_amount_ubo_18,
        amount_diff_pool_18: amount_diff_pool_18,
        amount_diff_ubo_18: amount_diff_ubo_18,
        amount_total_pool_pre_18: amount_total_pool_bf_pre,
    };
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
                decimals: existing.decimals,
                balance_pre_usd: None,
                balance_post_usd: None,
                difference_usd: None,

                balance_post: new_balance_post,
                difference: new_diff_post,
                value_transferred: new_diff_post,
                mint: existing.mint.clone(),
                owner: existing.owner.clone(),
                value_transferred_usd: None,
                fees: None,
            };

            map1.insert(item, new_change);
        } else {
            map1.insert(item, change);
        }
    }

    return map1;

    // map1.into_iter().chain(map2).collect()
}

fn parse_token_amount_rounded(token_amount: BigFloat, token_amount_pool: BigFloat) -> BigFloat {
    let ubo_token_a_perc_f = token_amount / token_amount_pool * BigFloat::from_f64(100.0);

    // let parse_token_amount_rounded = ubo_token_a_perc_f;

    return ubo_token_a_perc_f;
}

pub fn get_price(
    token_new_price_in_token_quote_18: BigFloat,
    token_trade_price_in_token_quote_18: BigFloat,
    token_quote_address: &String,
    sol_price_db: &String,
    // decimals_correct: i64,
    quote_decimal: u64,
    base_decimal: u64,
) -> Result<TokenPriceResult, Error> {
    let stable_coin_ref = token_quote_address == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    // let price_usd_token_base_bf = BigFloat::from_str(price_usd_token_base).unwrap();

    let token_quote_price_18 = if stable_coin_ref {
        BigFloat::from_i16(1) * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)))
    } else {
        BigFloat::from_str(sol_price_db).unwrap()
    };

    let _decimals_corrected = 18 - base_decimal as i64 - quote_decimal as i64;

    let token_new_price_18 =
    // if stable_coin_ref 
    {
        // let token_price_base = token_quote_price_18 / token_new_price_in_token_quote_18;
        let token_price_base = token_quote_price_18 * token_new_price_in_token_quote_18;
        // let token_price_base3 = token_new_price_in_token_quote_18 / token_quote_price_18;

        let token_price_18 =
            token_price_base * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

        //     * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-decimals_corrected)));

        token_price_18
    };

    // else {
    //     let token_price_base = token_new_price_in_token_quote_18 * token_quote_price_18;

    //     let token_price_18 =
    //         token_price_base * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    //     token_price_18
    // };

    let token_trade_price_18 = {
        // let token_price_base = token_quote_price_18 / token_new_price_in_token_quote_18;
        let token_price_base = token_quote_price_18 * token_trade_price_in_token_quote_18;
        // let token_price_base3 = token_new_price_in_token_quote_18 / token_quote_price_18;

        let token_price_18 =
            token_price_base * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

        //     * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-decimals_corrected)));

        token_price_18
    };

    // let token_trade_price_18 = if stable_coin_ref {
    //     let token_price_base = token_quote_price_18 / token_trade_price_in_token_quote_18;

    //     let token_price_18 = token_price_base
    //         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-decimals_corrected)));

    //     token_price_18
    // } else {
    //     let token_price_base = token_trade_price_in_token_quote_18 * token_quote_price_18;

    //     let token_price_18 =
    //         token_price_base * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    //     token_price_18
    // };

    let token_new_price_in_token_quote_fixed = token_new_price_in_token_quote_18
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let token_trade_price_in_token_quote_fixed = token_trade_price_in_token_quote_18
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));
    // 111288733531385
    // 22511500200737540

    let token_new_price_fixed =
        token_new_price_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let token_trade_price_fixed =
        token_trade_price_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let result = TokenPriceResult {
        token_quote_price_18,
        token_new_price_18,
        token_new_price_fixed,
        token_new_price_in_token_quote_18,
        token_new_price_in_token_quote_fixed,

        token_trade_price_18,
        token_trade_price_fixed,
        token_trade_price_in_token_quote_18,
        token_trade_price_in_token_quote_fixed,
    };

    // base_mint: So11111111111111111111111111111111111111112,
    // quote_mint: 6pxT5UmTumQXBknjwSzLePRwBA5k8VGs68LiZwncC2mB,

    // base_mint: CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J,
    // quote_mint: So11111111111111111111111111111111111111112,

    Ok(result)
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

pub fn parse_token_amounts_new(
    token_amounts: &TokenAmountsSwap,
    token_prices: &TokenPriceResult,
    // token_a_decimals: u64,
    // token_b_decimals: u64,
) -> SwapTokenAmountsPriced {
    let token_usd_a = multiply_token_amounts_to_usd(
        &token_amounts.token_amounts_quote,
        token_prices.token_new_price_in_token_quote_18,
    );

    let token_usd_b = multiply_token_amounts_to_usd(
        &token_amounts.token_amounts_base,
        token_prices.token_new_price_18,
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

fn multiply_amounts(token_price_usd_18: BigFloat, amount_18: BigFloat) -> UsdMultiplierResult {
    let token_price_usd_fixed =
        token_price_usd_18 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let amount_fixed = amount_18;

    let total_amount_usd = token_price_usd_fixed * amount_fixed;

    let total_amount_usd_rounded = total_amount_usd.round(0, RoundingMode::ToOdd);
    // .unwrap();

    // let rounded = total_amount_usd_rounded.round();
    // test

    return UsdMultiplierResult {
        amount_18: total_amount_usd,
        amount_18_rounded: total_amount_usd_rounded,
    };
}

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

        amount_total_pool: token_amounts.amount_total_pool_18,
        amount_diff_pool: token_amounts.amount_diff_pool_18,
        amount_total_ubo: token_amounts.amount_total_ubo_18,
        amount_diff_ubo: token_amounts.amount_diff_ubo_18,
    };

    return tpo_values_a;
}

pub fn get_rounded_amount(amount: BigFloat, decimals: usize) -> String {
    let amount_rounded = amount.round(decimals, RoundingMode::ToOdd);

    let amount = amount_rounded.to_f64();

    // if amount.is_none() {
    //     return amount_rounded.to_string();
    // }

    let value = amount.to_string();

    return value;
}

pub enum BalanceHolder {
    Owner,
    TokenAccount,
}

pub fn parse_balance_changes_new(
    transaction: &EncodedTransactionWithStatusMeta,
    account_keys: &Vec<String>,
    balance_holder: BalanceHolder,
    fees: HashMap<String, TransactionFees>,
    ubo: &str,
) -> HashMap<std::string::String, HashMap<std::string::String, BalanceChange>> {
    // println!("Fee {:#?} to substract from: {:#?}", fees, ubo);

    let post_balances = transaction.clone().meta.unwrap().post_balances;
    let pre_balances = transaction.clone().meta.unwrap().pre_balances;

    let post_token_balances: Option<Vec<UiTransactionTokenBalance>> =
        transaction.clone().meta.unwrap().post_token_balances.into();

    let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> =
        transaction.clone().meta.unwrap().pre_token_balances.into();

    let mut changes_by_owner: HashMap<String, HashMap<String, BalanceChange>> = HashMap::new();

    // let mut changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChange>> =
    //     HashMap::new();

    for balance in post_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;
        let decimals = balance.ui_token_amount.decimals;
        let owner_address = owner.unwrap();

        // print!("\n amount post: {:#?}", amount);

        let index_usize = balance.account_index.to_usize().unwrap();

        let pub_key_token_address = account_keys[index_usize].clone();

        // let owner_entry = changes_by_owner.entry(owner_address.clone());
        // let token_entry = owner_entry.or_default().entry(mint.clone());

        let owner_entry = match balance_holder {
            BalanceHolder::Owner => changes_by_owner.entry(owner_address.clone()),
            BalanceHolder::TokenAccount => changes_by_owner.entry(pub_key_token_address.clone()),
        };

        let token_entry = owner_entry.or_default().entry(mint.clone());

        // changes_by_owner.entry(owner_address.clone());

        // let token_account_address_entry =
        //     changes_by_token_account_address.entry(pub_key_token_address);

        // let token_entry_token_account_address =
        //     token_account_address_entry.or_default().entry(mint.clone());

        let amount_bf = BigFloat::from_str(&amount).unwrap();

        *token_entry.or_default() = BalanceChange {
            balance_pre: BigFloat::from_f64(0.0),
            balance_pre_usd: None,
            balance_post_usd: None,
            difference_usd: None,
            balance_post: amount_bf,
            value_transferred: amount_bf,
            difference: amount_bf,
            decimals: decimals,
            mint: mint.clone(),
            owner: owner_address.clone(),
            value_transferred_usd: None,
            fees: None,
        };

        // *token_entry_token_account_address.or_default() = BalanceChange {
        //     balance_pre: BigFloat::from_f64(0.0),
        //     balance_post: amount_bf,
        //     difference: amount_bf,
        //     mint: mint.clone(),
        //     owner: owner_address.clone(),
        // };
    }

    for balance in pre_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;

        // print!("\n amount pre: {:#?}", amount);

        let owner_address = owner.unwrap();

        let _owner_address_c = owner_address.clone();

        let index_usize = balance.account_index.to_usize().unwrap();
        let pub_key_token_address = account_keys[index_usize].clone();

        let owner_entry = match balance_holder {
            BalanceHolder::Owner => changes_by_owner.entry(owner_address.clone()),
            BalanceHolder::TokenAccount => changes_by_owner.entry(pub_key_token_address.clone()),
        };

        let token_entry = owner_entry.or_default().entry(mint.clone());

        // let token_account_address_entry =
        //     changes_by_token_account_address.entry(pub_key_token_address.to_string());

        // let token_entry_token_account_address =
        //     token_account_address_entry.or_default().entry(mint.clone());

        let amount_bf = BigFloat::from_str(&amount).unwrap();

        let existing_entry = token_entry.or_default();

        // let existing_entry_token_account = token_entry_token_account_address.or_default();

        existing_entry.balance_pre = amount_bf;
        existing_entry.difference = existing_entry.balance_post - amount_bf;

        // existing_entry_token_account.balance_pre = amount_bf;
        // existing_entry_token_account.difference =
        //     existing_entry_token_account.balance_post - amount_bf;
    }

    for (index, pubkey) in account_keys.iter().enumerate() {
        // let pubkey = account_key["pubkey"].as_str().unwrap();

        let pre = BigFloat::from_u64(pre_balances[index]);
        let post = BigFloat::from_u64(post_balances[index]);

        // let post_test = post
        // let fee_item = BigFloat::from_u64(fee); //.mul(&BigFloat::from_u64(10).pow(&BigFloat::from_i8(9)));
        // .pow(&BigFloat::from_i8(9));

        // println!(
        //     "fee = {} Fee item: {:#?}",
        //     fee,
        //     fee_item.to_f64().to_string()
        // );

        let key = pubkey.clone() + "##" + "sol";

        let fee_for_address = fees.get(key.as_str());

        let mut fee: Option<Vec<TransactionFees>> = None;

        let mut value_change = post - pre;

        if (fee_for_address.is_some()) {
            let fee_amount = fee_for_address.unwrap();

            // let fee = fee_for_address.unwrap().fee;
            // let fee_item = BigFloat::from_u64(fee);
            // let post = post - fee_item;

            if (value_change.is_positive()) {
                value_change = value_change - fee_amount.amount_bf;
            } else {
                value_change = value_change + fee_amount.amount_bf;
            }
            fee = Some(vec![fee_amount.clone()]);
        }

        // println!("Fee found: {:#?}", fee);
        let item = BalanceChange {
            balance_pre: pre,
            balance_pre_usd: None,
            difference_usd: None,
            balance_post_usd: None,
            balance_post: post,
            difference: post - pre,
            value_transferred: value_change,
            value_transferred_usd: None,
            fees: fee, //Some(fee_item),
            decimals: 9,
            mint: "sol".to_string(),
            owner: pubkey.to_string(),
        };

        // let owner_entry = changes_by_owner.entry(pubkey.to_string());

        let owner_entry = changes_by_owner.entry(pubkey.to_string());

        // match balance_holder {
        //     BalanceHolder::Owner => changes_by_owner.entry(pubkey.to_string()),
        //     BalanceHolder::TokenAddress => changes_by_owner.entry(pubkey.to_string()),
        // };

        let token_entry = owner_entry.or_default().entry("sol".to_string());
        // let token_account_address_entry =
        //     changes_by_token_account_address.entry(pubkey.to_string());

        // let token_entry_token_account_address = token_account_address_entry
        //     .or_default()
        //     .entry("sol".to_string());

        // if (fee.is_some()) {
        //     println!("Fee found: {:#?}", item);
        // }

        *token_entry.or_default() = item.clone();

        // *token_entry_token_account_address.or_default() = item.clone();
    }

    return changes_by_owner;
}

pub fn calc_token_usd_total(
    value: BigFloat,
    token_price: Option<&String>,
    decimals: u8,
) -> Option<BigFloat> {
    // let token_price_o = prices.get(&key.to_string());

    let balance_pre_priced = match token_price {
        Some(x) => {
            let price = BigFloat::from_str(x).unwrap();

            Some(
                value
                    .mul(&price)
                    .mul(&BigFloat::from_i64(10).pow(&BigFloat::from_i8(-(decimals as i8 + 18)))), // .round(0, num_bigfloat::RoundingMode::ToZero),
            )
        }
        None => None,
    };

    balance_pre_priced
}
