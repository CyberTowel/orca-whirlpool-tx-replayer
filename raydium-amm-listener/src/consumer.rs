use block_parser::block_parser::parse_block;
use block_parser::interfaces::ParserConnections;
use flume::{Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::helpers::retry_blocks;
use crate::BlockParsedDebug;

pub fn start_workers(
    tx_completed: Sender<Option<BlockParsedDebug>>,
    rx: Receiver<u64>,
    counter: Arc<AtomicUsize>,
    connections: ParserConnections,
    worker_amount: usize,
) {
    for _i in 0..worker_amount {
        let testing = tx_completed.clone();
        let rx1 = rx.clone();

        let rpc_connection_c = connections.rpc_connection.clone();
        let rpc_connection_builder_c = connections.rpc_connection_builder.clone();
        let db_client_c = connections.db_client.clone();
        let my_cache_c = connections.my_cache.clone();

        let counter_clone = counter.clone();
        tokio::spawn(async move {
            while let Ok(_msg) = rx1.recv_async().await {
                let dolar = testing.clone();
                // Handle received message
                let counter_value = counter_clone.fetch_add(1, Ordering::SeqCst);

                let result = retry_blocks(|| async {
                    parse_block(
                        counter_value as u64,
                        &rpc_connection_c,
                        &rpc_connection_builder_c,
                        &db_client_c,
                        &my_cache_c,
                    )
                    .await
                })
                .await;

                // // tokio::time::sleep(Duration::from_millis(300)).await;
                // let result = parse_block(
                //     counter_value as u64,
                //     &rpc_connection_c,
                //     &rpc_connection_builder_c,
                //     &db_client_c,
                //     &my_cache_c,
                // )
                // .await;

                if result.is_ok() {
                    let (
                        block_number,
                        transaction_amount,
                        duration_rpc,
                        duraction_total,
                        transaction_datetime,
                        sol_price_db,
                    ) = result.unwrap();

                    let block_parsed_debug = BlockParsedDebug {
                        block_number: block_number as u64,
                        transaction_amount,
                        duration_rpc,
                        duraction_total,
                        transaction_datetime,
                        error: None,
                        sol_price_db,
                    };
                    dolar.send(Some(block_parsed_debug)).unwrap();
                } else {
                    dolar
                        .send(Some(BlockParsedDebug {
                            block_number: counter_value as u64,
                            transaction_amount: 0,
                            duration_rpc: Duration::new(0, 0),
                            duraction_total: Duration::new(0, 0),
                            transaction_datetime: "".to_string(),
                            error: Some("error loading block rpc after 20 retries".to_string()),
                            sol_price_db: "".to_string(),
                        }))
                        .unwrap();
                }
            }
        });
    }
}
