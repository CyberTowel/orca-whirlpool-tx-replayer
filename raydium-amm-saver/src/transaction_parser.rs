use crate::pool_state::LiquidityStateLayoutV4;
use crate::token_db::{PriceItem, TokenDbClient};
use crate::token_parser::{
    get_price, get_token_amounts, parse_token_amounts_new, parse_token_price_oracle_values,
};
use crate::token_parser::{PoolVars, TokenPriceOracleValues};
use crate::transaction::Transaction;
use chrono::prelude::*;
use num::traits::sign;
use rust_decimal::prelude::*;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_sdk::vote::error;
use solana_transaction_status::UiTransactionEncoding;

pub fn parser_transaction(
    signature: &String,
    rpc_connection: &RpcClient,
    db_client: &TokenDbClient,
    pool_state: &LiquidityStateLayoutV4,
    poolvars: &PoolVars,
) -> (String, String, String) {
    let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(1),
    };

    // let signature_to_get = solana_sdk::signature::Signature::from_str(&signature).unwrap();

    // let transaction_status_err = rpc_connection
    //     .get_signature_status(&signature_to_get)
    //     .unwrap();

    // if transaction_status_err.is_some() && transaction_status_err.unwrap().is_err() {
    //     println!("Error in transaction: {:#?}", signature);
    //     return;
    // }

    let transaction_req = rpc_connection
        .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

    if transaction_req.is_err() {
        println!("Error in transaction: {:#?}", signature);
        return (
            signature.to_string(),
            "error".to_string(),
            "error getting rpc data".to_string(),
        );
    }

    let transaction = transaction_req.unwrap();

    // let testing_err = &transaction.transaction.meta.as_ref().unwrap().err;
    // if (testing.is_some()) {
    //     println!("Error in testing: {:#?}", testing);
    //     println!("Error in testing_err: {:#?}", testing_err);
    //     return;
    // }

    // return;

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
        // pool_state,
    );

    let sol_price_db = db_client
        .get_usd_price_sol(transaction_parsed.datetime)
        .unwrap();

    let decimals_correct = pool_state.quote_decimal as i64 - pool_state.base_decimal as i64;

    let token_prices = get_price(
        &token_amounts.price_usd_token_b.to_string(),
        &pool_state.quote_mint.to_string(),
        &sol_price_db.to_string(),
        decimals_correct,
    )
    .unwrap();

    // let (price_usd_18, price_token_ref, price_token_ref_18, token_a_price_parsed) = price.unwrap();

    // println!(
    //     "price_token_a_18 {:#?}\n
    //     price_usd_18 {:#?}\n
    //     price_token_ref {:#?}\n
    //     price_token_ref_18 {:#?}",
    //     price_token_a_18.to_string(),
    //     price_usd_18,
    //     price_token_ref,
    //     price_token_ref_18
    // );
    // println!(
    //     "price_usd: {:?}, {:?}, price_token_a: {:?}",
    //     price_token_b_18,
    //     price_token_ref.to_string(),
    //     price_token_a_18.to_string(),
    // );

    //price usd $0.0001842

    // let token_amounts_usd = parse_combined(
    //     &token_amounts,
    //     token_prices.token_price_usd_18,
    //     token_prices.token_ref_price_usd_18,
    //     pool_state.quote_decimal,
    //     pool_state.base_decimal,
    // );

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

    // let token_amounts_usd_c = token_amounts_usd.clone();

    // println!(
    //     "to_string, to_f64  {:#?}, {:#?}",
    //     token_prices.token_price_usd_18.to_string(),
    //     token_prices.token_price_usd_18.to_f64().to_string(),
    // );

    let total_amount_usd_rounded = token_prices
        .token_price_usd_18
        .round(0, num_bigfloat::RoundingMode::ToOdd)
        .to_string();

    // println!("total_amount_usd_rounded {:#?}", total_amount_usd_rounded);

    let item_to_save = PriceItem {
        signature: signature.to_string(),
        token_ref_address: pool_state.quote_mint.to_string(),
        token_address: pool_state.base_mint.to_string(),
        token_price_usd: token_prices.token_price_usd_18,
        // .token_price_usd_18
        // .round(0, num_bigfloat::RoundingMode::Down)
        // .to_string(), // sol_price_db.to_string(),
        token_ref_price_usd: token_prices.token_ref_price_usd_18.to_string(),
        token_price_usd_formatted: token_prices.token_price_usd_fixed,
        // token_ref_price_usd_formatted: token_amounts_usd_c.price_usd_token_b_formatted.to_string(),
        datetime: datetime,
        signer: transaction_parsed.signer.to_string(),
        ubo: transaction_parsed.ubo.to_string(),
        pool_address: poolvars.pool_id.to_string(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18.to_string(),
        // usd_total_pool: token_amounts_usd_c.usd_total_pool.to_string(),
        price_token_ref: token_prices.token_price_rel_to_ref,
        price_token_ref_formatted: token_prices.token_price_rel_to_ref,
        block_number: transaction_parsed.block_number.to_string(),
    };

    // println!(item_to_save)

    // let j = serde_json::to_string(&address)?;

    let testing = item_to_save.clone();

    // println!(" item_to_save {:#?}", item_to_save);

    let reponse = db_client.save_token_values(item_to_save);

    // let tpo_values_a = TokenPriceOracleValues {
    //     ubo: transaction_parsed.ubo.to_string(),
    //     signer: transaction_parsed.signer.to_string(),
    //     pool_address: poolvars.pool_id.to_string(),
    //     token_address: pool_state.base_mint.to_string(),
    //     signature: signature.to_string(),
    //     usd_total_pool: token_amounts_usd.token_amounts_a.usd_total_pool_18,
    //     usd_total_ubo: token_amounts_usd.token_amounts_a.usd_total_ubo_18,
    //     usd_diff_ubo: token_amounts_usd.token_amounts_a.usd_diff_ubo_18,
    //     usd_diff_pool: token_amounts_usd.token_amounts_a.usd_diff_pool_18,
    //     amount_total_pool: token_amounts.token_amounts_a.amount_total_pool,
    //     amount_diff_pool: token_amounts.token_amounts_a.amount_diff_pool,
    //     amount_total_ubo: token_amounts.token_amounts_a.amount_total_ubo,
    //     amount_diff_ubo: token_amounts.token_amounts_a.amount_diff_ubo,
    // };

    let tpo_values_a = parse_token_price_oracle_values(
        transaction_parsed.ubo.to_string(),
        transaction_parsed.signer.to_string(),
        poolvars.pool_id.to_string(),
        pool_state.base_mint.to_string(),
        &token_amounts.token_amounts_a,
        &swap_token_amounts_priced.token_amounts_priced_a,
        signature,
    );

    // println!("tpo_values_a {:#?}", tpo_values_a);

    let tpo_values_b = parse_token_price_oracle_values(
        transaction_parsed.ubo.to_string(),
        transaction_parsed.signer.to_string(),
        poolvars.pool_id.to_string(),
        pool_state.quote_mint.to_string(),
        &token_amounts.token_amounts_b,
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

    // let response_token_usd_b =
    //     db_client.insert_token_usd_values(&signature, &token_amounts_usd.token_amounts_b);

    // if response_token_usd_b.is_err() {
    //     println!(
    //         "Error saving token usd values to db: {:#?}",
    //         response_token_usd_b
    //     );
    // }

    // let response_token_amounts_a =
    //     db_client.insert_token_amounts(&signature, &token_amounts.token_amounts_a);

    // if response_token_amounts_a.is_err() {
    //     println!(
    //         "Error saving token usd values to db: {:#?}",
    //         response_token_amounts_a
    //     );
    // }

    // let response_token_amounts_b =
    //     db_client.insert_token_amounts(&signature, &token_amounts.token_amounts_b);

    // if response_token_amounts_b.is_err() {
    //     println!(
    //         "Error saving token usd values to db: {:#?}",
    //         response_token_amounts_b
    //     );
    // }

    if reponse.is_err() {
        println!("Error saving to db: {:#?}", reponse);
    }
    // else {
    //     println!(
    //         "Saved to db: {:#?}, datetime: {:#?}",
    //         signature, testing.datetime
    //     );
    // }

    return (
        signature.to_string(),
        testing.datetime,
        "success".to_string(),
    );

    // let item: PriceDbItem = PriceDbItem {
    //     price_token_ref: price_token_ref.to_string(),
    //     price_usd: price_usd.to_string(),
    //     datetime: DateTime::from_timestamp(transaction_parsed.block_timestamp, 0)
    //         .unwrap()
    //         .to_rfc3339(),
    //     signature: signature.to_string(),
    //     token_a_amount: token_amounts.amount_total_pool_a.to_string(),
    //     token_b_amount: token_amounts.amount_total_pool_b.to_string(),
    //     pool_address: "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
    //     token_a_address: transaction_parsed.token_a_address.to_string(),
    //     token_b_address: transaction_parsed.token_b_address.to_string(),
    //     token_b_price_rel: token_amounts.token_b_price_rel.to_string(),
    //     // ubo: transaction_parsed.ubo.to_string(),
    //     // ubo_token_a_pool_amount: transaction_parsed.ubo_token_a_pool_amount,
    //     // ubo_token_b_pool_amount: transaction_parsed.ubo_token_b_pool_amount,
    //     // ubo_token_a_amount: transaction_parsed.ubo_token_a_amount,
    //     // ubo_token_b_amount: transaction_parsed.ubo_token_b_amount,
    //     // ubo_pool_perc: "todo".to_string(),
    // };

    // println!("{:#?}", item);

    // return item;
}
