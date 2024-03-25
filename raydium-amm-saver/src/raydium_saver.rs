pub mod raydium {

    use arl::RateLimiter;
    use async_trait::async_trait;
    use chrono::prelude::*;
    use deadpool::managed::{self, Metrics, Pool};
    use deadpool_postgres::Manager;
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::Duration;
    // use deadpool::managed::{Manager, Object, Pool};
    use deadpool::managed::{RecycleError, RecycleResult};
    use rust_decimal::prelude::*;
    use solana_client::{
        rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
        rpc_response::RpcConfirmedTransactionStatusWithSignature,
    };
    use solana_sdk::signature::Signature;
    use solana_transaction_status::{UiTransactionEncoding, UiTransactionTokenBalance};

    use crate::raydium_saver::pg_saving::{save_price, PriceDbItem};

    use tokio::task::JoinSet;

    pub async fn batch_process_signatures(
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        rpc_connection: &deadpool::managed::Pool<RpcPoolManager>,
        limiter: &RateLimiter,
        db_connection: Pool<Manager>, // rpc_pool_manager: Pool<dyn Manager<Type = RpcClient, Error = Error>>,
    ) {
        // let _ = rpc_pool_manager;
        // println!("total items : {}", signatures.len());
        // println!(
        //     "first item : {:?}, time; {:?}",
        //     signatures.first().unwrap().signature,
        //     DateTime::from_timestamp(signatures.first().unwrap().block_time.unwrap(), 0)
        // );

        // println!(
        //     "first item : {:?}, time; {:?}",
        //     signatures.last().unwrap().signature,
        //     DateTime::from_timestamp(signatures.last().unwrap().block_time.unwrap(), 0)
        // );
        let mut signatures_to_process = JoinSet::new();

        for signature in signatures {
            let connection = rpc_connection.clone().get().await.unwrap();
            let db_connect_test = db_connection.clone();
            let tester = limiter.clone();
            signatures_to_process.spawn(async move {
                tester.wait().await;
                //
                let testing = parse_signature(&signature.signature, &connection);
                save_price(testing, &db_connect_test).await;
                return signature.signature;
            });
        }

        let mut crawled_signatures: Vec<String> = Vec::new();

        while let Some(res) = signatures_to_process.join_next().await {
            let idx = res.unwrap();
            crawled_signatures.push(idx);
        }

        // println!("saved, {} items", crawled_signatures.len());
    }

    pub fn parse_signature(signature: &String, rpc_connection: &RpcClient) -> PriceDbItem {
        // println!(
        //     "======================================= signature: {} ========================================",
        //     signature.signature
        // );

        let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: Some(1),
        };

        let transaction = rpc_connection
            .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

        let testing_blocktime = transaction.as_ref().unwrap().block_time.unwrap();

        let dolar: Option<Vec<UiTransactionTokenBalance>> = transaction
            .unwrap()
            .transaction
            .meta
            .unwrap()
            .post_token_balances
            .into();

        let amount_token_a = &dolar.clone().unwrap();

        let amount_token_a_test = &amount_token_a
            .iter()
            .find(|&x| {
                let owner: Option<String> = x.owner.clone().into();
                x.mint == "So11111111111111111111111111111111111111112"
                    && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let amount_token_b_test = &amount_token_a
            .iter()
            .find(|&x| {
                let owner: Option<String> = x.owner.clone().into();
                x.mint == "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J"
                    && owner == Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string())
                // && Some(x.owner.into()) == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let price = get_price(amount_token_a_test, amount_token_b_test);

        println!(
            "price: {}, time: {}, signature: {}, token_a: {}, token_b: {}",
            price,
            DateTime::from_timestamp(testing_blocktime, 0).unwrap(),
            signature,
            amount_token_a_test,
            amount_token_b_test,
        );

        let item: PriceDbItem = PriceDbItem {
            price: price.to_string(),
            datetime: "testing".to_string(),
            signature: signature.to_string(),
            token_a_amount: amount_token_a_test.to_string(),
            token_b_amount: amount_token_b_test.to_string(),
            pool_address: "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
            token_a_address: "S".to_string(),
            token_b_address: "".to_string(),
        };

        return item;
    }

    fn get_price(token_a_balance: &String, token_b_balance: &String) -> Decimal {
        let token_a_price = Decimal::from_str(&token_a_balance.to_string()).unwrap();
        let token_b_price = Decimal::from_str(&token_b_balance.to_string()).unwrap();

        let price2 = token_a_price
            .checked_div(token_b_price)
            .unwrap()
            .checked_mul(Decimal::TEN.powi((-6) as i64))
            .unwrap();

        price2
    }

    #[derive(Debug)]
    pub enum Error {
        Fail,
    }

    // pub struct Limiter {
    //     rate_limiter: RateLimiter,
    // }

    // impl Limiter {
    //     pub fn new(rate: usize, duration: Duration) -> Self {
    //         limiter {
    //             rate_limiter: RateLimiter::new(rate, duration),
    //         }
    //     }

    //     pub async fn wait(&self) {
    //         self.rate_limiter.wait().await;
    //     }
    // }

    // pub type Limiter = Limiter::new(10, Duration::from_secs(2));

    pub struct RpcPoolManager {}

    #[async_trait]
    impl managed::Manager for RpcPoolManager {
        type Type = RpcClient;
        type Error = Error;

        async fn create(&self) -> Result<Self::Type, Self::Error> {
            Ok(RpcClient::new_with_commitment(
                // cluster,
                // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
                "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
                solana_sdk::commitment_config::CommitmentConfig::confirmed(),
            ))
        }

        async fn recycle(
            &self,
            obj: &mut Self::Type,
            metrics: &Metrics,
        ) -> RecycleResult<Self::Error> {
            Ok(())
        }
    }

    pub type RpcPool = managed::Pool<RpcPoolManager>;
}

pub mod pg_saving {
    use deadpool::managed::Pool;
    use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod};
    use tokio::task::JoinSet;
    use tokio_postgres::{Error, NoTls};

    #[derive(Debug)]
    pub struct PriceDbItem {
        pub price: String,
        pub datetime: String,
        pub signature: String,
        pub token_a_amount: String,
        pub token_b_amount: String,
        pub pool_address: String,
        pub token_a_address: String,
        pub token_b_address: String,
    }

    pub fn create_db_pool() -> Pool<Manager> {
        // let mut cfg = Config::new();
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host("static.236.19.181.135.clients.your-server.de");
        // cfg.host(pg_config.host_path("/run/postgresql"););
        pg_config.user("postgres");
        pg_config.password("JD*kFWVQ3ZK4f9Q");
        pg_config.dbname("sol_whirlpool_execs");
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(16).build().unwrap();
        return pool;
    }

    async fn batch_save_price(transactions_to_save: Vec<PriceDbItem>) {
        let pool = create_db_pool();

        let mut prices_to_save = JoinSet::new();

        for testing in transactions_to_save {
            let tester = pool.clone();
            prices_to_save.spawn(async move {
                let conn = tester.get().await.unwrap();
                let _ = conn
                    .execute(
                        "INSERT INTO token_prices (
                            price, 
                            datetime, 
                            signature, 
                            token_a_address,
                            token_b_address,
                             pool_address, 
                            token_a_amount, 
                        token_b_amount) VALUES ($1,
                                 $2,
                                 $3,
                                 $4,
                                 $5,
                                 $6,
                                 $7,
                                 $8)",
                        &[
                            &testing.price,
                            &testing.datetime,
                            &testing.signature,
                            &testing.token_a_address,
                            &testing.token_b_address,
                            &testing.pool_address,
                            &testing.token_a_amount,
                            &testing.token_b_amount,
                        ],
                    )
                    .await;
            });
        }

        // let mut saved_prices_ts: Vec<String> = Vec::new();

        while let Some(res) = prices_to_save.join_next().await {
            let idx = res.unwrap();
            // saved_prices_ts.push(idx);
        }

        // println!("saved, {} items", saved_prices_ts.len());
    }

    pub async fn save_price(price_item: PriceDbItem, pool: &Pool<Manager>) {
        let conn = pool.get().await.unwrap();

        let testing = conn
            .execute(
                "INSERT INTO token_prices (
                    price, 
                    datetime, 
                    signature, 
                    token_a_address,
                    token_b_address,
                     pool_address, 
                    token_a_amount, 
                token_b_amount) VALUES ($1,
                         $2,
                         $3,
                         $4,
                         $5,
                         $6,
                         $7,
                         $8)",
                &[
                    &price_item.price,
                    &price_item.datetime,
                    &price_item.signature,
                    &price_item.token_a_address,
                    &price_item.token_b_address,
                    &price_item.pool_address,
                    &price_item.token_a_amount,
                    &price_item.token_b_amount,
                ],
            )
            .await;

        if (testing.is_err()) {
            println!("error saving item, {:?}", testing.unwrap());
        }
    }
}
