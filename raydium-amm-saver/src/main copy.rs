mod raydium_saver;
use std::sync::Arc;

use chrono::prelude::*;
use raydium_saver::raydium::{batch_process_signatures, parse_signature};
use rust_decimal::prelude::*;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use deadpool::managed::{self, Metrics, Object, PoolError, RecycleResult, Timeouts};
use std::{convert::Infallible, time::Duration};

type Pool = managed::Pool<Manager>;

struct Manager {}

impl managed::Manager for Manager {
    type Type = usize;
    type Error = Infallible;

    async fn create(&self) -> Result<usize, Infallible> {
        Ok(0)
    }

    async fn recycle(&self, _conn: &mut usize, _: &Metrics) -> RecycleResult<Infallible> {
        Ok(())
    }
}

#[derive(Debug)]
enum Error {
    // PoolError(PoolError),
    // PostgresError(tokio_postgres::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let rpc_connection = Arc::new(RpcClient::new_with_commitment(
        // cluster,
        "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
        // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    ));

    // let rpc_connection =

    //  March 18, 2024 11:23:37 Central European Standard Time
    let signature =
        "edbGpGBuAaFnjde2KU3YgWE1Vw5Kawz19gXKXtc5bPPE3VMVhaGf2Ba3o6Lc3HyqBZJd6vBKT2dyjhjxBgufBA9"
            .to_string();

    parse_signature(&signature, &rpc_connection.clone());

    let price_ref = "00009260";

    // return Ok(());

    let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    let pub_key = Pubkey::from_str(pool_id).unwrap();

    let mut has_more = true;
    // let mut before_signature: Option<Signature> = None;

    let mut before_signature: Option<Signature> = Some(Signature::from_str(
        "5qcAJQKafzci6oDkjwuCmFn8wxnGYnYr1MpED2xsM5SDbgreM1FU6ypsCokQ3Newkvbxqo8TgyjKJ2UbffjZjVHc",
    )
    .unwrap());

    let max_tries = 2;
    let mut round = 0;

    // while has_more == true && round < max_tries {
    // round += 1;
    let signature_pagination_config: GetConfirmedSignaturesForAddress2Config =
        GetConfirmedSignaturesForAddress2Config {
            commitment: None,
            before: before_signature,
            limit: Some(1000),
            until: None,
        };

    let signatures_to_process = rpc_connection
        .clone()
        .get_signatures_for_address_with_config(&pub_key, signature_pagination_config)
        .unwrap();

    if signatures_to_process.len() != 1000 {
        has_more = false;
    }

    let last_signature =
        Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());

    before_signature = last_signature;

    batch_process_signatures(signatures_to_process, rpc_connection.clone()).await;

    // }

    return Ok(());
}
