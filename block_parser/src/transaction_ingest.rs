use crate::{
    interfaces::PriceItem,
    parse_pool::{get_pool_id, parse_pool_price},
    pool_state::get_pool_meta,
    rpc_pool_manager::RpcPool,
    token_db::DbPool,
    token_parser::{
        get_price, get_token_amounts, parse_token_amounts_new, parse_token_price_oracle_values,
        PoolMeta,
    },
    transaction,
    transactions_loader::get_transction,
};

use moka::future::Cache;
use solana_client::rpc_response::Response;

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
