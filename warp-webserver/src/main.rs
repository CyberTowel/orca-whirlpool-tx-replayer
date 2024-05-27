use get_transactions::handler;
use moka::future::Cache;
use serde::{Serialize};


use warp::Filter;
mod get_address_transactions;
mod get_transactions;
use block_parser::{
    interfaces::ArrayMapRequest,
    rpc_pool_manager::{RpcPool, RpcPoolManager},
    token_db::{DbClientPoolManager, DbPool},
    token_parser::PoolMeta,
};

use crate::get_address_transactions::get_address_transactions_handler;

fn with_rpc_pool(
    pool: RpcPool,
) -> impl Filter<Extract = (RpcPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_token_db_pool(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_testing(
    pool: Cache<String, Option<PoolMeta>>,
) -> impl Filter<Extract = (Cache<String, Option<PoolMeta>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || pool.clone())
}

// fn with_pool_cache(
//     pool: DbPool,
// ) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
//     warp::any().map(|| pool.clone())
// }

#[derive(Serialize)]
pub struct User {
    name: String,
    age: u32,
    signature: String,
}

use serde_qs::Config;

fn custom_config() -> Config {
    let config = Config::new(5, true);
    config
}
#[tokio::main]
async fn main() {
    let mgr_info = RpcPoolManager {
        rpc_type: Some("info_rpc".to_string()),
    };

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let cache: Cache<String, Option<PoolMeta>> = Cache::new(1_000_000);

    let rpc_connection_builder = RpcPool::builder(mgr_info).max_size(1000).build().unwrap();
    // let db_pool = DbPool::builder().max_size(1000).build().unwrap();
    let db_pool_connection = DbPool::builder(db_mgr).max_size(1000).build().unwrap();

    // let user_route = warp::path("transaction")
    //     .and(warp::path::param::<String>())
    //     .map(|signature| get_user(signature));

    let route = warp::path("path")
        .and(with_rpc_pool(rpc_connection_builder.clone()))
        .and(with_token_db_pool(db_pool_connection.clone()))
        .and(with_testing(cache.clone()))
        .and(warp::path::param::<String>())
        .and(serde_qs::warp::query::<ArrayMapRequest>(custom_config()))
        .and_then(handler);

    let route_addresses = warp::path("address")
        .and(with_rpc_pool(rpc_connection_builder.clone()))
        .and(with_token_db_pool(db_pool_connection.clone()))
        .and(with_testing(cache.clone()))
        .and(warp::path::param::<String>())
        .and(warp::path("tx"))
        .and(warp::get())
        .and(serde_qs::warp::query::<ArrayMapRequest>(custom_config()))
        .and_then(get_address_transactions_handler);
    // .with(log);

    // let users_route = warp::path("users").map(|| get_users());

    // let routes = warp::get().and(user_route.or(users_route));

    let routes = warp::get().and(route).or(route_addresses);

    warp::serve(routes).run(([0, 0, 0, 0], 8081)).await;

    println!("Hello, world!, server running");
}
