use std::borrow::Borrow;
use std::pin::Pin;
use std::thread::sleep;
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
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::rpc_response::{Response, RpcLogsResponse};
use solana_sdk::commitment_config::CommitmentConfig;
// use sqlx::{Acquire, PgPool};
use clap::Parser;
use deadpool::managed::{self, Metrics, Pool};
use solana_sdk::signature;
use tokio::task::{self, JoinSet};
use transaction_parser;
use transaction_parser::rpc_pool_manager::{get_pub_sub_client, RpcPool, RpcPoolManager};
// use transaction_parser::

use deadpool::managed::RecycleResult;
use transaction_parser::token_db::{DbClientPoolManager, DbPool};

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mgr = RpcPoolManager {
        rpc_type: args.rpc_type,
    };

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let db_pool_connection = DbPool::builder(db_mgr).max_size(100).build().unwrap();

    let rpc_connection = RpcPool::builder(mgr).max_size(100).build().unwrap();

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

    let testing = pubsub_client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![
                // "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                pool_id.clone(),
            ]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::processed()),
            },
        )
        .await;

    println!("Listener added");

    let dolar = testing.unwrap();

    let mut stream = select_all(vec![dolar.0]);
    let mut signatures_to_process = JoinSet::new();

    let cache: Cache<String, String> = Cache::new(10_000);

    loop {
        // let connection = rpc_connection.clone().get().await.unwrap();
        let connection: managed::Object<RpcPoolManager> =
            rpc_connection.clone().get().await.unwrap();

        let my_cache = cache.clone();

        my_cache
            .insert("testing".to_string(), "testing".to_string())
            .await;

        // println!("Waiting for logs");

        let db_pool = db_pool_connection.clone().get().await.unwrap();

        let logs_stream = stream.next().await;

        if (logs_stream.is_none()) {
            continue;
        }
        let logs = logs_stream.unwrap();

        let testing: bool = logs.value.err.is_some();

        if (testing) {
            // println!("Transaction error");
            continue;
        }

        // println!("{} streams waiting", stream.len().to_string());
        // println!("{:#?}", logs.value.signature);

        // let pool_id_c = pool_id.clone();

        signatures_to_process.spawn(async move {
            // println!("{:?}", logs.value.signature);

            let mut sleep_duraction = 20;
            if (args.sleep.is_some()) {
                sleep_duraction = args.sleep.unwrap();
            }
            // println!("sleep start for {} secs", { sleep_duraction });
            // sleep(Duration::from_secs(sleep_duraction as u64));

            tokio::time::sleep(Duration::from_secs(sleep_duraction as u64)).await;

            //     // // let signature = "5wLsoFtf4k1Su9s8xxFeiep3Cx3P7oZWyV8bzEgQ29KqLjGWC2vpeSkvkNG39vPB6QTW5mR5fPJ3AdEdeEKszfMR";

            let result = transaction_parser::transactions_loader::init(
                logs.value.signature,
                None,
                &connection,
                &db_pool,
            );
            //     // return result;
        });

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
