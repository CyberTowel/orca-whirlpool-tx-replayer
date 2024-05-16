use moka::future::Cache;
use warp::{
    reject::{custom, Reject},
    reply, Filter, Reply,
};

use crate::User;
use block_parser::{
    rpc_pool_manager::{RpcPool, RpcPoolManager},
    token_db::DbPool,
    token_parser::PoolMeta,
    transactions_loader,
};

pub async fn get_user(signature: String) -> impl Reply {
    let user = User {
        name: "Test".to_string(),
        age: 30,
        signature,
    };

    // let testing = transactions_loader::get_transction(signature.clone(), None).await;

    warp::reply::json(&user)
}

pub async fn handler(
    pool: RpcPool,
    db_pool: DbPool,
    cache: Cache<String, Option<PoolMeta>>,
    signature: String,
) -> Result<impl Reply, warp::Rejection> {
    let user = User {
        name: "Test".to_string(),
        age: 30,
        signature: "test".to_string(),
    };
    // let custom_rejection = custom("This is a custom rejection".to_string());

    let rpc_connect = pool.get().await.unwrap(); // Get a connection from the pool
    let token_db_connect = db_pool.get().await.unwrap(); // Get a connection from the pool

    let testing = transactions_loader::get_transction(
        signature.clone(),
        None,
        &rpc_connect,
        &token_db_connect,
        cache,
    )
    .await;

    if testing.is_err() {
        return Ok(warp::reply::json(&{}));
    }

    let transaction = testing.unwrap();

    println!("testing {:#?}", transaction);
    // Use the connection...
    Ok(warp::reply::json(&transaction))
}
