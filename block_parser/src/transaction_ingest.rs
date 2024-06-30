use crate::{
    parse_pool::parse_pool_price, rpc_pool_manager::RpcPool, token_db::DbPool,
    token_parser::PoolMeta, transactions_loader::parse_transaction_ecoded,
};

use moka::future::Cache;
use solana_transaction_status::EncodedTransactionWithStatusMeta;

pub async fn ingest_transaction(
    pool: &RpcPool,
    db_pool: &DbPool,
    _cache: Cache<String, Option<PoolMeta>>,
    signature: String,
    ubo_override: Option<String>,
    sol_price_18: Option<String>,
    transaction: &EncodedTransactionWithStatusMeta,
    block_time: i64,
    block_number: u64,
) {
    let rpc_connect = pool.get().await.unwrap(); // Get a connection from the pool

    let db_client_pricing = db_pool.get().await.unwrap();
    let db_client = db_pool.get().await.unwrap();

    let rpc_response = parse_transaction_ecoded(
        transaction,
        block_time,
        block_number,
        ubo_override,
        // &token_db_connect,
        // cache,
    );

    if rpc_response.is_err() {
        println!("Error getting transaction: {}", signature);
        // panic!("to implemnent, saving transaction with err to db");
        return;
    }

    let rpc_response = rpc_response.unwrap();

    let saved = db_client.insert_transaction_values(&rpc_response);
    if saved.is_err() {
        println!("Error saving transaction values: {:#?}", saved);
    }

    // if rpc_response.err.is_some() {
    //     println!("Error in transaction: {:#?}", rpc_response.err);
    //     return;
    // }

    let pool_meta_opt = parse_pool_price(
        rpc_response.clone(),
        rpc_connect,
        db_client_pricing,
        _cache,
        sol_price_18,
    )
    .await;

    if !pool_meta_opt.is_some() {
        // println!("No pool_meta_opt");
        // return;
        // return Ok(transaction_base);
    }

    // let price_item = pool_meta_opt.unwrap();

    // let reponse = db_client.save_token_values(price_item_to_save);

    // if (reponse.is_err()) {
    //     println!("Error saving price item: {:#?}", reponse);
    // }

    // println!("Saved transaction: {}", signature);
}
