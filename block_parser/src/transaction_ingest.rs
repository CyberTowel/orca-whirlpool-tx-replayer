use crate::{
    parse_pool::parse_pool_price, rpc_pool_manager::RpcPool, token_db::DbPool,
    token_parser::PoolMeta, transactions_loader::get_transction,
};

use moka::future::Cache;

pub async fn ingest_transaction(
    pool: &RpcPool,
    db_pool: &DbPool,
    _cache: Cache<String, Option<PoolMeta>>,
    signature: String,
    ubo_override: Option<String>,
    sol_price_18: Option<String>,
) {
    let rpc_connect = pool.get().await.unwrap(); // Get a connection from the pool

    let db_client_pricing = db_pool.get().await.unwrap();
    let db_client = db_pool.get().await.unwrap();

    let rpc_response = get_transction(
        signature.clone(),
        // None,
        &rpc_connect,
        ubo_override,
        // &token_db_connect,
        // cache,
    )
    .await;

    if rpc_response.is_err() {
        println!("Error getting transaction: {}", signature);
        // panic!("to implemnent, saving transaction with err to db");
        return;
    }

    let rpc_response = rpc_response.unwrap();

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

    let saved = db_client.insert_transaction_values(&rpc_response);
    if saved.is_err() {
        println!("Error saving transaction values: {:#?}", saved);
    }

    // println!("Saved transaction: {}", signature);
}
