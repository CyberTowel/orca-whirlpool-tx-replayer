use chrono::prelude::*;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

use crate::{
    pool_state::get_pool_meta,
    token_parser::{get_price, get_token_amounts, parse_token_amounts_new, PriceItem},
    transaction,
};

pub fn testing_nested() {
    println!("Testing nested");
}

pub fn init(signature: String, pool_id: String, rpc_connection: &RpcClient) {
    let sol_price_db = "1400000000000000000000".to_string();

    let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(1),
    };

    let transaction_req = rpc_connection
        .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

    if transaction_req.is_err() {
        println!(
            "Error in transaction: {:#?} {:#?}",
            signature, transaction_req
        );
        return;
    }

    let pool_meta = get_pool_meta(&pool_id, rpc_connection);

    let transaction = transaction_req.unwrap();

    let transaction_parsed = transaction::Transaction::new(&transaction);

    let token_amounts = get_token_amounts(
        &transaction,
        &transaction_parsed.account_keys,
        &transaction_parsed.ubo,
        &pool_meta.quote_mint.to_string(),
        &pool_meta.base_mint.to_string(),
        &pool_meta.quote_vault.to_string(),
        &pool_meta.base_vault.to_string(),
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
        // decimals_correct, // pool_state,
    );

    let token_prices = get_price(
        token_amounts.token_new_price_in_token_quote_18,
        token_amounts.token_trade_price_in_token_quote_18,
        &pool_meta.quote_mint.to_string(),
        &sol_price_db.to_string(),
        // decimals_correct,
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
    )
    .unwrap();

    let swap_token_amounts_priced = parse_token_amounts_new(
        &token_amounts,
        &token_prices,
        // price.token_price_usd_18,
        // price.token_ref_price_usd_18,
        // pool_state.quote_decimal,
        // pool_state.base_decimal,
    );

    let datetime = DateTime::from_timestamp(transaction_parsed.block_timestamp, 0)
        .unwrap()
        .to_rfc3339();

    let item_to_save = PriceItem {
        signature: signature.to_string(),
        token_quote_address: pool_meta.quote_mint.to_string(),
        token_base_address: pool_meta.base_mint.to_string(),

        token_new_price_18: token_prices.token_new_price_18,
        token_new_price_fixed: token_prices.token_new_price_fixed,
        token_new_price_in_token_quote_18: token_prices.token_new_price_in_token_quote_18,
        token_new_price_in_token_quote_fixed: token_prices.token_new_price_in_token_quote_fixed,

        token_trade_price_18: token_prices.token_trade_price_18,
        token_trade_price_fixed: token_prices.token_trade_price_fixed,
        token_trade_price_in_token_quote_18: token_prices.token_trade_price_in_token_quote_18,
        token_trade_price_in_token_quote_fixed: token_prices.token_trade_price_in_token_quote_fixed,

        datetime: datetime,
        signer: transaction_parsed.signer.to_string(),
        ubo: transaction_parsed.ubo.to_string(),
        pool_address: pool_id.to_string(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18,
        block_number: transaction_parsed.block_number.to_string(),
    };

    // println!("item_to_save: {:#?}", item_to_save);
    println!(
        "
===== price update for token {:#?} ============
token_new_price_18: {:#?}
token_new_price_fixed: {:#?}
token_new_price_in_token_quote_18: {:#?}
token_new_price_in_token_quote_fixed: {:#?}
===============================================
",
        item_to_save.token_base_address.to_string(),
        item_to_save.token_new_price_18.to_f64().to_string(),
        item_to_save.token_new_price_fixed.to_f64().to_string(),
        item_to_save
            .token_new_price_in_token_quote_18
            .to_f64()
            .to_string(),
        item_to_save
            .token_new_price_in_token_quote_fixed
            .to_f64()
            .to_string(),
    )

    // println!("Token amounts: {:#?}", swap_token_amounts_priced);
}
