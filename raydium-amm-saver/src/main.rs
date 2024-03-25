mod raydium_saver;
use arl::RateLimiter;
use async_trait::async_trait;
use deadpool::managed::{self, Manager, Metrics};
use deadpool::managed::{RecycleError, RecycleResult};
use raydium_saver::pg_saving::create_db_pool;
use raydium_saver::raydium::{batch_process_signatures, parse_signature, RpcPool, RpcPoolManager};
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mgr = RpcPoolManager {};
    let pool = RpcPool::builder(mgr).max_size(20).build().unwrap();

    let limiter = RateLimiter::new(15, Duration::from_secs(1));
    // let rpc_connection = pool.get().await.unwrap();

    // let signature =
    //     "edbGpGBuAaFnjde2KU3YgWE1Vw5Kawz19gXKXtc5bPPE3VMVhaGf2Ba3o6Lc3HyqBZJd6vBKT2dyjhjxBgufBA9"
    //         .to_string();

    // // let answer = con.get_answer().await;
    // parse_signature(&signature, &rpc_connection);

    let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    let pub_key = Pubkey::from_str(pool_id).unwrap();

    let mut before_signature: Option<Signature> = None;

    // Some(Signature::from_str(
    //     "5qcAJQKafzci6oDkjwuCmFn8wxnGYnYr1MpED2xsM5SDbgreM1FU6ypsCokQ3Newkvbxqo8TgyjKJ2UbffjZjVHc",
    // )
    // .unwrap());

    let mut has_more = true;

    // let rpc_connection = pool.clone().get().await.unwrap();

    let mut batch = 0;

    while (has_more == true) {
        batch += 1;

        println!("=======================================================");
        println!("=======================================================");
        println!("started processing Batch: {}", batch);
        println!("=======================================================");
        println!("=======================================================");
        let rpc_connection = pool.clone().get().await.unwrap();

        let signature_pagination_config: GetConfirmedSignaturesForAddress2Config =
            GetConfirmedSignaturesForAddress2Config {
                commitment: None,
                before: before_signature,
                limit: Some(1000),
                until: None,
            };

        let signatures_to_process = rpc_connection
            .get_signatures_for_address_with_config(&pub_key, signature_pagination_config)
            .unwrap();

        if (signatures_to_process.len() != 1000) {
            has_more = false;
        }

        let last_signature =
            Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());

        before_signature = last_signature;

        let db_connection = create_db_pool();

        batch_process_signatures(signatures_to_process, &pool, &limiter, db_connection).await;
    }

    println!("Done");

    // assert_eq!(answer, 42);
}
