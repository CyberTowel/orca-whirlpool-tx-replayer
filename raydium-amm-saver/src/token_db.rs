use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use deadpool::managed::RecycleResult;
use deadpool::managed::{self, Metrics, Pool};
use deadpool_postgres::Manager;
use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};
use tokio_postgres::Error as TPError;

use crate::raydium_saver::pg_saving::create_db_pool;

pub fn testing() {
    println!("Hello World");
}

pub type DbPool = managed::Pool<DbClientPoolManager>;

pub struct DbClientPoolManager {}

// pub trait SetDb {
//     type DbPool;
//     fn setDb(&self, db: DbPool);
// }

#[async_trait]
impl managed::Manager for DbClientPoolManager {
    type Type = TokenDbClient;
    type Error = TPError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(TokenDbClient::new(
            // db_pool, // // cluster,
                    // // "https://solana-mainnet.g.alchemy.com/v2/0uuM5dFqqhu79XiFtEa4dZkfLZDlNOGZ",
                    // "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
                    // solana_sdk::commitment_config::CommitmentConfig::confirmed(),
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

// impl SetDb for DbClientPoolManager {
//     type DbPool = DbPool;
//     fn setDb(&self, db: DbPool) {
//         // self = db;
//     }
// }

pub type RpcPool = managed::Pool<TokenDbClient>;

pub struct DbTokenTesterInner {
    // pool: Option<Arc<DbTokenTester>>,
    // runtime: Option<tokio::runtime::Runtime>,
    _sender: Box<dyn RpcSender + Send + Sync>,
}

// pub trait Clone: Sized {
//     // Required method
//     fn clone(&mut self) -> Self;

//     // Provided method
//     fn clone_from(&mut self, source: &Self) {}
// }

pub struct DbTokenMethods {
    // pool: Option<Pool<Manager>>,
    // runtime: Option<tokio::runtime::Runtime>,
    _sender: Arc<DbTokenTesterInner>,
}

pub struct TokenDbClient {
    runtime: Option<tokio::runtime::Runtime>,
    pub sender: DbTokenMethods,
    pub db_pool: Option<Pool<Manager>>,
}

impl Drop for TokenDbClient {
    fn drop(&mut self) {
        self.runtime.take().expect("runtime").shutdown_background();
    }
}

impl DbTokenMethods {
    pub async fn send(&self, _testing: bool) -> Result<bool, TPError> {
        let url = "testing";
        println!("url: {}", url);
        Ok(true)
    }
}

pub struct HttpSender {
    url: String,
}

impl HttpSender {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

pub type DbResult<T> = std::result::Result<T, TPError>;

impl TokenDbClient {
    pub fn new() -> Self {
        let db_connect = create_db_pool();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .thread_name("solRpcClient")
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let sender = DbTokenMethods::new_sender(HttpSender::new("http://localhost:8080"));
        Self {
            runtime: Some(runtime),
            sender,
            db_pool: Some(db_connect),
        }
    }

    pub fn get_usd_price_sol(&self, transaction_datetime: String) -> Result<Decimal, TPError> {
        // let result = self.invoke(self.test_db_inn(testing));
        // let result = self.invoke(self.db_pool.clone())(self.test_db(testing));

        // let db_connect = self.db_pool.clone();

        let result = self.invoke(self.test_db_inn(transaction_datetime));

        return result;

        // println!("result: {:?}", result);
    }

    pub async fn test_db_inn(&self, transaction_datetime: String) -> DbResult<Decimal> {
        // let result = self.sender.send(testing).await;

        // let datetime = "2021-10-01T00:00:00Z";

        // println!("{:?}", transaction_datetime);

        let statement =
            "SELECT * FROM token_prices WHERE token_address = $1 AND conversion_ref = 'USD'
                    order by abs(extract(epoch from (timestamp - $2))) limit 1";

        let rolar: DateTime<Utc> = chrono::DateTime::from_str(&transaction_datetime).unwrap();

        // let dolar = Timestamp::Value(rolar.timestamp() as i64);

        // println!("{:?}", dolar);

        let dolar = self.db_pool.clone();
        let datetime_param: NaiveDateTime = rolar.naive_utc();

        let db_connect = match dolar {
            Some(x) => x,
            None => panic!("No db connection"),
        };

        let client = db_connect.get().await.unwrap();
        let testing = client
            .query(
                statement,
                &[
                    &"So11111111111111111111111111111111111111112",
                    &datetime_param,
                ],
            )
            .await;
        // .unwrap();

        let lipsum = testing.unwrap();

        let row = lipsum.get(0).unwrap();

        let dolar2: Decimal = row.get("value_num");

        println!(
            "value from db {:?} for datetime, ${:?}",
            Some(dolar2),
            rolar
        );

        return Ok(dolar2);
    }

    pub async fn test_sender_inn(&self, testing: bool) -> DbResult<bool> {
        let result = self.sender.send(testing).await;

        return Ok(result.unwrap());
    }

    fn invoke<T, F: std::future::Future<Output = DbResult<T>>>(&self, f: F) -> DbResult<T> {
        // `block_on()` panics if called within an asynchronous execution context. Whereas
        // `block_in_place()` only panics if called from a current_thread runtime, which is the
        // lesser evil.
        tokio::task::block_in_place(move || self.runtime.as_ref().expect("runtime").block_on(f))
    }
}

// pub trait DbTokenTrait {
//     async fn send(&self, testing: bool) -> Result<usize, TPError>;
//     fn url(&self) -> String;
// }

#[async_trait]
pub trait RpcSender {
    fn url(&self) -> String;
}

impl RpcSender for DbTokenMethods {
    fn url(&self) -> String {
        "http://localhost:8080".to_string()
    }
}

impl RpcSender for HttpSender {
    fn url(&self) -> String {
        self.url.clone()
    }
}

impl DbTokenMethods {
    pub fn new_sender<T: RpcSender + Send + Sync + 'static>(sender: T) -> Self {
        Self {
            // pool: None,
            // pool: None,
            _sender: Arc::new(DbTokenTesterInner::new_sender_inner(sender)),
        }
    }

    // pub fn new_sender_inner<T: RpcSender + Send + Sync + 'static>(sender: T) -> Self {
    //     Self {
    //         sender: Box::new(sender),
    //         pool: None,
    //         runtime: None,
    //     }
    // }
}

impl DbTokenTesterInner {
    pub fn new_sender_inner<T: RpcSender + Send + Sync + 'static>(sender: T) -> Self {
        Self {
            _sender: Box::new(sender),
            // pool: None,
        }
    }
}
