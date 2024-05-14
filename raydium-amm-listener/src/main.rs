use anyhow::Result;
use clap::Parser;
use consumer::start_workers;
use deadpool::managed::Pool;
use flume::{unbounded, Receiver, Sender};
use moka::future::Cache;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use transaction_parser::rpc_pool_manager::{RpcPool, RpcPoolManager};
use transaction_parser::token_db::{DbClientPoolManager, DbPool};
use transaction_parser::token_parser::PoolMeta;

mod consumer;

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

pub struct ParserConnections {
    pub rpc_connection: Pool<RpcPoolManager>,
    pub rpc_connection_builder: Pool<RpcPoolManager>,
    pub db_client: Pool<DbClientPoolManager>,
    pub my_cache: Cache<String, Option<PoolMeta>>,
}

pub struct BlockParsedDebug {
    pub block_number: u64,
    pub transaction_amount: usize,
    pub duration_rpc: std::time::Duration,
    pub duraction_total: std::time::Duration,
    pub transaction_datetime: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mgr = RpcPoolManager {
        rpc_type: args.rpc_type,
    };

    let mgr_info = RpcPoolManager {
        rpc_type: Some("info_rpc".to_string()),
    };

    let mut durations_total: VecDeque<Duration> = VecDeque::with_capacity(10);
    let mut rolling_avg_total = Duration::new(0, 0);

    let mut durations_rpc: VecDeque<Duration> = VecDeque::with_capacity(10);
    let mut rolling_avg_rpc = Duration::new(0, 0);

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let db_pool_connection = DbPool::builder(db_mgr).max_size(1000).build().unwrap();

    let rpc_connection = RpcPool::builder(mgr).max_size(1000).build().unwrap();

    let rpc_connection_builder = RpcPool::builder(mgr_info).max_size(1000).build().unwrap();

    let cache: Cache<String, Option<PoolMeta>> = Cache::new(1_000_000);

    let connections = ParserConnections {
        rpc_connection,
        rpc_connection_builder,
        db_client: db_pool_connection,
        my_cache: cache,
    };

    let start_at = 245528177;

    let (block_parser_worker, block_parser_watcher) = flume::unbounded::<u64>();
    let block_completed_worker_block_worker = block_parser_worker.clone();

    // let block_parser_worker_internal = block_parser_worker.clone();

    let counter = Arc::new(AtomicUsize::new(245528177));

    let (_block_completed_worker, block_completed_watcher) =
        flume::unbounded::<Option<BlockParsedDebug>>();

    tokio::spawn(async move {
        while let Ok(msg) = block_completed_watcher.recv_async().await {
            let result = msg.unwrap();

            durations_total.push_back(result.duraction_total);
            durations_rpc.push_back(result.duration_rpc);

            if durations_total.len() > 10 {
                rolling_avg_total -= durations_total.pop_front().unwrap();
                rolling_avg_rpc -= durations_rpc.pop_front().unwrap();
            }

            rolling_avg_total += result.duraction_total;
            rolling_avg_rpc += result.duration_rpc;

            let avg = rolling_avg_total / durations_total.len() as u32;
            let avg_rpc = rolling_avg_rpc / durations_rpc.len() as u32;

            let completed_task = result.block_number - start_at;

            println!(
                "Completed task {:?} Block number: {} timestmap: {} transaction #: {} Rolling average total: {:?}, rolling avarage get_block {:?}",
                completed_task,
                result.block_number,
                result.transaction_datetime,
                result.transaction_amount,
                avg,
                avg_rpc
            );

            let block_worker_c = block_completed_worker_block_worker.clone();

            // let block_number = msg.unwrap().block_number;
            block_worker_c.send(result.block_number).unwrap();
        }
    });

    start_workers(
        _block_completed_worker,
        block_parser_watcher,
        counter.clone(),
        connections,
    );

    for i in 0..2 {
        let block_consumer = block_parser_worker.clone();
        block_consumer.send(i).unwrap();
    }

    loop {}
}
