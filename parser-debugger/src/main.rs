use block_parser::rpc_pool_manager::{RpcPool, RpcPoolManager};
use block_parser::token_db::{DbClientPoolManager, DbPool};
use block_parser::token_parser::PoolMeta;
use block_parser::{block_parser::parse_block, interfaces::ParserConnections};
use moka::future::Cache;

#[tokio::main]
async fn main() {
    let cache: Cache<String, Option<PoolMeta>> = Cache::new(1_000_000);

    let mgr = RpcPoolManager {
        rpc_type: None, //args.rpc_type,
    };

    let mgr_info = RpcPoolManager {
        rpc_type: Some("info_rpc".to_string()),
    };

    let rpc_connection = RpcPool::builder(mgr).max_size(1000).build().unwrap();

    let rpc_connection_builder = RpcPool::builder(mgr_info).max_size(1000).build().unwrap();

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let db_pool_connection = DbPool::builder(db_mgr).max_size(1000).build().unwrap();

    // let connect = rpc_connection_builder.clone().get().await.unwrap();

    let connections = ParserConnections {
        rpc_connection,
        rpc_connection_builder,
        db_client: db_pool_connection,
        my_cache: cache,
    };

    let rpc_connection_c = connections.rpc_connection.clone();
    let rpc_connection_builder_c = connections.rpc_connection_builder.clone();
    let db_client_c = connections.db_client.clone();
    let my_cache_c = connections.my_cache.clone();

    let counter_value = 272017655; //connect.get_slot().await.unwrap_or(265757043);

    let _signature =
        "3QWhRiQh9HSS8VjMWrc74JmRNvtfyAFrP612oEoBuftHjC6zJa3qo6SdDAJ16hw88pdVSZ6YS1UeTKRE9V9aR7e5"
            // "2gCeBpDFx4wXK4jwDa7X3Sq7wwK7Zoz1GFmryXefatMA3rDJiBod8cfeQPmKvfvCuweMPmUXKvCrEncz2bTuPTG3"
            // "5msERHwCbzDLWgmcx11pCz1GjGiVR7Vx49QaJmuNRyD75DkaH8dWYfx5j9qFdry39XRW2va9oLag9tDDZuuUYqrY"
            .to_string();

    let _result = parse_block(
        counter_value as u64,
        &rpc_connection_c,
        &rpc_connection_builder_c,
        &db_client_c,
        &my_cache_c,
    )
    .await;

    // if result.is_err() {
    //     println!("Error parsing block: {:?}", result.err());
    // }

    // ingest_transaction(
    //     &rpc_connection_builder_c,
    //     &db_client_c,
    //     my_cache_c,
    //     signature,
    //     None,
    //     None,
    // )
    // .await;

    println!("Current block height: {:?}", counter_value);

    loop {}
    // let dolar = ingest_transaction(
    //     &rpc_connection_builder_c,
    //     &db_client_c,
    //     my_cache_c,
    //     "4r4tCLTiQS4wLAjSW6NDGgPB5bBKWo6cveSNxh4EkMLG2cM2CNdFjD5x57UZR5eDZewfLMQ5yyWcoNKQzJ1fg7TQ"
    //         .to_string(),
    //     None,
    //     None,
    // )
    // .await;

    // println!("Transaction response: {:?}", dolar);

    // parse_block(
    //     block_number,
    //     rpc_connection,
    //     rpc_connection_builder,
    //     db_client,
    //     my_cache,
    // )
}
