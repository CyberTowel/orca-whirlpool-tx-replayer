mod raydium_saver;
use arl::RateLimiter;
use raydium_saver::raydium::{batch_process_signatures, RpcPool, RpcPoolManager};
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::time::Duration;

use crate::token_db::{DbClientPoolManager, DbPool};

pub mod token_db;

#[tokio::main]
async fn main() {
    let mgr = RpcPoolManager {};

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let pool = RpcPool::builder(mgr).max_size(20).build().unwrap();

    let db_pool = DbPool::builder(db_mgr).max_size(20).build().unwrap();

    let limiter = RateLimiter::new(300, Duration::from_secs(1));

    let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    let pub_key = Pubkey::from_str(pool_id).unwrap();

    let  before_signature: Option<Signature> = Some(Signature::from_str(
        "4KfkEVp2QMCM4vEsJgE3fWKuXZmpsv1ema7uBkcHjU4uoM9tVVwuSdPmynx5zJC4mPfirm9mJJCRGT1NRQE2euPA",
    )
    .unwrap());

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
                limit: Some(2),
                until: None,
            };

        let signatures_to_process = rpc_connection
            .get_signatures_for_address_with_config(&pub_key, signature_pagination_config)
            .unwrap();

        if signatures_to_process.len() != 1000 {
            has_more = false;
        }

        println!("Processing {} signatures", signatures_to_process.len());

        let mut testing_singatures: Vec<RpcConfirmedTransactionStatusWithSignature> = vec![];

        testing_singatures.push(RpcConfirmedTransactionStatusWithSignature{
            signature:"5SxbSH6prmvXo3tn8F7fjPGjz3bXivdsLDu9EVEsELzJp7PkmwprvRznEY9wGWwPiJknZAyK5suEP2Cp1dGCtHSR".to_string(), 
            slot: 0,
            err: None,
            memo: None,
            block_time: None,
            confirmation_status: None,
        });
        // testingSingatures.push(signatures_to_process.first().unwrap().clone());
        // testingSingatures.push(signatures_to_process.last().unwrap().clone());

        // let last_signature =
        //     Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());

        // before_signature = last_signature;

        // let token_db_client: TokenDbClient = TokenDbClient::new(db_connect);

        batch_process_signatures(testing_singatures, &pool, &limiter, &db_pool).await;
    }

    println!("Done");

    // assert_eq!(answer, 42);
}
