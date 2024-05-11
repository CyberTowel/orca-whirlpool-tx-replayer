use deadpool::managed::{self, Metrics, Pool};
use moka::future::Cache;
use serde_json::json;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_sdk::{commitment_config::CommitmentConfig, signature};
use solana_transaction_status::UiTransactionEncoding;

use crate::{
    rpc_pool_manager::RpcPoolManager,
    token_db::{DbClientPoolManager, TokenDbClient},
    token_parser::PoolMeta,
    transactions_loader::init,
};

pub async fn parse_block(
    block_number: u64,
    rpc_connection: &Pool<RpcPoolManager>,
    rpc_connection_builder: &Pool<RpcPoolManager>,
    db_client: &Pool<DbClientPoolManager>,
    my_cache: &Cache<String, PoolMeta>,
) {
    //

    let connection = rpc_connection.get().await.unwrap();
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

    if (block_req.is_err()) {
        return;
    }

    let block = block_req.unwrap();

    let block_transactions = block.transactions.unwrap();

    println!("transaction amount: {:#?}", block_transactions.len());

    // println!("{:#?}", tesitng.unwrap());

    // let block_time = block.block_time.unwrap();

    for transaction in block_transactions {
        let rpc_conn = rpc_connection.get().await.unwrap();
        let rpc_build_conn = rpc_connection_builder.get().await.unwrap();
        let db_conn = db_client.get().await.unwrap();

        let signature = json!(transaction.transaction);

        let signature = signature["signatures"][0].as_str().unwrap().to_string();

        let block_time = block.block_time.unwrap();

        // let transaction_encoded = transaction;

        let signature = json!(transaction.transaction);

        let signature = signature["signatures"][0].as_str().unwrap().to_string();

        let cache_clone = my_cache.clone();

        tokio::spawn(async move {
            init(
                signature,
                None,
                &rpc_conn,
                &rpc_build_conn,
                &db_conn,
                &cache_clone,
                &transaction,
                block_time,
                block_number,
            )
            .await;
        });
    }

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

    // println!("Transaction amount {:#?}", signature);
}
