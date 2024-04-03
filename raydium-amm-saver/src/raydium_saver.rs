pub mod raydium {

    use arl::RateLimiter;
    use async_trait::async_trait;
    use chrono::prelude::*;
    use deadpool::managed::{self, Metrics, Pool};
    use deadpool_postgres::Manager;
    use num::bigint::BigInt;
    use num::rational::{BigRational, Ratio};
    use num::{FromPrimitive, Rational};
    use num_bigfloat::{BigFloat, RoundingMode};
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
    use crate::token_db::{self, DbClientPoolManager, TokenDbClient};

    use tokio::task::JoinSet;

    pub async fn batch_process_signatures(
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        rpc_connection: &deadpool::managed::Pool<RpcPoolManager>,
        limiter: &RateLimiter,
        db_pool: &deadpool::managed::Pool<DbClientPoolManager>,
        // token_db_client: &mut TokenDbClient,
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

        // let result = db_connection
        //     .get_price_sol_usd("2021-10-01T00:00:00Z")
        //     .unwrap();

        // println!("testing result from fn , {:#?}", result);

        for signature in signatures {
            let connection = rpc_connection.clone().get().await.unwrap();
            // let db_connect_test = db_connection.get_price_sol_usd();
            // let testing = db_connection; //.clone();

            // let testing = db_connection.db_client.clone(); //.get_price().await.unwrap();

            // println!("Testing: {:?}", testing);

            // let dolarTest = token_db_client.db_pool.clone();

            // let dolar_testt = token_db_client.clone();

            let tester = limiter.clone();

            let db_client = db_pool.clone().get().await.unwrap();

            // let selit = token_db_client.clone();
            signatures_to_process.spawn(async move {
                // token_db_client.test_db(true);
                tester.wait().await;

                // token_db_.test_db(false);
                // token_db_client.sender.send(true);
                // token_db_client.test_db(true);

                // let db = dolarTest.get().await.unwrap();

                // let result = db_connection
                //     .get_price_sol_usd("2021-10-01T00:00:00Z")
                //     .unwrap();

                // println!("processing signature: {}", signature.signature);
                // let dolar = db_connection.bartest();
                // println!("processing signature: {}", signature.signature);
                let testing = parse_signature(&signature.signature, &connection, &db_client);
                // println!("saving signature: {:#?}", testing);
                // save_price(testing, &db_connect_test).await;
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

    pub fn parse_signature(
        signature: &String,
        rpc_connection: &RpcClient,
        db_client: &TokenDbClient,
        // db_client: Arc<DbClient>, // db_connection: &DbConnectionClient,
    ) -> PriceDbItem {
        // println!(
        //     "======================================= signature: {} ========================================",
        //     signature.signature
        // );

        const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
        let token_a_address: &str = "So11111111111111111111111111111111111111112";
        let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

        let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: Some(1),
        };

        let transaction = rpc_connection
            .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config);

        let transaction_datetime = transaction.as_ref().unwrap().block_time.unwrap();

        println!(
            "======================================= signature: {}, time: {} ========================================",
            signature,
            DateTime::from_timestamp(transaction_datetime, 0).unwrap()
        );

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
                x.mint == token_a_address && owner == Some(RAYDIUM_AUTHORITY.to_string())
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let amount_token_b_test = &amount_token_a
            .iter()
            .find(|&x| {
                let owner: Option<String> = x.owner.clone().into();
                x.mint == token_b_address && owner == Some(RAYDIUM_AUTHORITY.to_string())
                // && Some(x.owner.into()) == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
            })
            .unwrap()
            .ui_token_amount
            .amount;
        let dolar_selit = DateTime::from_timestamp(transaction_datetime, 0)
            .unwrap()
            .to_rfc3339();

        let usd_price_nearest = db_client.get_usd_price_sol(dolar_selit).unwrap();

        let price = get_price(
            amount_token_a_test,
            amount_token_b_test,
            &usd_price_nearest.to_string(),
        );

        let (price_usd, price_token_ref) = price.unwrap();

        let item: PriceDbItem = PriceDbItem {
            price_token_ref: price_token_ref.to_string(),
            price_usd: price_usd.to_string(),
            datetime: DateTime::from_timestamp(transaction_datetime, 0)
                .unwrap()
                .to_rfc3339(),
            signature: signature.to_string(),
            token_a_amount: amount_token_a_test.to_string(),
            token_b_amount: amount_token_b_test.to_string(),
            pool_address: "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
            token_a_address: token_a_address.to_string(),
            token_b_address: token_b_address.to_string(),
        };

        println!("{:#?}", item);

        return item;
    }

    fn get_price(
        token_a_balance: &String,
        token_b_balance: &String,
        usd_price_nearest: &String,
    ) -> Result<(f64, f64), Error> {
        // let token_a_price = Decimal::from_str(&token_a_balance.to_string()).unwrap();
        // let token_b_price = Decimal::from_str(&token_b_balance.to_string()).unwrap();

        // let price2 = token_a_price
        //     .checked_div(token_b_price)
        //     .unwrap()
        //     .checked_mul(Decimal::TEN.powi((-6) as i64))
        //     .unwrap();

        // let balance_a = BigInt::from_str(token_a_balance)
        //     .unwrap()
        //     .checked_mul(&BigInt::from(10).pow(36))
        //     .unwrap();
        // // .to_string();

        // let balance_b = BigInt::from_str(token_b_balance)
        //     .unwrap()
        //     .checked_mul(&BigInt::from(10).pow(18))
        //     .unwrap();
        // .to_string();

        // let price3 = balance_a.checked_div(&balance_b).unwrap(); //.to_string();

        // let price4 = BigFloat::from_str(token_a_balance).unwrap()
        //     * (BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18))));

        // // let test5 = BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(18))).to_string();

        let price_token_a_bf = BigFloat::from_str(token_a_balance).unwrap()
            / BigFloat::from_str(token_b_balance).unwrap();

        // let test6 = test5 * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-6)));

        let usd_price_token_b = price_token_a_bf * BigFloat::from_str(usd_price_nearest).unwrap();

        let usd_price_token_dec =
            usd_price_token_b * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-6 + -18)));

        let price_token_usd = usd_price_token_dec.round(32, RoundingMode::ToOdd).to_f64();

        let price_token_ref = (price_token_a_bf
            * BigFloat::from(BigFloat::from(10).pow(&BigFloat::from(-6))))
        .to_f64();

        println!(
            "\nprice_token_ref: {}
            \nprice_token_usd: {}
            \n===========",
            price_token_usd, price_token_ref
        );

        Ok((price_token_usd, price_token_ref))
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
"https://rpc.ankr.com/solana/71915acca8127aacb9f83c90556138f82decde6b7a66f5fad32d2e005c26ca8e",
                // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
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
        pub price_token_ref: String,
        pub price_usd: String,
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
        pg_config.host("65.108.76.168");
        pg_config.port(6432);
        // cfg.host(pg_config.host_path("/run/postgresql"););
        pg_config.user("postgres");
        pg_config.password("JD*kFWVQ3ZK4f9Q");
        pg_config.dbname("token_pricing");
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
                            &testing.price_usd,
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
                    &price_item.price_usd,
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

// pub mod pg_token_data {
//     use async_trait::async_trait;
//     use chrono::{DateTime, NaiveDateTime, Utc};
//     use deadpool::managed::Pool;
//     use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod};
//     use rust_decimal::prelude::*;
//     use std::sync::Arc;
//     use tokio_postgres::{types::Timestamp, Error, NoTls};

//     pub struct DbConnectionClient {
//         runtime: Option<tokio::runtime::Runtime>,
//         // pool: Option<Pool<Manager>>,
//         pub db_client: Arc<DbClient>, // pub test: TestingStruct,
//     }

//     pub struct DbClient {
//         pool: Pool<Manager>,
//         // runtime: Option<tokio::runtime::Runtime>,
//     }

//     pub struct DbNew {
//         rpc_client: Arc<InnerClient>,
//         runtime: Option<tokio::runtime::Runtime>,
//     }

//     impl Drop for DbNew {
//         fn drop(&mut self) {
//             self.runtime.take().expect("runtime").shutdown_background();
//         }
//     }

//     #[async_trait]
//     pub trait DbTester {
//         // type Type = DbConnectionClient;
//         // type Error = Error;

//         fn get_price(&self) -> DbResult<Decimal>;
//     }

//     pub struct DbConfig implements Pool<RpcPoolManager> {
//         pub testing: bool,
//     }

//     impl DbNew {
//         pub fn new_sender<T: DbTester + Send + Sync + 'static>(
//             sender: T,
//             config: DbConfig,
//         ) -> Self {
//             Self {
//                 rpc_client: Arc::new(InnerClient::new_sender(sender, config)),
//                 runtime: Some(
//                     tokio::runtime::Builder::new_current_thread()
//                         .thread_name("solRpcClient")
//                         .enable_io()
//                         .enable_time()
//                         .build()
//                         .unwrap(),
//                 ),
//             }
//         }
//     }

//     impl DbClient {
//         pub async fn get_price(&self) -> DbResult<Decimal> {
//             println!("testing from f 3");
//             return Ok(Decimal::from_str("1").unwrap());
//         }

//         // pub fn get_price_sol_usd(&self, datetime: &str) -> DbResult<Decimal> {
//         //     println!("testing from f 2");
//         //     self.invoke::<Decimal, _>(self.get_price_sol_usd_inn(datetime))
//         // }
//     }

//     struct InnerClient {
//         sender: Box<dyn DbTester + Send + Sync + 'static>,
//         config: DbConfig,
//     }

//     impl InnerClient {
//         pub fn new_sender<T: DbTester + Send + Sync + 'static>(
//             sender: T,
//             config: DbConfig,
//         ) -> Self {
//             Self {
//                 sender: Box::new(sender),
//                 config,
//             }
//         }
//     }

//     // #[derive(Clone)]
//     // pub struct TestingStruct {
//     //     pub get_price: Arc<dyn Fn() -> DbResult<Decimal>>,
//     // }

//     // async fn fetch_price() -> Box<dyn Fn() -> Arc<i32>> {
//     //     println!("fetching price from async testing");

//     //     return Box::new(|| {
//     //         println!("testing from f 3");
//     //         Arc::new(1)
//     //     });
//     // }

//     pub type DbResult<T> = std::result::Result<T, Error>;

//     // impl DbConnectionClient {
//     //     pub fn new() -> Self {
//     //         // fn create_db_pool() -> Pool<Manager> {
//     //         // let mut cfg = Config::new();
//     //         let mut pg_config = tokio_postgres::Config::new();
//     //         pg_config.host("65.108.76.168");
//     //         pg_config.port(6432);
//     //         // cfg.host(pg_config.host_path("/run/postgresql"););
//     //         pg_config.user("postgres");
//     //         pg_config.password("JD*kFWVQ3ZK4f9Q");
//     //         pg_config.dbname("token_pricing");
//     //         let mgr_config = ManagerConfig {
//     //             recycling_method: RecyclingMethod::Fast,
//     //         };
//     //         let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
//     //         let pool = Pool::builder(mgr).max_size(16).build().unwrap();
//     //         // return pool;
//     //         // }

//     //         let runtime = tokio::runtime::Builder::new_current_thread()
//     //             .thread_name("dbConnect")
//     //             .enable_io()
//     //             .enable_time()
//     //             .build()
//     //             .unwrap();

//     //         // let db_connectino = create_db_pool();

//     //         let runtime = Some(runtime);

//     //         DbConnectionClient {
//     //             // runtime: runtime,
//     //             // pool: Some(pool),
//     //             db_client: Arc::new(DbClient { pool: pool }),
//     //             runtime: runtime,
//     //             // test: testingStruct,
//     //         }
//     //     }

//     //     fn invoke<T, F: std::future::Future<Output = DbResult<T>>>(&self, f: F) -> DbResult<T> {
//     //         // `block_on()` panics if called within an asynchronous execution context. Whereas
//     //         // `block_in_place()` only panics if called from a current_thread runtime, which is the
//     //         // lesser evil.
//     //         tokio::task::block_in_place(move || self.runtime.as_ref().expect("runtime").block_on(f))
//     //     }

//     //     pub fn get_price_sol_usd(&self, datetime: &str) -> DbResult<Decimal> {
//     //         println!("testing from f 2");
//     //         self.invoke::<Decimal, _>(self.get_price_sol_usd_inn(datetime))
//     //     }

//     //     async fn get_price_sol_usd_inn(&self, datetime: &str) -> DbResult<Decimal> {
//     //         println!("getting price from async");

//     //         let rolar: DateTime<Utc> = chrono::DateTime::from_str(datetime).unwrap();

//     //         let lipsum: NaiveDateTime = rolar.naive_utc();

//     //         println!("testing from f 4, {:#?}", rolar.to_string());

//     //         let statement =
//     //             "SELECT * FROM token_prices WHERE token_address = $1 AND conversion_ref = 'USD'
//     //         order by abs(extract(epoch from (timestamp - $2))) limit 1";

//     //         let dolar = Timestamp::Value(rolar.timestamp() as i64);

//     //         println!("{:?}", dolar);

//     //         // let dolar = self.db_client.pool.clone();
//     //         let dolar = self.pool.clone();

//     //         let client = dolar.get().await.unwrap();
//     //         let testing = client
//     //             .query(
//     //                 statement,
//     //                 &[&"So11111111111111111111111111111111111111112", &lipsum],
//     //             )
//     //             .await;
//     //         // .unwrap();

//     //         if (testing.is_err()) {
//     //             println!("testing, {:#?}", testing.as_ref().unwrap_err());
//     //             return Err(testing.unwrap_err());
//     //         }

//     //         let lipsum = testing.unwrap();

//     //         let info = lipsum.get(0).unwrap();

//     //         let dolar2: Decimal = info.get("value_num");

//     //         println!("testing from row , {:#?}", dolar2);
//     //         return Ok(dolar2);
//     //     }

//     //     // pub fn send_and_confirm_transaction(
//     //     //     &self,
//     //     //     transaction: &impl SerializableTransaction,
//     //     // ) -> ClientResult<Signature> {
//     //     //     self.invoke((self.rpc_client.as_ref()).send_and_confirm_transaction(transaction))
//     //     // }
//     // }

//     // fn get_price_sol_usd(&self, db_connection: &Pool<Manager>) -> () {
//     //     println!("getting price");

//     //     // tokio::task::block_in_place(move || self.runtime.as_ref().expect("runtime").block_on(f))

//     //     tokio::task::block_in_place(move || self.runtime.as_ref().expect("runtime").block_on(f))

//     //     // let rt = tokio::runtime::Builder::new_current_thread()
//     //     //     .enable_all()
//     //     //     .build()
//     //     //     .unwrap();

//     //     // let res = rt.block_on(async {
//     //     //     let client = db_connection.get().await.unwrap();
//     //     //     let testing = client
//     //     //         .query("SELECT * FROM token_prices limit 1", &[])
//     //     //         .await
//     //     //         .unwrap();

//     //     //     println!("testing, {:#?}", testing);
//     //     // });
//     // }
// }
