use crate::pool_state::PoolMeta;
use crate::token_db::{PriceItem, TokenDbClient};
use crate::token_parser::PoolVars;
use crate::token_parser::{
    get_price, get_token_amounts, parse_token_amounts_new, parse_token_price_oracle_values,
};
use crate::transaction::Transaction;
use chrono::prelude::*;
use rust_decimal::prelude::*;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

pub fn testing() {
    println!("Testing");
}

pub fn parser_transaction(
    signature: &String,
    rpc_connection: &RpcClient,
    db_client: &TokenDbClient,
    pool_state: &PoolMeta,
    poolvars: &PoolVars,
) -> (String, String, String) {
    println!("Parsing transaction: {:#?}", signature);
    let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(1),
    };

    let start = std::time::Instant::now();
    let transaction_req = rpc_connection
        .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

    if transaction_req.is_err() {
        println!("Error in transaction: {:#?}", signature);
        println!("Error in transaction: {:#?}", transaction_req);
        return (
            signature.to_string(),
            "error".to_string(),
            "error getting rpc data".to_string(),
        );
    }

    let duration = start.elapsed();
    println!(
        "Time elapsed in get_transaction_with_config() is: {:?}",
        duration
    );

    let transaction = transaction_req.unwrap();

    let transaction_parsed = Transaction::new(&transaction);

    let token_amounts = get_token_amounts(
        &transaction,
        &transaction_parsed.account_keys,
        &transaction_parsed.ubo,
        &pool_state.quote_mint.to_string(),
        &pool_state.base_mint.to_string(),
        &pool_state.quote_vault.to_string(),
        &pool_state.base_vault.to_string(),
        pool_state.quote_decimal,
        pool_state.base_decimal,
        // decimals_correct, // pool_state,
    );

    let sol_price_db = db_client
        .get_usd_price_sol(transaction_parsed.datetime)
        .unwrap();

    let token_prices = get_price(
        token_amounts.token_new_price_in_token_quote_18,
        token_amounts.token_trade_price_in_token_quote_18,
        &pool_state.quote_mint.to_string(),
        &sol_price_db.to_string(),
        // decimals_correct,
        pool_state.quote_decimal,
        pool_state.base_decimal,
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
        token_quote_address: pool_state.quote_mint.to_string(),
        token_base_address: pool_state.base_mint.to_string(),

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
        pool_address: poolvars.pool_id.to_string(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18,
        block_number: transaction_parsed.block_number.to_string(),
    };

    let price_item_c = item_to_save.clone();

    let reponse = db_client.save_token_values(item_to_save);

    let tpo_values_a = parse_token_price_oracle_values(
        transaction_parsed.ubo.to_string(),
        transaction_parsed.signer.to_string(),
        poolvars.pool_id.to_string(),
        pool_state.base_mint.to_string(),
        &token_amounts.token_amounts_quote,
        &swap_token_amounts_priced.token_amounts_priced_a,
        signature,
    );

    let tpo_values_b = parse_token_price_oracle_values(
        transaction_parsed.ubo.to_string(),
        transaction_parsed.signer.to_string(),
        poolvars.pool_id.to_string(),
        pool_state.quote_mint.to_string(),
        &token_amounts.token_amounts_base,
        &swap_token_amounts_priced.token_amounts_priced_b,
        signature,
    );

    let response_token_usd_a = db_client.insert_token_usd_values(&signature, &tpo_values_a);

    if response_token_usd_a.is_err() {
        println!(
            "Error saving token usd values to db: {:#?}",
            response_token_usd_a
        );
    }

    let response_token_usd_b = db_client.insert_token_usd_values(&signature, &tpo_values_b);

    if response_token_usd_b.is_err() {
        println!(
            "Error saving token usd values to db: {:#?}",
            response_token_usd_b
        );
    }

    if reponse.is_err() {
        println!("Error saving to db: {:#?}", reponse);
    }

    return (
        signature.to_string(),
        price_item_c.datetime,
        "success".to_string(),
    );
}
