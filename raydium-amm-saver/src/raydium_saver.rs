use deadpool::managed::Pool;
use solana_client::{
    rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;

use self::raydium::RpcPoolManager;

pub mod raydium {
    use arl::RateLimiter;
    use async_trait::async_trait;
    use std::str::FromStr;

    use deadpool::managed::{self, Metrics, Pool};

    use deadpool::managed::RecycleResult;

    use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
    use solana_client::{
        rpc_client::RpcClient, rpc_response::RpcConfirmedTransactionStatusWithSignature,
    };
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Signature;

    use crate::pool_state::{LiquidityStateLayoutV4, PoolMeta};
    use crate::token_db::DbClientPoolManager;
    use crate::token_parser::PoolVars;
    use crate::transaction_parser::parser_transaction;

    use tokio::task::JoinSet;

    pub async fn batch_process_signatures(
        signatures: Vec<String>,
        rpc_connection: &deadpool::managed::Pool<RpcPoolManager>,
        limiter: &RateLimiter,
        db_pool: &deadpool::managed::Pool<DbClientPoolManager>,
        pool_state: &PoolMeta,
        poolvars: &PoolVars,
    ) {
        let mut signatures_to_process = JoinSet::new();
        // let testing = signatures.clone();

        // let result: Vec<_> = testing.iter().step_by(100).collect();

        for signature in signatures {
            let connection = rpc_connection.clone().get().await.unwrap();

            let tester = limiter.clone();

            let db_client = db_pool.clone().get().await.unwrap();

            let pool_state_c = pool_state.clone();
            let poolvars_c = poolvars.clone();

            signatures_to_process.spawn(async move {
                // wait for ratelimiting
                // tester.wait().await;
                let results = parser_transaction(
                    &signature,
                    &connection,
                    &db_client,
                    &pool_state_c,
                    &poolvars_c,
                );
                return results;
            });
        }

        let mut crawled_signatures: Vec<(String, String, String)> = Vec::new();

        while let Some(res) = signatures_to_process.join_next().await {
            // let idx = res.unwrap();
            let result_i = match res {
                Ok(_) => res.unwrap(),
                Err(_) => ("".to_string(), "".to_string(), "".to_string()),
            };
            crawled_signatures.push(result_i);
        }

        let (signature_from, datatime_from, _err) = crawled_signatures.first().unwrap();
        let (signature_until, datatime_until, _err_until) = crawled_signatures.last().unwrap();
        println!(
            "
========================================
Processed {:?} until {:?} ({:#?})
========================================",
            datatime_from, datatime_until, signature_until,
        );
    }

    #[derive(Debug)]
    pub enum Error {}

    pub struct RpcPoolManager {
        pub rpc_type: Option<String>,
    }

    #[async_trait]
    impl managed::Manager for RpcPoolManager {
        type Type = RpcClient;
        type Error = Error;

        async fn create(&self) -> Result<Self::Type, Self::Error> {
            // println!("Creating new connection {:#?}", self.rpc_type);

            let mut rpc_url = "https://rpc.ankr.com/solana/71915acca8127aacb9f83c90556138f82decde6b7a66f5fad32d2e005c26ca8e";

            if self.rpc_type.is_some() {
                let prop = self.rpc_type.as_ref().unwrap();
                if prop == "dedicated" {
                    rpc_url = "http://66.248.205.6:8899"
                }
            }

            Ok(RpcClient::new_with_commitment(
                // cluster,
                // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
                rpc_url,
                // "http://66.248.205.6:8899",
                // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
                solana_sdk::commitment_config::CommitmentConfig::confirmed(),
            ))
        }

        async fn recycle(
            &self,
            _obj: &mut Self::Type,
            _metrics: &Metrics,
        ) -> RecycleResult<Self::Error> {
            Ok(())
        }
    }

    pub type RpcPool = managed::Pool<RpcPoolManager>;

    pub async fn get_paginated_singatures(
        pool_id: &str,
        pool: Pool<RpcPoolManager>,
        before_signature_param: Option<String>,
        sample_rate: Option<usize>,
    ) -> (Vec<String>, Option<String>) {
        let process_interval = 300;
        // let sample_rate = 30;

        let mut before_signature: Option<Signature> = None;

        if (before_signature_param.is_some()) {
            before_signature = Some(Signature::from_str(&before_signature_param.unwrap()).unwrap());
        }

        // Signature::from_str(before_signature).unwrap();

        //  Some(Signature::from_str(
        //     "4KfkEVp2QMCM4vEsJgE3fWKuXZmpsv1ema7uBkcHjU4uoM9tVVwuSdPmynx5zJC4mPfirm9mJJCRGT1NRQE2euPA",
        // )
        // .unwrap());

        let pool_pubkey = Pubkey::from_str(pool_id).unwrap();

        let has_more = true;

        let mut all_signatures: Vec<String> = Vec::new();

        while has_more == true {
            let signature_pagination_config: GetConfirmedSignaturesForAddress2Config =
                GetConfirmedSignaturesForAddress2Config {
                    commitment: None,
                    before: before_signature,
                    limit: Some(1000),
                    until: None,
                };

            let rpc_connection = pool.clone().get().await.unwrap();

            let signatures_to_process = rpc_connection
                .get_signatures_for_address_with_config(&pool_pubkey, signature_pagination_config)
                .unwrap();

            if (signatures_to_process.last().is_some()) {
                let last_signature = &signatures_to_process.last().unwrap().signature;
                before_signature = Option::Some(Signature::from_str(&last_signature).unwrap());
            }

            if (signatures_to_process.len() == 0) {
                println!("No more signatures to process");
                break;
            }

            let testing: Vec<RpcConfirmedTransactionStatusWithSignature> = signatures_to_process
                .into_iter()
                .filter(|cts: &RpcConfirmedTransactionStatusWithSignature| cts.err.is_none())
                .collect();

            let step_by_value = if sample_rate.is_some() {
                let interval = (testing.len() as f64 / sample_rate.unwrap() as f64) as f64;

                let mut selit = interval.floor() as usize;

                if selit < 1 {
                    selit = 1;
                }
                selit
            } else {
                1
            };

            let dolar: Vec<String> = testing
                .into_iter()
                .step_by(step_by_value)
                .map(|item| item.signature)
                .collect();

            all_signatures.extend(dolar.clone());

            if all_signatures.len() > process_interval + 1 {
                // has_more = false;
                break;
            }

            // let last_signature =
            //     Some(Signature::from_str(&signatures_to_process.last().unwrap().signature).unwrap());
        }

        // let items: Vec<String> = all_signatures.iter().take(101).collect();

        let (item_to_process, b) = all_signatures.split_at(process_interval);

        let next_item = Option::from(b[0].to_string());

        return (item_to_process.to_vec(), next_item);
    }
}

pub mod pg_saving {
    use deadpool::managed::Pool;
    use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod};
    use tokio_postgres::NoTls;

    #[derive(Debug)]
    pub struct PriceDbItem {
        pub price_token_ref: String,
        pub price_usd: String,
        pub datetime: String,
        pub signature: String,
        pub token_a_amount: String,
        pub token_b_amount: String,
        pub pool_address: String,
        pub token_a_address: String,
        pub token_b_address: String,
        pub token_b_price_rel: String,
        // pub token_a_address: String,
        // pub token_b_address: String,
        // pub ubo: String,
        // pub ubo_pool_perc: String,
        // pub ubo_token_a_amount: i64,
        // pub ubo_token_b_amount: i64,
        // pub ubo_token_a_pool_amount: i64,
        // pub ubo_token_b_pool_amount: i64,
    }

    pub fn create_db_pool() -> Pool<Manager> {
        // let mut cfg = Config::new();
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host("65.108.76.168");
        pg_config.port(5432);
        // cfg.host(pg_config.host_path("/run/postgresql"););
        pg_config.user("postgres");
        pg_config.password("2u_XEQuJYvCZnEu5WHZx");
        pg_config.dbname("token_pricing");
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(16).build().unwrap();
        return pool;
    }

    pub async fn _save_price_to_db(_price_item: PriceDbItem, pool: &Pool<Manager>) {
        let _conn = pool.get().await.unwrap();

        // let testing = conn
        //     .execute(
        //         "INSERT INTO token_prices (
        //             price,
        //             datetime,
        //             signature,
        //             token_a_address,
        //             token_b_address,
        //              pool_address,
        //             token_a_amount,
        //         token_b_amount) VALUES ($1,
        //                  $2,
        //                  $3,
        //                  $4,
        //                  $5,
        //                  $6,
        //                  $7,
        //                  $8)",
        //         &[
        //             &price_item.price_usd,
        //             &price_item.datetime,
        //             &price_item.signature,
        //             &price_item.token_a_address,
        //             &price_item.token_b_address,
        //             &price_item.pool_address,
        //             &price_item.token_a_amount,
        //             &price_item.token_b_amount,
        //         ],
        //     )
        //     .await;
    }
}
