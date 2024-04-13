use crate::token_db::{PriceItem, TokenDbClient};
use crate::token_parser::{get_price, get_token_amounts, parse_combined};
use crate::transaction::Transaction;
use chrono::prelude::*;
use num::traits::sign;
use rust_decimal::prelude::*;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

pub fn parser_transaction(
    signature: &String,
    rpc_connection: &RpcClient,
    db_client: &TokenDbClient,
) {
    let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(1),
    };

    let transaction_req = rpc_connection
        .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

    let transaction = transaction_req.unwrap();

    let transaction_parsed = Transaction::new(&transaction);

    let pool_coin_token_account = "Ffo9MEhfH5tBBkZMi1vWVpZLqmbDKvEWJhW3XyMQz4QY";
    let amm_target_orders = "EM9ebwJyrenPmgXQyn9aR5X2tiJssrVPwLSZXxmg2dLy";

    let token_amounts = get_token_amounts(
        &transaction,
        &transaction_parsed.account_keys,
        &transaction_parsed.ubo,
        &transaction_parsed.token_a_address,
        &transaction_parsed.token_b_address,
        &pool_coin_token_account,
        &amm_target_orders,
    );

    let price_token_a_18 = db_client
        .get_usd_price_sol(transaction_parsed.datetime)
        .unwrap();

    let price = get_price(
        token_amounts.price_usd_token_b,
        &price_token_a_18.to_string(),
    );

    let (price_token_b_18, price_token_ref) = price.unwrap();
    // println!(
    //     "price_usd: {:?}, {:?}, price_token_a: {:?}",
    //     price_token_b_18,
    //     price_token_ref.to_string(),
    //     price_token_a_18.to_string(),
    // );

    let token_amounts_usd = parse_combined(
        &token_amounts,
        price_token_a_18.to_f64().unwrap(),
        price_token_b_18,
    );

    let datetime = DateTime::from_timestamp(transaction_parsed.block_timestamp, 0)
        .unwrap()
        .to_rfc3339();

    //     println!(
    //         "
    // ---- transaction info ----
    // signer          {:#?},
    // UBO             {:#?}
    // datetime        {:#?}
    // signature       {:#?}
    // pool_address    {:#?}

    // ",
    //         transaction_parsed.signer,
    //         transaction_parsed.ubo,
    //         datetime,
    //         signature,
    //         "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
    //     );

    // println!(
    //     "
    // ---- token amounts ----
    // {:#?}

    // ---- token amounts USD ----
    // {:#?} ",
    //     token_amounts, token_amounts_usd
    // );

    // let token_a_usd_json = serde_json::to_string(&token_amounts_usd.token_amounts_a).unwrap();

    let item_to_save = PriceItem {
        signature: signature.to_string(),
        token_a_address: transaction_parsed.token_a_address.to_string(),
        token_b_address: transaction_parsed.token_b_address.to_string(),
        token_a_price_usd: price_token_a_18.to_string(),
        token_b_price_usd: price_token_b_18.to_string(),
        token_a_price_usd_formatted: token_amounts_usd.price_usd_token_a_formatted.to_string(),
        token_b_price_usd_formatted: token_amounts_usd.price_usd_token_b_formatted.to_string(),
        datetime: datetime,
        signer: transaction_parsed.signer.to_string(),
        ubo: transaction_parsed.ubo.to_string(),
        pool_address: "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
        usd_total_pool: token_amounts_usd.usd_total_pool.to_string(),
        token_a_usd: token_amounts_usd.token_amounts_a,
        token_b_usd: token_amounts_usd.token_amounts_b,
        token_amounts_a: token_amounts.token_amounts_a,
        token_amounts_b: token_amounts.token_amounts_b,
    };

    // let j = serde_json::to_string(&address)?;

    let testing = item_to_save.clone();

    let reponse = db_client.insert_token_price(item_to_save);

    if reponse.is_err() {
        println!("Error saving to db: {:#?}", reponse);
    } else {
        println!(
            "Saved to db: {:#?}, datetime: {:#?}",
            signature, testing.datetime
        );
    }

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
