use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
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

use tokio::sync::mpsc;

mod scheduler;

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
    // let task_queue = TaskQueue::new();

    let task_manager = scheduler::TaskManager::new();

    // Add some initial tasks
    task_manager.add_task("Task 1".to_string());
    task_manager.add_task("Task 2".to_string());
    task_manager.add_task("Task 3".to_string());

    // Run the task manager
    task_manager.run().await;

    // let task1 = CustomTask {
    //     name: "Do laundry".to_string(),
    //     priority: 2,
    // };

    // let serialized_task1 = bincode::serialize(&task1).unwrap();
    // tx.send(serialized_task1).await.unwrap();

    // task_queue.get_tasks();

    // let tasks = vec![
    //     CustomTask {
    //         priority: 1,
    //         name: "Do laundry".to_string(),
    //     },
    //     CustomTask {
    //         priority: 1,
    //         name: "Buy groceries".to_string(),
    //     },
    //     CustomTask {
    //         priority: 1,
    //         name: "Buy groceries".to_string(),
    //     },
    //     CustomTask {
    //         priority: 1,
    //         name: "Walk the dog".to_string(),
    //     },
    // ];

    // // Send the tasks to the queue
    // for task in tasks {
    //     task_queue.add_task(task).await;
    // }

    // let (tx, mut rx) = mpsc::channel::<Task>(100);

    // Spawn a new task to process items from the queue
    // tokio::spawn(async move {
    //     while let Some(task) = rx.recv().await {
    //         println!("Processing task: {}", task.description);

    //         // Perform the task here
    //     }
    // });

    // let task = Task {
    //     description: "Do laundry".to_string(),
    // };

    // scheduled_new_tasks(&tx, &mut scheduled_tasks).await;

    // Close the sender side of the channel
    // drop(tx);

    // let mut queue = scheduler::TaskQueue::new(100);

    // queue.process_tasks().await;

    // queue.process_tasks().await;

    Ok(())
}

// pub fn consume_tasks(queue: &TaskQueue) {
// while let Some(task) = queue.receive() {
//     println!("Processing task: {}", task.description)
//     // Execute the task
// }
// }

// async fn scheduled_new_tasks(
//     tx: &tokio::sync::mpsc::Sender<Task>,
//     scheduled_tasks: &mut HashSet<Task>,
// ) {

//     //     TaskQueue::send(task);
//     //     // if !scheduled_tasks.contains(&task) {
//     //     // println!("Task not scheduled: {}", task.description);
//     //     // // Schedule the task
//     //     // scheduled_tasks.insert(task.clone());
//     //     // tx.send(Task {
//     //     //     description: "Do laundry".to_string(),
//     //     // }) // Add some tasks to the queue
//     //     // .await
//     //     // .unwrap();
//     //     // } else {
//     //     //     println!("Task already scheduled: {}", task.description);
//     //     // }
//     // }
// }
