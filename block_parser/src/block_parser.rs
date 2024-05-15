use chrono::DateTime;
use deadpool::managed::Pool;
use moka::future::Cache;
use serde_json::json;

use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::UiTransactionEncoding;

use crate::{
    rpc_pool_manager::RpcPoolManager, token_db::DbClientPoolManager, token_parser::PoolMeta,
    transactions_loader::init,
};

#[derive(Debug)]
pub enum RpcErrorCustom {
    BlockNotFoundError, // {
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
) -> Result<(u64, usize, std::time::Duration, std::time::Duration, String), RpcErrorCustom> {
    let start = std::time::Instant::now();
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

    let duration_rpc = start.elapsed();
    // println!(
    //     "Time elapsed to get block {} is: {:?}",
    //     block_number, duration_rpc
    // );

    if block_req.is_err() {
        let error = block_req.as_ref().err().unwrap();

        let _tesitng = error.kind();

        // let error_code: &i64 = match tesitng {
        //     ClientErrorKind::RpcError(RpcResponseError {
        //         code,
        //         message: _,
        //         data: _,
        //     }) => {
        //         // println!("Error getting block test: {:#?}", code);
        //         // return (block_number, 0, duration_rpc, duration_rpc, "".to_string());
        //         code
        //     }
        //     _ => &0,
        // };

        // println!("will return error");

        // return Err({ error_code.clone() });
        return Err(RpcErrorCustom::BlockNotFoundError);

        // println!("Error getting block: {:#?}", error_code);

        // println!("Error getting block: {:#?}", tesitng);

        // if (tesitng. == RpcError::RpcResponseError) {
        //     println!("Error getting block: {:#?}", error);
        //     return (block_number, 0, duration_rpc, duration_rpc, "".to_string());
        // }

        // match error {
        //     _ => {}
        //     RpcResponseError {
        //         code,
        //         message,
        //         data,
        //     } => {
        //         println!("Error getting block: {:#?}", error);
        //         return (block_number, 0, duration_rpc, duration_rpc, "".to_string());
        //     }
        // }
        // if (error.kind == -32009) {
        //     println!("Block not found: {:#?}", block_number);
        //     // return (block_number, 0, duration_rpc, duration_rpc, "".to_string());
        // }

        // println!("Error getting block: {:#?}", block_req.as_ref().err());
        // return Ok((block_number, 0, duration_rpc, duration_rpc, "".to_string()));
    }

    let block = block_req.unwrap();

    let block_transactions = block.transactions.unwrap();

    let transaction_amount = block_transactions.len();

    // println!("transaction amount: {:#?}", block_transactions.len());

    // println!("{:#?}", tesitng.unwrap());

    // let block_time = block.block_time.unwrap();

    let block_time = block.block_time.unwrap();
    let transaction_datetime = DateTime::from_timestamp(block_time, 0)
        .unwrap()
        .to_rfc3339();
    // println!("Block time: {:#?}", transaction_datetime);

    for transaction in block_transactions {
        let rpc_conn = rpc_connection.get().await.unwrap();
        let rpc_build_conn = rpc_connection_builder.get().await.unwrap();
        let db_conn = db_client.get().await.unwrap();

        let signature = json!(transaction.transaction);

        let _signature = signature["signatures"][0].as_str().unwrap().to_string();

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
    ))

    // println!("Transaction amount {:#?}", signature);
}
