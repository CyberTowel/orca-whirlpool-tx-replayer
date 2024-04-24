mod raydium_saver;
use anchor_client::anchor_lang::accounts::account;
use arl::RateLimiter;
use mpl_token_metadata::processor::print;
use mpl_token_metadata::utils::get_mint_decimals;
use raydium_saver::raydium::{
    batch_process_signatures, get_paginated_singatures, RpcPool, RpcPoolManager,
};
use solana_account_decoder::parse_account_data::{self, parse_account_data, AccountAdditionalData};
use solana_account_decoder::parse_token::get_token_account_mint;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::{
    rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::account_info::AccountInfo;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::time::Duration;

use crate::pool_state::get_pool_state;
use crate::token_db::{DbClientPoolManager, DbPool};
use crate::token_parser::PoolVars;

pub mod pool_state;
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

    let limiter = RateLimiter::new(1000, Duration::from_secs(1));

    let testing_mode = false;

    // values IDK
    // let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    // let signature =
    //     "3yTX6qVC5UZgB8zVa5n4Nh4kQoPPLnXZnTP9bWTaSuGQfj74yZcWtWzeUUk9xf2qVXkyDmw1VhaYvcBa4ZDoyfqu";
    // let pool_coin_token_account = "Ffo9MEhfH5tBBkZMi1vWVpZLqmbDKvEWJhW3XyMQz4QY";
    // let amm_target_orders = "EM9ebwJyrenPmgXQyn9aR5X2tiJssrVPwLSZXxmg2dLy";
    // let token_a_address: &str = "So11111111111111111111111111111111111111112";
    // let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

    // values SOL USDC
    let pool_id = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let signature =
        "rprLhWtWRWvRnmzPPYFVH5PL6aE2iST9sphXoDnze1CDz14GvpYPZFSNKBeieQ4U8PKNfU173pUL8ErJgwjp4ow";
    // let pool_coin_token_account = "DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz";
    // let amm_target_orders = "HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz"; // for this one it's - Pool Pc Token Account
    // let token_a_address: &str = "So11111111111111111111111111111111111111112";
    // let token_b_address: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    let poolvars = PoolVars {
        pool_id: pool_id.to_string(),
        // pool_coin_token_account: pool_coin_token_account.to_string(),
        // amm_target_orders: amm_target_orders.to_string(),
        // token_a_address: token_a_address.to_string(),
        // token_b_address: token_b_address.to_string(),
    };

    let pool_state = get_pool_state(poolvars.pool_id.clone());

    // println!(
    //     "item_to_process length: {}, next_item: {}",
    //     items_to_process.len(),
    //     next_item
    // );

    // return;/

    let mut has_more = true;
    let mut batch = 0;

    let mut before_signature: Option<String> = None;

    while has_more == true {
        batch += 1;

        has_more = false;

        // println!("=======================================================");
        // println!("=======================================================");
        // println!("started processing Batch: {}", batch);
        // println!("=======================================================");
        // println!("=======================================================");

        // let rpc_connection = pool.clone(); //.get().await.unwrap();

        // let signature_pagination_config: GetConfirmedSignaturesForAddress2Config =
        //     GetConfirmedSignaturesForAddress2Config {
        //         commitment: None,
        //         before: before_signature,
        //         limit: Some(1000),
        //         until: None,
        //     };

        // let signatures_to_process = rpc_connection
        //     .get_signatures_for_address_with_config(&pool_pubkey, signature_pagination_config)
        //     .unwrap();

        // if signatures_to_process.len() != 1000 || testing_mode == true {
        //     has_more = false;
        // }

        let mut testing_singatures: Vec<String> = vec![];

        let pool_c = pool.clone();

        let (items_to_process, next_item) =
            get_paginated_singatures(pool_id, pool_c, before_signature).await;

        before_signature = next_item.clone();
        has_more = next_item.is_some();

        testing_singatures.push(signature.to_string());

        // let last_signature =
        //     Some(Signature::from_str(&items_to_process.last().unwrap().signature).unwrap());

        // before_signature = last_signature;

        let signatures_to_use = if testing_mode == true {
            testing_singatures
        } else {
            items_to_process
        };

        // println!("start processing: {:#?}", signatures_to_use.len());

        batch_process_signatures(
            signatures_to_use,
            &pool,
            &limiter,
            &db_pool,
            &pool_state,
            &poolvars,
        )
        .await;

        // println!("start processing");
    }

    // println!("Done");

    // assert_eq!(answer, 42);
}
