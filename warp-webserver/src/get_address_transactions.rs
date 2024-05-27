use block_parser::{
    get_signatures::get_paginated_singatures,
    interfaces::{ArrayMapRequest, CtTransaction, TransactionParsedResponse},
    rpc_pool_manager::{RpcPool},
    token_db::{DbPool},
    token_parser::PoolMeta,
    transactions_loader::{get_transaction_priced, TransactionError},
};

// use futures::stream::{FuturesUnordered, StreamExt};
use moka::future::Cache;
use serde::Serialize;
use std::collections::VecDeque;
use tokio::task::{JoinSet};
use warp::{reply::Reply};

// use crate::ArrayMapRequest;

#[derive(Serialize)]
pub struct Response {
    adddress: String,
    next_hash: Option<String>,
    success: bool,
    transactions: Vec<TransactionParsedResponse>,
    // signature: Vec<String>,
}

pub async fn get_address_transactions_handler(
    rpc_connection: RpcPool,
    db_pool: DbPool,
    cache: Cache<String, Option<PoolMeta>>,
    address: String,
    params: ArrayMapRequest,
) -> Result<impl Reply, warp::Rejection> {
    // println!("testing: {:#?}", expand.clone());

    let mut tasks = VecDeque::new();

    // let mut task_handles = VecDeque::new();

    let expand = params.expand.clone();

    let before_signature_param = None;
    let sample_rate = None;

    let (signatures, next_hash) = get_paginated_singatures(
        &address,
        rpc_connection.clone(),
        before_signature_param,
        sample_rate,
    )
    .await;

    // let mut results = VecDeque::new();
    // let mut futures = Vec::new();

    // for i in 0..10 {
    //     let future = async move {
    //         // Simulate some async work
    //         tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    //         i
    //     };
    //     futures.push(future);
    // }

    // while let Some(result) = futures.next().await {
    //     results.push_back(result);
    // }

    let _signatures_to_process: JoinSet<Result<CtTransaction, TransactionError>> =
        JoinSet::new();

    for signature in signatures {
        let rpc_connect = rpc_connection.clone();
        let db_connect = db_pool.clone();
        let cache_connect = cache.clone();

        let handle = tokio::spawn(async move {
            // wait for ratelimiting
            // tester.wait().await;
            let results =
                get_transaction_priced(rpc_connect, db_connect, cache_connect, signature).await;
            return results;
        });

        tasks.push_back(handle);
    }

    let _crawled_signatures: Vec<String> = Vec::new();
    let mut transactions = Vec::<TransactionParsedResponse>::new();

    // while let Some(res) = signatures_to_process.join_next().await {
    //     crawled_signatures.push("tesitng".to_string());

    //     if (res.is_err()) {
    //         println!("Error processing signature: {:#?}", res.err());
    //         continue;
    //     }

    //     let tesitng = res.unwrap();

    //     let result = tesitng.unwrap().format(expand.clone());

    //     // println!("Processing signature: {:#?}", result.actions);
    //     // let testing = res.unwrap();

    //     transactions.push(result);
    // }

    while let Some(handle) = tasks.pop_front() {
        let res: Result<CtTransaction, TransactionError> = handle.await.unwrap();

        // let res = result.unwrap();
        // results.push(result);

        if res.is_err() {
            println!("Error processing signature: {:#?}", res.err());
            continue;
        }

        let tesitng = res.unwrap();

        let result = tesitng.format(expand.clone());

        // println!("Processing signature: {:#?}", result.actions);
        // let testing = res.unwrap();

        transactions.push(result);
    }

    // while let Some(handle) = tasks.pop_front() {
    //     let result = handle.await.unwrap();
    //     println!("Result: {}", result);
    // }

    Ok(warp::reply::json(&&Response {
        adddress: address.to_string(),
        success: true,
        transactions,
        next_hash,
    }))
}
