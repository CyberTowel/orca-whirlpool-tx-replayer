use chrono::DateTime;
use deadpool::managed::Pool;
use moka::future::Cache;
use serde_json::json;

use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::UiTransactionEncoding;

use crate::{
    rpc_pool_manager::RpcPoolManager,
    token_db::DbClientPoolManager,
    token_parser::PoolMeta,
    transaction_ingest::ingest_transaction,
    transactions_loader::to_replace_parse_transaction_and_save_values, // transactions_loader::{parse_transaction_and_save_values},
};

#[derive(Debug)]
pub enum RpcErrorCustom {
    BlockNotFoundError,
    // {
    //     code: i64,
    //     message: String,
    //     data: String,
    // },
}

pub async fn parse_block(
    block_number: u64,
    rpc_connection: &Pool<RpcPoolManager>,
    rpc_connection_builder: &Pool<RpcPoolManager>,
    db_client: &Pool<DbClientPoolManager>,
    my_cache: &Cache<String, Option<PoolMeta>>,
) -> Result<
    (
        u64,
        usize,
        std::time::Duration,
        std::time::Duration,
        String,
        String,
    ),
    RpcErrorCustom,
> {
    let start = std::time::Instant::now();
    let connection = rpc_connection.get().await.unwrap();

    // println!("======== block_number: {}", block_number);
    // let db_connection = db_client.get().await.unwrap();

    let rpc_block_config = RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: Some(CommitmentConfig::finalized()),
        transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
        rewards: Some(false),
        max_supported_transaction_version: Some(0),
    };

    let block_req = connection
        .get_block_with_config(block_number, rpc_block_config)
        .await;

    let duration_rpc = start.elapsed();

    if block_req.is_err() {
        let error = block_req.as_ref().err().unwrap();

        let _tesitng = error.kind();

        // return Err({ error_code.clone() });
        return Err(RpcErrorCustom::BlockNotFoundError);
    }

    let block = block_req.unwrap();

    let block_transactions = block.transactions.unwrap();

    let transaction_amount = block_transactions.len();

    // let block_time = block.block_time.unwrap();

    let block_time = block.block_time.unwrap();
    let transaction_datetime = DateTime::from_timestamp(block_time, 0)
        .unwrap()
        .to_rfc3339();

    let db_client_sol_price = db_client.get().await.unwrap();
    let sol_price_transaction_datetime = transaction_datetime.clone();
    let sol_price_db = db_client_sol_price
        .get_token_price_usd(
            &sol_price_transaction_datetime,
            "So11111111111111111111111111111111111111112".to_string(),
        )
        .unwrap();

    // println!(
    //     "============== information loaded - start parsing block {} ",
    //     block_number
    // );

    for transaction in block_transactions {
        let sol_price_db_c = sol_price_db.clone();
        // let rpc_conn = rpc_connection.get().await.unwrap();
        // let rpc_build_conn = rpc_connection_builder.get().await.unwrap();
        // let db_conn = db_client.get().await.unwrap();

        let testing = rpc_connection_builder.clone();
        let testng_db = db_client.clone();

        let signature = json!(transaction.transaction);

        let _signature = signature["signatures"][0].as_str().unwrap().to_string();

        let signature = json!(transaction.transaction);

        let signature = signature["signatures"][0].as_str().unwrap().to_string();

        let cache_clone = my_cache.clone();

        tokio::spawn(async move {
            ingest_transaction(
                &testing,
                &testng_db,
                cache_clone,
                signature,
                None,
                Some(sol_price_db_c),
            )
            .await;
            // to_replace_parse_transaction_and_save_values(
            //     signature,
            //     None,
            //     &rpc_conn,
            //     &rpc_build_conn,
            //     &db_conn,
            //     &cache_clone,
            //     &transaction,
            //     block_time,
            //     block_number,
            //     Some(sol_price_db_c),
            // )
            // .await;
        });
    }

    let duraction_total = start.elapsed();

    // let block_number = block.to_string();

    // init(
    //     signature,
    //     None,
    //     &rpc_connection,
    //     db_client,
    //     my_cache,
    //     transaction_encoded,
    //     block_time,
    //     block_number,
    // )
    // .await;

    Ok((
        block_number,
        transaction_amount,
        duration_rpc,
        duraction_total,
        transaction_datetime,
        sol_price_db.to_string(),
    ))
}
