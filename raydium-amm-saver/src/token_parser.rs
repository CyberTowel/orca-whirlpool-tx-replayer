use num::ToPrimitive;
use num_bigfloat::{BigFloat, RoundingMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionTokenBalance,
};
use std::{collections::HashMap, str::FromStr};
const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

#[derive(Debug)]
pub enum Error {}

#[derive(Default, Debug, Clone)]
pub struct BalanceChange {
    pub owner: String,
    pub mint: String,
    pub balance_pre: i64,
    pub balance_post: i64,
    pub difference: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmountsSwapPriced {
    pub token_amounts_a: TokenAmountsPriced,
    pub token_amounts_b: TokenAmountsPriced,
    pub usd_total_pool: f64,
    pub price_usd_token_a_formatted: f64,
    pub price_usd_token_b_formatted: f64,
}

pub fn get_token_amounts(
    rpc_transaction: &EncodedConfirmedTransactionWithStatusMeta,
    account_keys: &Vec<Value>,
    ubo: &str,
    token_a_address: &str,
    token_b_address: &str,
    token_a_taa: &str,
    token_b_taa: &str,
) -> TokenAmountsSwap {
    let (token_changes_by_wallet, changes_by_token_account_address) =
        parse_balance_changes(rpc_transaction, account_keys);

    let token_changes_ubo = token_changes_by_wallet.get(ubo).unwrap();
    let token_changes_pool_old = token_changes_by_wallet.get(RAYDIUM_AUTHORITY).unwrap();

    let token_changes_pool_new_a = changes_by_token_account_address.get(token_a_taa).unwrap();
    let token_changes_pool_new_b = changes_by_token_account_address.get(token_b_taa).unwrap();

    let token_changes_pool = merge_hashmap(
        token_changes_pool_new_a.clone(),
        token_changes_pool_new_b.clone(),
    );

    let token_amounts_a =
        parse_token_amounts(token_changes_ubo, &token_changes_pool, token_a_address);

    let token_amounts_b =
        parse_token_amounts(token_changes_ubo, &token_changes_pool, token_b_address);

    let token_b_price_rel = BigFloat::from_i64(token_amounts_a.amount_total_pool)
        / BigFloat::from_i64(token_amounts_b.amount_total_pool);

    let price_token_ref =
        token_b_price_rel * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-6)));

    // println!(
    //     "token_b_price_rel: {:#?}
    //     token_amounts_a: {:#?}",
    //     token_b_price_rel.to_f64().to_string(),
    //     token_amounts_a
    // );

    let token_amounts_swap = TokenAmountsSwap {
        token_a_address: token_a_address.to_string(),
        token_b_address: token_b_address.to_string(),
        token_amounts_a: token_amounts_a,
        token_amounts_b: token_amounts_b,
        price_usd_token_b: price_token_ref.to_f64(),
        price_token_b_rel: price_token_ref.to_f64(),
        price_usd_token_b_formatted: price_token_ref.to_f64().to_string(),
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

        let amount_64 = amount.parse::<i64>().unwrap();

        *token_entry.or_default() = BalanceChange {
            balance_pre: 0,
            balance_post: amount_64,
            difference: 0,
            mint: mint.clone(),
            owner: owner_address.clone(),
        };

        *token_entry_token_account_address.or_default() = BalanceChange {
            balance_pre: 0,
            balance_post: amount_64,
            difference: 0,
            mint: mint.clone(),
            owner: owner_address.clone(),
        };
    }

    for balance in pre_token_balances.unwrap() {
        let owner: Option<String> = balance.owner.clone().into();
        let mint = balance.mint.clone();
        let amount = balance.ui_token_amount.amount;
        let owner_address = owner.unwrap();

        let index_usize = balance.account_index.to_usize().unwrap();
        let pub_key_token_address = account_keys[index_usize]["pubkey"].as_str().unwrap();

        let owner_entry = changes_by_owner.entry(owner_address);
        let token_entry = owner_entry.or_default().entry(mint.clone());

        let token_account_address_entry =
            changes_by_token_account_address.entry(pub_key_token_address.to_string());

        let token_entry_token_account_address =
            token_account_address_entry.or_default().entry(mint.clone());

        let amount_64 = amount.parse::<i64>().unwrap();

        let existing_entry = token_entry.or_default();

        let existing_entry_token_account = token_entry_token_account_address.or_default();

        existing_entry.balance_pre = amount_64;
        existing_entry.difference = existing_entry.balance_post - amount_64;

        existing_entry_token_account.balance_pre = amount_64;
        existing_entry_token_account.difference =
            existing_entry_token_account.balance_post - amount_64;
    }

    for (index, account_key) in account_keys.iter().enumerate() {
        let pubkey = account_key["pubkey"].as_str().unwrap();

        let pre = pre_balances[index].to_i64().unwrap();
        let post = post_balances[index].to_i64().unwrap();

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

pub fn get_price(token_b_price_rel: f64, token_a_price: &String) -> Result<(f64, f64), Error> {
    let usd_price_token_b =
        BigFloat::from_f64(token_b_price_rel) * BigFloat::from_str(token_a_price).unwrap();

    let usd_price_token_dec =
        usd_price_token_b * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let price_token_usd = usd_price_token_dec.round(32, RoundingMode::ToOdd).to_f64();

    let price_usd_18 = BigFloat::from_f64(price_token_usd)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let price_usd_18_rounded = price_usd_18.round(0, RoundingMode::ToOdd).to_f64();

    Ok((price_usd_18_rounded, token_b_price_rel))
}

fn parse_token_amounts(
    token_changes_ubo: &HashMap<String, BalanceChange>,
    token_changes_pool: &HashMap<String, BalanceChange>,
    token_address: &str,
) -> TokenAmounts {
    let mut token_amount_ubo = match token_changes_ubo.get(token_address) {
        Some(x) => x.balance_post,
        None => 0,
    };

    let mut token_amount_pool = match token_changes_pool.get(token_address) {
        Some(x) => x.balance_post,
        None => 0,
    };

    let mut amount_diff_ubo = match token_changes_ubo.get(token_address) {
        Some(x) => x.difference,
        None => 0,
    };

    let mut amount_diff_pool = match token_changes_pool.get(token_address) {
        Some(x) => x.difference,
        None => 0,
    };

    if token_address == "So11111111111111111111111111111111111111112" {
        let native_sol_amount = token_changes_ubo.get("sol");
        let amount = match native_sol_amount {
            Some(x) => x.balance_post,
            None => 0,
        };

        token_amount_ubo += amount;

        let amount_diff = match native_sol_amount {
            Some(x) => x.difference,
            None => 0,
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

    return TokenAmounts {
        amount_total_pool: token_amount_pool,
        amount_total_ubo: token_amount_ubo,
        amount_diff_pool: amount_diff_pool,
        amount_diff_ubo: amount_diff_ubo,
        perc_ubo: token_perc_ubo,
        perc_ubo_formatted: token_perc_ubo.to_string(),
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

fn parse_token_amount_rounded(token_amount: i64, token_amount_pool: i64) -> f64 {
    let ubo_token_a_perc_f = BigFloat::from_i64(token_amount)
        / BigFloat::from_i64(token_amount_pool)
        * BigFloat::from_i64(100);

    let parse_token_amount_rounded = ubo_token_a_perc_f.round(32, RoundingMode::ToOdd).to_f64();

    return parse_token_amount_rounded;
}

#[derive(Debug)]
pub struct TokenAmountsSwap {
    pub token_amounts_a: TokenAmounts,
    pub token_amounts_b: TokenAmounts,
    pub price_token_b_rel: f64,
    pub price_usd_token_b: f64,
    pub price_usd_token_b_formatted: String,
    pub token_a_address: String,
    pub token_b_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenAmounts {
    pub amount_total_pool: i64,
    pub amount_diff_pool: i64,
    pub amount_total_ubo: i64,
    pub amount_diff_ubo: i64,
    pub perc_ubo: f64,
    pub perc_ubo_formatted: String,
}

// pub fn parse_pricing_to_token_amounts(
//     token_amounts: &TokenAmounts,
//     token_a_price_usd: f64,
//     token_b_price_usd: f64,
// ) {
//     let token_a_price_usd_dec = BigFloat::from_f64(token_a_price_usd)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

//     let token_amount_pool_a_decimals = BigFloat::from_i64(token_amounts.amount_total_pool_a)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-9)));

//     let token_amount_pool_b_decimals = BigFloat::from_i64(token_amounts.amount_total_pool_b)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-3)));

//     let usd_total_pool_a = token_amount_pool_a_decimals * token_a_price_usd_dec;
//     let usd_total_pool_b = token_amount_pool_b_decimals * BigFloat::from_f64(token_b_price_usd);
//     let usd_total_pool = usd_total_pool_a + usd_total_pool_b;

//     // ubo amounts
//     let token_amount_ubo_a_decimals = BigFloat::from_i64(token_amounts.amount_total_ubo_a)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-9)));

//     let token_amount_ubo_b_decimals = BigFloat::from_i64(token_amounts.amount_total_ubo_b)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-3)));

//     let amount_diff_ubo_a_decimals = BigFloat::from_i64(token_amounts.amount_diff_ubo_a)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-9)));

//     let amount_diff_ubo_b_decimals = BigFloat::from_i64(token_amounts.amount_diff_ubo_b)
//         * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-3)));

//     let usd_total_ubo_a = token_amount_ubo_a_decimals * token_a_price_usd_dec;
//     let usd_total_ubo_b = token_amount_ubo_b_decimals * BigFloat::from_f64(token_b_price_usd);

//     let usd_diff_ubo_a = amount_diff_ubo_a_decimals * token_a_price_usd_dec;
//     let usd_diff_ubo_b = amount_diff_ubo_b_decimals * BigFloat::from_f64(token_b_price_usd);

//     // rounding
//     let token_a_price_usd_dec_rounded = token_a_price_usd_dec
//         .round(32, RoundingMode::ToOdd)
//         .to_f64();

//     let token_amount_pool_a_decimals_rounded = token_amount_pool_a_decimals
//         .round(32, RoundingMode::ToOdd)
//         .to_f64();

//     let token_amount_pool_b_decimals_rounded = token_amount_pool_b_decimals
//         .round(32, RoundingMode::ToOdd)
//         .to_f64();

//     let usd_total_pool_a_rounded = usd_total_pool_a.round(32, RoundingMode::ToOdd).to_f64();

//     let usd_total_pool_b_rounded = usd_total_pool_b.round(32, RoundingMode::ToOdd).to_f64();

//     let usd_total_pool_rounded = usd_total_pool.round(32, RoundingMode::ToOdd).to_f64();

//     let usd_total_ubo_a_rounded = usd_total_ubo_a.round(32, RoundingMode::ToOdd).to_f64();
//     let usd_total_ubo_b_rounded = usd_total_ubo_b.round(32, RoundingMode::ToOdd).to_f64();

//     let usd_diff_ubo_a_rounded = usd_diff_ubo_a.round(32, RoundingMode::ToOdd).to_f64();
//     let usd_diff_ubo_b_rounded = usd_diff_ubo_b.round(32, RoundingMode::ToOdd).to_f64();

//     println!(
//         "
// Price token a: {:#?}
// price token b: {:#?}

// ---- pool values ----
// total tokens a in pool :        {:#?}
// total tokens b in pool :        {:#?}
// total value token a pool (USD): {:#?}
// total value token b pool (USD): {:#?}
// total value pool (USD)          {:#?}

// ---- ubo values ----
// total value token a UBO (USD):      {:#?}
// total value token b UBO (USD):      {:#?}
// Transaction value token a (USD):    {:#?}
// Transaction value token b (USD):    {:#?}",
//         token_a_price_usd_dec_rounded.to_string(),
//         token_b_price_usd.to_string(),
//         token_amount_pool_a_decimals_rounded.to_string(),
//         token_amount_pool_b_decimals_rounded.to_string(),
//         usd_total_pool_a_rounded.to_string(),
//         usd_total_pool_b_rounded.to_string(),
//         usd_total_pool_rounded.to_string(),
//         usd_total_ubo_a_rounded.to_string(),
//         usd_total_ubo_b_rounded.to_string(),
//         usd_diff_ubo_a_rounded.to_string(),
//         usd_diff_ubo_b_rounded.to_string(),
//     );
// }

pub fn parse_combined(
    token_amounts: &TokenAmountsSwap,
    price_usd_token_a: f64,
    price_usd_token_b: f64,
) -> TokenAmountsSwapPriced {
    let token_prices_a =
        parse_pricing_to_token_amounts_new(&token_amounts.token_amounts_a, price_usd_token_a, 9);

    let token_prices_b =
        parse_pricing_to_token_amounts_new(&token_amounts.token_amounts_b, price_usd_token_b, 3);
    let usd_total_pool = token_prices_a.usd_total_pool + token_prices_b.usd_total_pool;
    let usd_total_pool_rounded = BigFloat::from_f64(usd_total_pool)
        .round(32, RoundingMode::ToOdd)
        .to_f64();

    // formattted values

    let price_usd_dec = BigFloat::from_f64(price_usd_token_a)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let price_usd_token_a_rounded = price_usd_dec.round(32, RoundingMode::ToOdd).to_f64();

    let price_usd_dec_token_b = BigFloat::from_f64(price_usd_token_b)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let price_usd_token_b_rounded = price_usd_dec_token_b
        .round(32, RoundingMode::ToOdd)
        .to_f64();

    let return_value = TokenAmountsSwapPriced {
        token_amounts_a: token_prices_a,
        token_amounts_b: token_prices_b,
        usd_total_pool: usd_total_pool_rounded,
        price_usd_token_a_formatted: price_usd_token_a_rounded,
        price_usd_token_b_formatted: price_usd_token_b_rounded,
    };

    return return_value;
}

pub fn parse_pricing_to_token_amounts_new(
    token_amounts: &TokenAmounts,
    token_price_usd: f64,
    token_decimals: i64,
) -> TokenAmountsPriced {
    let token_a_price_usd_dec = BigFloat::from_f64(token_price_usd)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-18)));

    let token_amount_pool_a_decimals = BigFloat::from_i64(token_amounts.amount_total_pool)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-token_decimals)));

    let usd_total_pool = token_amount_pool_a_decimals * token_a_price_usd_dec;

    // // ubo amounts
    let token_amount_ubo_a_decimals = BigFloat::from_i64(token_amounts.amount_total_ubo)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-token_decimals)));

    let amount_diff_ubo_a_decimals = BigFloat::from_i64(token_amounts.amount_diff_ubo)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-token_decimals)));

    let amount_diff_pool_a_decimals = BigFloat::from_i64(token_amounts.amount_diff_pool)
        * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-token_decimals)));

    let usd_total_ubo_a = token_amount_ubo_a_decimals * token_a_price_usd_dec;

    let usd_diff_ubo_a = amount_diff_ubo_a_decimals * token_a_price_usd_dec;
    let usd_diff_pool = amount_diff_pool_a_decimals * token_a_price_usd_dec;

    // let usd_diff_pool = amount_diff_pool

    // // rounding
    let token_price_usd_dec_rounded = token_a_price_usd_dec
        .round(32, RoundingMode::ToOdd)
        .to_f64();

    let usd_total_pool_a_rounded = usd_total_pool.round(32, RoundingMode::ToOdd).to_f64();
    let usd_total_ubo_a_rounded = usd_total_ubo_a.round(32, RoundingMode::ToOdd).to_f64();
    let usd_diff_ubo_a_rounded = usd_diff_ubo_a.round(32, RoundingMode::ToOdd).to_f64();
    let usd_diff_pool_rounded = usd_diff_pool.round(32, RoundingMode::ToOdd).to_f64();

    // 18 decimals
    let usd_total_pool_18_bf =
        usd_total_pool * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let usd_total_pool_18 = usd_total_pool_18_bf
        .round(32, RoundingMode::ToOdd)
        .to_i128()
        .unwrap();

    let usd_total_ubo_a_bf =
        usd_total_ubo_a * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let usd_total_ubo_18 = usd_total_ubo_a_bf
        .round(32, RoundingMode::ToOdd)
        .to_i128()
        .unwrap();

    let usd_diff_ubo_a_bf =
        usd_diff_ubo_a * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let usd_diff_ubo_18 = usd_diff_ubo_a_bf
        .round(32, RoundingMode::ToOdd)
        .to_i128()
        .unwrap();

    let usd_diff_pool_bf =
        usd_diff_pool * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18)));

    let usd_diff_pool_18 = usd_diff_pool_bf
        .round(32, RoundingMode::ToOdd)
        .to_i128()
        .unwrap();

    let token_amounts_priced = TokenAmountsPriced {
        usd_total_pool: usd_total_pool_a_rounded,
        usd_total_ubo: usd_total_ubo_a_rounded,
        usd_diff_ubo: usd_diff_ubo_a_rounded,
        usd_diff_pool: usd_diff_pool_rounded,
        token_price_usd: token_price_usd_dec_rounded,
        usd_total_pool_18: usd_total_pool_18,
        usd_total_ubo_18: usd_total_ubo_18,
        usd_diff_ubo_18: usd_diff_ubo_18,
        usd_diff_pool_18: usd_diff_pool_18,
    };

    return token_amounts_priced;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmountsPriced {
    pub usd_total_pool: f64,
    pub usd_total_ubo: f64,
    pub usd_diff_ubo: f64,
    pub usd_diff_pool: f64,
    pub token_price_usd: f64,

    pub usd_total_pool_18: i128,
    pub usd_total_ubo_18: i128,
    pub usd_diff_ubo_18: i128,
    pub usd_diff_pool_18: i128,
}

pub struct TokenAmountsPriced18 {}
