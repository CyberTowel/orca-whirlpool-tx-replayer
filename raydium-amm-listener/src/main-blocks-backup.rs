use std::borrow::Borrow;
use std::collections::HashMap;
use std::pin::Pin;
use std::thread::{self, sleep};
use std::time::Duration;

use anyhow::Result;
// use backfill::backfill::backfill_tree;
// use config::rpc_config::{get_pubsub_client, setup_rpc_clients};
// use dotenv::dotenv;
// use futures::future::join;
use futures::prelude::*;
use futures::stream::SelectAll;
use futures::{future::join_all, stream::select_all};
use moka::future::Cache;
use solana_client::nonblocking::pubsub_client::PubsubClient;
// use mpl_bubblegum::accounts::MerkleTree;
// use processor::logs::process_logs;
// use processor::metadata::fetch_store_metadata;
// use processor::queue_processor::process_transactions_queue;
// use sea_orm::SqlxPostgresConnector;
use solana_client::rpc_config::{
    RpcBlockConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter, RpcTransactionLogsConfig,
    RpcTransactionLogsFilter,
};
use solana_client::rpc_response::{Response, RpcLogsResponse, SlotUpdate};
use solana_sdk::commitment_config::CommitmentConfig;
// use sqlx::{Acquire, PgPool};
use clap::Parser;
use deadpool::managed::{self, Metrics, Pool};
use solana_sdk::signature;
use solana_sdk::transaction::Transaction;
use tokio::task::{self, JoinSet};
use transaction_parser;
use transaction_parser::block_parser::parse_block;
use transaction_parser::rpc_pool_manager::{get_pub_sub_client, RpcPool, RpcPoolManager};
// use transaction_parser::
use solana_transaction_status::UiTransactionEncoding;

use deadpool::managed::RecycleResult;
use transaction_parser::token_db::{DbClientPoolManager, DbPool};
use transaction_parser::token_parser::PoolMeta;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    /// maximum depth to which sub-directories should be explored
    sample_rate: Option<usize>,

    #[clap(long)]
    start_at: Option<String>,

    #[clap(long)]
    pool_id: Option<String>,

    #[clap(long)]
    rpc_type: Option<String>,

    #[clap(long)]
    rate_limit: Option<usize>,

    #[clap(long)]
    sleep: Option<usize>,

    #[clap(long)]
    block_amount: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mgr = RpcPoolManager {
        rpc_type: args.rpc_type,
    };

    let mgr_info = RpcPoolManager {
        rpc_type: Some("info_rpc".to_string()),
    };

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let db_pool_connection = DbPool::builder(db_mgr).max_size(1000).build().unwrap();

    let rpc_connection = RpcPool::builder(mgr).max_size(1000).build().unwrap();

    let rpc_connection_builder = RpcPool::builder(mgr_info).max_size(1000).build().unwrap();

    let cache: Cache<String, Option<PoolMeta>> = Cache::new(100_000);

    let block_number_cache: Cache<u64, (u64, usize, Duration, Duration)> = Cache::new(100_000);

    let connection = rpc_connection_builder.get().await.unwrap();

    let mut block_to_get = connection.get_slot().await.unwrap();

    println!("Start at block: {:#?}", block_to_get);

    let start = std::time::Instant::now();

    let items_to_get = if args.block_amount.is_some() {
        args.block_amount.unwrap() as u64
    } else {
        10
    };

    let mut signatures_to_process = JoinSet::new();

    // loop {
    //     let connection = rpc_connection_builder.get().await.unwrap();

    //     let slot = connection.get_slot().await.unwrap();

    //     let start = std::time::Instant::now();

    //     // let block_number = start_at_block + i;
    //     let connection = rpc_connection.clone();
    //     let db_pool_connect = db_pool_connection.clone();
    //     let rpc_connection_builder = rpc_connection_builder.clone();
    //     let my_cache = cache.clone();

    //     println!("{:#?} block received", slot);

    //     // println!("Connection established, {:#?}", slot);

    //     // println!("Waiting for logs");
    // }

    // return Ok(());

    let mut active_request: HashMap<u64, bool> = HashMap::new();

    println!("hashmap created, {}", active_request.len());

    // return Ok(());

    // for i in 0..items_to_get {
    loop {
        if (active_request.len() > 10) {
            // println!("current hasmap items, {}", active_request.len());
            continue;
        }

        active_request.insert(block_to_get, true);
        block_to_get = block_to_get + 1;
        let start = std::time::Instant::now();

        let block_number = block_to_get;
        let connection = rpc_connection.clone();
        let db_pool_connect = db_pool_connection.clone();
        let rpc_connection_builder = rpc_connection_builder.clone();
        let my_cache = cache.clone();

        signatures_to_process.spawn(async move {
            let result = parse_block(
                block_number,
                &connection,
                &rpc_connection_builder,
                &db_pool_connect,
                &my_cache,
            )
            .await;
            let duration = start.elapsed();

            println!("Time elapsed is: {:?}", duration);
            println!(
                "
=====================================
"
            );

            // active_request.

            return result;
        });

        println!("block done: {}", block_to_get);
        active_request.remove(&block_to_get);
    }

    while let Some(res) = signatures_to_process.join_next().await {
        let (block_number, transaction_amount, duration_rpc, duraction_total) = res.unwrap();

        println!(
            "done with block {}, with {} transaction -> time to get rpc {:?}, total duration: {:?}",
            block_number, transaction_amount, duration_rpc, duraction_total
        );
        // let result_i = match res {
        //     Ok(_) => res.unwrap(),
        //     Err(_) => ("".to_string(), "".to_string(), "".to_string()),
        // };
        // crawled_signatures.push(result_i);
    }

    println!("done");

    let duration = start.elapsed();
    let duration_per_item = duration / items_to_get as u32;

    println!("Time elapsed is: {:?}", duration);
    println!("Time elapsed per item is: {:?}", duration_per_item);

    // parse_block(
    //     265012503,
    //     &rpc_connection,
    //     &rpc_connection_builder,
    //     &db_pool_connection,
    //     &cache,
    // )
    // .await;

    // let tesitng = connection.get_block_with_config(
    //     265012503,
    //     RpcBlockConfig {
    //         encoding: Some(UiTransactionEncoding::JsonParsed),
    //         commitment: Some(CommitmentConfig::finalized()),
    //         transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
    //         rewards: Some(true),
    //         max_supported_transaction_version: Some(0),
    //     },
    // );

    // println!("{:#?}", tesitng.unwrap().transactions.unwrap().len());

    return Ok(());

    // let connection = rpc_connection.clone().get().await.unwrap();

    // let db_pool_connect = db_pool_connection.clone().get().await.unwrap();

    // let signature =
    //     "5r6gK8BeV71QQ7riJHrrEubhT62nPFmumeEML81wtvgGseaZwbdHRobkdkbPePsxQ58PPpxVh2nLHyGywa6o4iVo"
    //         .to_string();

    // let result = transaction_parser::transactions_loader::init(
    //     signature,
    //     None,
    //     &connection,
    //     &db_pool_connect,
    // );

    // return Ok(());

    // let mut rpc_url = "wss://api.mainnet-beta.solana.com/";

    // println!("Connecting to {}", rpc_url);

    // let pubsub_client = PubsubClient::new(rpc_url).await.unwrap();

    println!("Pubsub client created");

    let pubsub_client = get_pub_sub_client().await;

    // 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8

    // let pool_id = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string();
    let pool_id = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string();

    // let testing = pubsub_client
    //     .logs_subscribe(
    //         RpcTransactionLogsFilter::Mentions(vec![
    //             // "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
    //             pool_id.clone(),
    //         ]),
    //         RpcTransactionLogsConfig {
    //             commitment: Some(CommitmentConfig::processed()),
    //         },
    //     )
    //     .await;

    let testing = pubsub_client.slot_updates_subscribe().await;

    // let testing = pubsub_client
    //     .block_subscribe(
    //         RpcBlockSubscribeFilter::All,
    //         // RpcBlockSubscribeFilter::MentionsAccountOrProgram(
    //         //     "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
    //         // ),
    //         None,
    //         // Some(RpcBlockSubscribeConfig {
    //         //     encoding: Some(UiTransactionEncoding::JsonParsed),
    //         //     commitment: Some(CommitmentConfig::processed()),
    //         //     transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
    //         //     show_rewards: Some(true),
    //         //     max_supported_transaction_version: Some(1),
    //         // }),
    //     )
    //     .await;

    println!("Listener added");

    let dolar = testing.unwrap();

    let mut stream = select_all(vec![dolar.0]);
    // let mut signatures_to_process = JoinSet::new();

    loop {
        // let connection = rpc_connection.clone().get().await.unwrap();
        let connection: managed::Object<RpcPoolManager> =
            rpc_connection.clone().get().await.unwrap();

        let my_cache = cache.clone();

        // my_cache
        //     .insert("testing".to_string(), "testing".to_string())
        //     .await;

        // println!("Waiting for logs");

        let db_pool = db_pool_connection.clone().get().await.unwrap();

        let logs_stream = stream.next().await;

        if (logs_stream.is_none()) {
            continue;
        }

        // // println!("testing 1");

        // println!("{:#?}", logs_stream.unwrap());

        let testing_dolar: solana_client::rpc_response::SlotUpdate = logs_stream.unwrap().into();
        println!("{:#?}", testing_dolar);

        //  {
        //     // solana_client::rpc_response::SlotUpdate::Completed { slot, timestamp } => {
        //     //     println!("slot: {}, timestamp: {}", slot, timestamp);
        //     // },
        // }

        // let logs = logs_stream.unwrap();

        // let testing: bool = logs.value.err.is_some();

        // if (testing) {
        //     // println!("Transaction error");
        //     continue;
        // }

        // // println!("{} streams waiting", stream.len().to_string());
        // // println!("{:#?}", logs.value.signature);

        // // let pool_id_c = pool_id.clone();

        // let mut sleep_duraction = 20;
        // if (args.sleep.is_some()) {
        //     sleep_duraction = args.sleep.unwrap();
        // }

        // tokio::spawn(async move {
        //     // sleep(Duration::from_secs(sleep_duraction as u64));

        //     // println!(
        //     //     "new transaction in main thread, {}, start sleep",
        //     //     logs.value.signature
        //     // );
        //     let sleep = tokio::time::sleep(Duration::from_secs(sleep_duraction as u64)).await;

        //     // loop {
        //     //     tokio::select! {
        //     //         () = &mut sleep => {
        //     //             println!("timer elapsed");
        //     //             sleep.as_mut().reset(Instant::now() + Duration::from_millis(50));
        //     //         },
        //     //     }
        //     // }
        //     // match sleep.() {
        //     //     Pending => return Pending,
        //     //     Ready(val) => val,
        //     // }

        //     // println!("{:?}", logs.value.signature);
        //     //     // // let signature = "5wLsoFtf4k1Su9s8xxFeiep3Cx3P7oZWyV8bzEgQ29KqLjGWC2vpeSkvkNG39vPB6QTW5mR5fPJ3AdEdeEKszfMR";

        //     // println!("Processing signature {:?}", logs.value.signature);
        //     let result = transaction_parser::transactions_loader::init(
        //         logs.value.signature,
        //         None,
        //         &connection,
        //         &db_pool,
        //         my_cache,
        //     );
        //     //     // return result;
        // });

        // return Ok(());

        //         else {
        //             transaction_parser::transactions::testing_nested();

        //             transaction_parser::transactions::init(logs.value.signature, &connection);

        //             transaction_parser::add(logs.value.signature.clone());
        //             println!(
        //                 "
        // ===========================================================
        // signature {:?} success
        // ===========================================================
        // ",
        //                 // raydium_amm_saver
        //                 logs.value.signature
        //             );
        //         }

        // process_logs(logs.value).await;
    }

    // handle_stream(testing.unwrap());
    Ok(())
    // .into_iter();

    // let stream = testing.map(|mut result| {
    //     let mut testing = result.0.next();

    // match testing {
    //     Ok(subscription) => {
    //         let stream = subscription.0;
    //         task::spawn(handle_stream(stream));
    //     }
    //     Err(e) => {
    //         eprintln!("error creating subscription: {e}");
    //     }
    // }
    // });

    // let handle = task::spawn(handle_stream(stream));

    // // task::spawn(handle_metadata_downloads(database_pool.clone()));

    // // join_all(tree_addresses.into_iter().map(backfill_tree)).await;

    // // task::spawn(process_transactions_queue(database_pool.clone())).await?;

    // Ok(())
}

// async fn handle_stream(
//     mut stream: SelectAll<Pin<Box<dyn Stream<Item = Response<RpcLogsResponse>> + Send>>>,
// ) {
//     loop {
//         let logs = stream.next().await.unwrap();

//         println!("{:?}", logs.value);
//         // process_logs(logs.value).await;
//     }
// }

// async fn handle_metadata_downloads(pool: PgPool) {
//     let connection = SqlxPostgresConnector::from_sqlx_postgres_pool(pool);
//     loop {
//         let _ = fetch_store_metadata(&connection).await;
//         println!("No metadata to update, sleeping for 5 secs");
//         sleep(Duration::from_secs(5))
//     }
// }
