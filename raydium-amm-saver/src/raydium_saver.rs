pub mod raydium {

    use arl::RateLimiter;
    use async_trait::async_trait;

    use deadpool::managed::{self, Metrics};

    use deadpool::managed::RecycleResult;

    use solana_client::{
        rpc_client::RpcClient, rpc_response::RpcConfirmedTransactionStatusWithSignature,
    };

    use crate::token_db::DbClientPoolManager;
    use crate::token_parser::PoolVars;
    use crate::transaction_parser::parser_transaction;

    use tokio::task::JoinSet;

    pub async fn batch_process_signatures(
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        rpc_connection: &deadpool::managed::Pool<RpcPoolManager>,
        limiter: &RateLimiter,
        db_pool: &deadpool::managed::Pool<DbClientPoolManager>,
        poolvars: &PoolVars,
    ) {
        let mut signatures_to_process = JoinSet::new();

        for signature in signatures {
            let connection = rpc_connection.clone().get().await.unwrap();

            let tester = limiter.clone();

            let db_client = db_pool.clone().get().await.unwrap();

            let pool_vars_c = poolvars.clone();

            signatures_to_process.spawn(async move {
                // wait for ratelimiting
                tester.wait().await;
                parser_transaction(&signature.signature, &connection, &db_client, &pool_vars_c);
                return signature.signature;
            });
        }

        let mut crawled_signatures: Vec<String> = Vec::new();

        while let Some(res) = signatures_to_process.join_next().await {
            // let idx = res.unwrap();
            let idx = match res {
                Ok(_) => res.unwrap(),
                Err(_) => "".to_string(),
            };
            crawled_signatures.push(idx);
        }
    }

    #[derive(Debug)]
    pub enum Error {}

    pub struct RpcPoolManager {}

    #[async_trait]
    impl managed::Manager for RpcPoolManager {
        type Type = RpcClient;
        type Error = Error;

        async fn create(&self) -> Result<Self::Type, Self::Error> {
            Ok(RpcClient::new_with_commitment(
                // cluster,
                // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
                // "https://rpc.ankr.com/solana/71915acca8127aacb9f83c90556138f82decde6b7a66f5fad32d2e005c26ca8e",
                "http://66.248.205.6:8899",
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

        // if testing.is_err() {
        //     println!("error saving item, {:?}", testing.unwrap());
        // }
    }
}
