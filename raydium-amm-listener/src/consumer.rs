use flume::{Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use transaction_parser::block_parser::parse_block;

use crate::{BlockParsedDebug, ParserConnections};

pub fn start_workers(
    tx_completed: Sender<Option<BlockParsedDebug>>,
    rx: Receiver<u64>,
    counter: Arc<AtomicUsize>,
    connections: ParserConnections,
    worker_amount: usize,
) {
    for i in 0..worker_amount {
        let testing = tx_completed.clone();
        let rx1 = rx.clone();

        let rpc_connection_c = connections.rpc_connection.clone();
        let rpc_connection_builder_c = connections.rpc_connection_builder.clone();
        let db_client_c = connections.db_client.clone();
        let my_cache_c = connections.my_cache.clone();

        let counter_clone = counter.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx1.recv_async().await {
                let dolar = testing.clone();
                // Handle received message
                let counter_value = counter_clone.fetch_add(1, Ordering::SeqCst);

                // tokio::time::sleep(Duration::from_millis(300)).await;
                let (
                    block_number,
                    transaction_amount,
                    duration_rpc,
                    duraction_total,
                    transaction_datetime,
                ) = parse_block(
                    counter_value as u64,
                    &rpc_connection_c,
                    &rpc_connection_builder_c,
                    &db_client_c,
                    &my_cache_c,
                )
                .await;

                // println!(
                //     "Block number: {} transaction amount: {} duration rpc: {:?} duration total: {:?}",
                //     block_number, transaction_amount, duration_rpc, duraction_total
                // );

                let block_parsed_debug = BlockParsedDebug {
                    block_number: block_number as u64,
                    transaction_amount,
                    duration_rpc,
                    duraction_total,
                    transaction_datetime,
                };

                dolar.send(Some(block_parsed_debug)).unwrap();
            }
        });
    }
}
