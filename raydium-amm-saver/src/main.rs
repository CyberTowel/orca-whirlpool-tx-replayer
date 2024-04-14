mod raydium_saver;
use arl::RateLimiter;
use raydium_saver::raydium::{batch_process_signatures, RpcPool, RpcPoolManager};
use solana_client::{
    rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::time::Duration;

use crate::token_db::{DbClientPoolManager, DbPool};

pub mod token_db;
pub mod token_parser;
pub mod transaction;
pub mod transaction_parser;

#[tokio::main]
async fn main() {
    let mgr = RpcPoolManager {};

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let pool = RpcPool::builder(mgr).max_size(20).build().unwrap();

    let db_pool = DbPool::builder(db_mgr).max_size(20).build().unwrap();

    let limiter = RateLimiter::new(300, Duration::from_secs(1));

    let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    let pub_key = Pubkey::from_str(pool_id).unwrap();

    let mut before_signature: Option<Signature> = None;

    //  Some(Signature::from_str(
    //     "4KfkEVp2QMCM4vEsJgE3fWKuXZmpsv1ema7uBkcHjU4uoM9tVVwuSdPmynx5zJC4mPfirm9mJJCRGT1NRQE2euPA",
    // )
    // .unwrap());

    let mut has_more = true;

    let mut batch = 0;

    while has_more == true {
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

        if signatures_to_process.len() != 1000 {
            has_more = false;
        }

        let mut testing_singatures: Vec<RpcConfirmedTransactionStatusWithSignature> = vec![];

        testing_singatures.push(RpcConfirmedTransactionStatusWithSignature{
            // signature:"4q1vrYF4VNCdWrN8Bj6C6WtEewYG6o1VK7Cgjz3xVTRksNdp7ziUPpYgc7wwVSvAwva9X1PWNwYHVH3Da9tA4z3i".to_string(),
            // signature:"55eFRTsuMk3iBeg9GRnsjsdtjzM7NAiJSSLhped8iFAeMkDEsHGQ6aVm7EmXmqS1Scgq3n2CBJHYebumzzTETK3d".to_string(),
            signature:"3dNwQ6vmUEpSQcVmbjKoBTVx2TAig7J4GuapfeHuKnWq2BqaPAoNA89TVaeSkAx92mdbLQReSkZjn6oZbWWx4qv4".to_string(),
            // signature:"5zx9LQAEAMcyBgKfcKEfQTGGadEFJsvJk832V74K6yXnrGh9WuS5XwFn5LtWNd8xavDzNsPksTGejS4EQEmvpwsn".to_string(),
            // signature:"3oMjtgUyY2JqUWGDutP3hPZYkCrYrLq5XPWbQWo4joVn7R3p9frZW3bgBfJgzhkPtrpSmAjGDNwZLCaAREtVPJ6m".to_string(),
            // signature:"4jrn4BxxcmQyEd7g9ma4qt3wxV5RwjTZybZPPzbsNsePPAd3UVRFBfNLYvEVZVA5k4o7rDts63UKp1jBhsvJSjeg".to_string(),
            // signature:"5SxbSH6prmvXo3tn8F7fjPGjz3bXivdsLDu9EVEsELzJp7PkmwprvRznEY9wGWwPiJknZAyK5suEP2Cp1dGCtHSR".to_string(),
            // signature:"qEtkNJPdoVMcgEJ5Xf3omAWmYpvpUEK5yoG7qVokWBy2f3FskDCRg9VDvkzqzToSjHwqKd377hxj7tNBgVsuy8B".to_string(),
            slot: 0,
            err: None,
            memo: None,
            block_time: None,
            confirmation_status: None,
        });

        // testingSingatures.push(signatures_to_process.first().unwrap().clone());
        // testingSingatures.push(signatures_to_process.last().unwrap().clone());

        let last_signature =
            Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());

        before_signature = last_signature;

        // let token_db_client: TokenDbClient = TokenDbClient::new(db_connect);

        batch_process_signatures(signatures_to_process, &pool, &limiter, &db_pool).await;
    }

    println!("Done");

    // assert_eq!(answer, 42);
}
