use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use deadpool::managed::RecycleResult;
use deadpool::managed::{self, Metrics, Pool};
use deadpool_postgres::Manager;
use num::FromPrimitive;
use pg_bigdecimal::{BigDecimal, PgNumeric};
use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};
use tokio_postgres::types::Json;
use tokio_postgres::Error as TPError;
// use tokio_postgres::types::Json;

use crate::raydium_saver::pg_saving::create_db_pool;
use crate::token_parser::{TokenAmounts, TokenAmountsPricedToArchive, TokenPriceOracleValues};

pub fn testing() {}

pub type DbPool = managed::Pool<DbClientPoolManager>;

pub struct DbClientPoolManager {}

#[derive(Debug, Clone)]
pub struct PriceItem {
    pub signature: String,
    pub token_ref_address: String,
    pub token_address: String,
    pub token_price_usd: String,
    pub token_ref_price_usd: String,
    pub token_price_usd_formatted: String,
    pub token_ref_price_usd_formatted: String,
    pub datetime: String,
    pub signer: String,
    pub ubo: String,
    pub pool_address: String,
    pub usd_total_pool: String,
    pub price_token_ref: String,
    pub price_token_ref_formatted: String,
    pub block_number: String,
    // pub token_a_usd: TokenAmountsPriced,
    // pub token_b_usd: TokenAmountsPriced,
    // pub token_amounts_a: TokenAmounts,
    // pub token_amounts_b: TokenAmounts,
}

#[derive(Debug, Clone)]
pub struct PriceItemDb {
    pub conversion_ref: String,
    pub token_address: String,
    pub price: String,
    pub datetime: String,
    pub transaction_hash: String,
    pub price_formatted: String,
    pub oracle_id: String,
    pub blocknumber: String,
}

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

        let result = self.invoke(self.get_usd_price_sol_inn(transaction_datetime));

        return result;

        // println!("result: {:?}", result);
    }

    pub fn save_token_values(&self, input: PriceItem) -> Result<(), TPError> {
        let result = self.invoke(self.save_token_values_inn(&input));

        return result;

        // println!("result: {:?}", result);
    }

    pub fn insert_token_usd_values(
        &self,
        signature: &str,
        input: &TokenPriceOracleValues,
    ) -> Result<(), TPError> {
        let result = self.invoke(self.insert_token_usd_values_inn(signature, input));

        return result;

        // println!("result: {:?}", result);
    }

    pub fn insert_token_amounts(
        &self,
        signature: &str,
        input: &TokenPriceOracleValues,
    ) -> Result<(), TPError> {
        let result = self.invoke(self.insert_token_amounts_inn(signature, input));

        return result;

        // println!("result: {:?}", result);
    }

    pub async fn insert_token_amounts_inn(
        &self,
        signature: &str,
        input: &TokenPriceOracleValues,
    ) -> Result<(), TPError> {
        let dolar = self.db_pool.clone();

        let db_connect = match dolar {
            Some(x) => x,
            None => panic!("No db connection"),
        };

        let client = db_connect.get().await.unwrap();

        let stmt = client
            .prepare_cached(
                "INSERT INTO token_prices_token_amounts (
            signature, 
            token_address,
            amount_total_pool,
            amount_diff_pool,
            amount_total_ubo,
            amount_diff_ubo
            ) VALUES ($1::TEXT, 
                $2::TEXT,  
                $3::NUMERIC,       
                $4::NUMERIC, 
                $5::NUMERIC,
                $6::NUMERIC
            ) ON CONFLICT ON CONSTRAINT token_prices_token_amounts_pkey DO Update
            SET token_address = excluded.token_address,
            amount_total_pool = excluded.amount_total_pool ,
            amount_diff_pool = excluded.amount_diff_pool ,
            amount_total_ubo = excluded.amount_total_ubo ,
            amount_diff_ubo = excluded.amount_diff_ubo
                ",
            )
            .await
            .unwrap();

        client
            .query(
                &stmt,
                &[
                    &signature,
                    &input.token_address,
                    &Decimal::from_f64_retain(input.amount_total_pool).unwrap(),
                    &Decimal::from_f64_retain(input.amount_diff_pool).unwrap(),
                    &Decimal::from_f64_retain(input.amount_total_ubo).unwrap(),
                    &Decimal::from_f64_retain(input.amount_diff_ubo).unwrap(),
                ],
            )
            .await
            .unwrap();

        return Ok(());
    }

    pub async fn insert_token_usd_values_inn(
        &self,
        signature: &str,
        input: &TokenPriceOracleValues,
    ) -> Result<(), TPError> {
        let dolar = self.db_pool.clone();

        let db_connect = match dolar {
            Some(x) => x,
            None => panic!("No db connection"),
        };

        let client = db_connect.get().await.unwrap();

        let stmt = client
            .prepare_cached(
                "INSERT INTO token_prices_oracle_values (
            ubo,
            signer,
            pool_address,
            token_address,
            transaction_hash, 
            usd_total_pool,
            usd_total_ubo,
            usd_diff_ubo,
            usd_diff_pool,
            amount_total_pool,
            amount_diff_pool, 
            amount_total_ubo, 
            amount_diff_ubo
            ) VALUES ($1::TEXT, 
                $2::TEXT,  
                $3::TEXT,  
                $4::TEXT,  
                $5::TEXT,  
                $6::TEXT,       
                $7::TEXT, 
                $8::TEXT,
                $9::TEXT, 
                $10::NUMERIC,
                $11::NUMERIC, 
                $12::NUMERIC,
                $13::NUMERIC
            ) ON CONFLICT ON CONSTRAINT token_prices_oracle_values_pkey DO Update
            SET 
            ubo = excluded.ubo,
            signer = excluded.signer,
            pool_address = excluded.pool_address,
            usd_total_pool = excluded.usd_total_pool,
            usd_total_ubo = excluded.usd_total_ubo,
            usd_diff_ubo = excluded.usd_diff_ubo,
            usd_diff_pool = excluded.usd_diff_pool,
            amount_total_pool = excluded.amount_total_pool,
            amount_diff_pool = excluded.amount_diff_pool,
            amount_total_ubo = excluded.amount_total_ubo,
            amount_diff_ubo = excluded.amount_diff_ubo
                ",
            )
            .await
            .unwrap();

        // token_address = excluded.token_address,
        // usd_total_pool = excluded.usd_total_pool ,
        // usd_total_ubo = excluded.usd_total_ubo ,
        // usd_diff_ubo = excluded.usd_diff_ubo ,
        // usd_diff_pool = excluded.usd_diff_pool ,
        // token_price_usd = excluded.token_price_usd ,
        // usd_total_pool_18 = excluded.usd_total_pool_18 ,
        // usd_total_ubo_18 = excluded.usd_total_ubo_18 ,
        // usd_diff_ubo_18 = excluded.usd_diff_ubo_18 ,
        // usd_diff_pool_18 = excluded.usd_diff_pool_18

        // println!("input: {:#?}", input);

        // println!(
        //     "testing lipsum {:#?}",
        //     Decimal::from_i128(input.usd_total_pool)
        // );

        // let dolar = input.usd_total_pool >> 96;

        // println!("testing dolar shifted {:#?}", dolar);

        let testing = BigDecimal::from_f64(input.usd_total_pool);

        client
            .query(
                &stmt,
                &[
                    &input.ubo,
                    &input.signer,
                    &input.pool_address,
                    &input.token_address,
                    &input.signature,
                    &input.usd_total_pool.to_string(),
                    &input.usd_total_ubo.to_string(),
                    &input.usd_diff_ubo.to_string(),
                    &input.usd_diff_pool.to_string(),
                    // &input.usd_total_pool.to_string(),
                    // &input.usd_total_ubo.to_string(),
                    // &input.usd_diff_ubo.to_string(),
                    // &input.usd_diff_pool.to_string(),
                    // &Decimal::from_str(&input.usd_total_pool.to_string()).unwrap(),
                    // &Decimal::from_str(&input.usd_total_ubo.to_string()).unwrap(),
                    // &Decimal::from_str(&input.usd_diff_ubo.to_string()).unwrap(),
                    // &Decimal::from_str(&input.usd_diff_pool.to_string()).unwrap(),
                    // &Decimal::from_i128(input.usd_total_ubo).unwrap(),
                    // &Decimal::from_i128(input.usd_diff_ubo).unwrap(),
                    // &Decimal::from_i128(input.usd_diff_pool).unwrap(),
                    &Decimal::from_f64(input.amount_total_pool).unwrap(),
                    &Decimal::from_f64(input.amount_diff_pool).unwrap(),
                    &Decimal::from_f64(input.amount_total_ubo).unwrap(),
                    &Decimal::from_f64(input.amount_diff_ubo).unwrap(),
                    // &Decimal::from_i128(input.usd_total_pool_18).unwrap(),
                    // &Decimal::from_i128(input.usd_total_ubo_18).unwrap(),
                    // &Decimal::from_i128(input.usd_diff_ubo_18).unwrap(),
                    // &Decimal::from_i128(input.usd_diff_pool_18).unwrap(),
                ],
            )
            .await
            .unwrap();

        return Ok(());
    }

    pub async fn save_token_values_inn(&self, input: &PriceItem) -> Result<(), TPError> {
        let price_ref_1 = "USD";

        let value1 = PriceItemDb {
            conversion_ref: price_ref_1.to_string(),
            token_address: input.token_address.to_string(),
            price: input.token_price_usd.to_string(),
            datetime: input.datetime.to_string(),
            transaction_hash: input.signature.to_string(),
            price_formatted: input.token_price_usd_formatted.to_string(),
            oracle_id: "feed80ec-c187-47f5-8684-41931fc780e9".to_string(),
            blocknumber: input.block_number.to_string(),
        };

        let price_ref_2 = "SOL";

        let value2 = PriceItemDb {
            conversion_ref: price_ref_2.to_string(),
            token_address: input.token_address.to_string(),
            price: input.price_token_ref.to_string(),
            datetime: input.datetime.to_string(),
            transaction_hash: input.signature.to_string(),
            price_formatted: input.price_token_ref_formatted.to_string(),
            oracle_id: "feed80ec-c187-47f5-8684-41931fc780e9".to_string(),
            blocknumber: input.block_number.to_string(),
        };

        // price_token_ref

        // let result = self.invoke(self.insert_token_price_inn(value1));

        let _result_1 = self.insert_token_price_inn(value1).await;

        if input.token_address == "So11111111111111111111111111111111111111112" {
            return Ok(());
        }

        let _result_2 = self.insert_token_price_inn(value2).await;

        return Ok(());
        // return result;
    }

    pub async fn insert_token_price_inn(&self, input: PriceItemDb) -> Result<(), TPError> {
        //  conversion_ref, token_address, price_usd, datetime, transaction_hash, price_usd_formatted, oracle_id, blocknumber

        let dolar = self.db_pool.clone();

        let datetime_input: DateTime<Utc> = chrono::DateTime::from_str(&input.datetime).unwrap();
        let datetime_pg: NaiveDateTime = datetime_input.naive_utc();

        let db_connect = match dolar {
            Some(x) => x,
            None => panic!("No db connection"),
        };

        let client = db_connect.get().await.unwrap();

        let stmt = client
            .prepare_cached(
                "INSERT INTO token_prices (
            conversion_ref, 
            token_address, 
            price, 
            datetime, 
            transaction_hash, 
            price_formatted, 
            oracle_id, 
            blocknumber
    ) VALUES ($1::TEXT, 
            $2::TEXT, 
            $3::NUMERIC, 
            $4::TIMESTAMP, 
            $5::TEXT,
            $6::NUMERIC, 
            $7::TEXT,
            $8::NUMERIC
            ) ON CONFLICT ON CONSTRAINT token_prices_v2_pkey DO update set
            conversion_ref=excluded.conversion_ref, 
            token_address=excluded.token_address, 
            price=excluded.price, 
            transaction_hash=excluded.transaction_hash, 
            price_formatted=excluded.price_formatted, 
            oracle_id=excluded.oracle_id, 
            blocknumber=excluded.blocknumber,
            crawled_at = now()::timestamp with time zone
            ;",
            )
            .await
            .unwrap();

        let insert_result = client
            .query(
                &stmt,
                &[
                    &input.conversion_ref,
                    &input.token_address,
                    &Decimal::from_str(&input.price).unwrap(),
                    &datetime_pg,
                    &input.transaction_hash,
                    &Decimal::from_str(&input.price_formatted).unwrap(),
                    &input.oracle_id.to_string(),
                    &Decimal::from_str(&input.blocknumber).unwrap(),
                ],
            )
            .await;

        if (insert_result.is_err()) {
            println!("error inserting: {:?}", insert_result);
        }

        return Ok(());
    }

    pub async fn insert_token_price_inn_ref(&self, input: PriceItem) -> Result<(), TPError> {
        let rolar: DateTime<Utc> = chrono::DateTime::from_str(&input.datetime).unwrap();

        let dolar = self.db_pool.clone();
        let datetime_param: NaiveDateTime = rolar.naive_utc();

        let db_connect = match dolar {
            Some(x) => x,
            None => panic!("No db connection"),
        };

        let client = db_connect.get().await.unwrap();

        let stmt = client
            .prepare_cached(
                "INSERT INTO token_prices (
            signature, 
            token_a_address,
            token_b_address,
            token_a_price_usd,
            token_b_price_usd,
            token_a_price_usd_formatted,
            token_b_price_usd_formatted,
            datetime,
            oracle_id, 
            signer, 
            ubo, 
            pool_address,
            usd_total_pool
    ) VALUES ($1::TEXT, 
            $2::TEXT, 
            $3::TEXT, 
            $4::NUMERIC, 
            $5::NUMERIC,
            $6::NUMERIC, 
            $7::NUMERIC,
            $8::TIMESTAMP,
            $9::TEXT, 
            $10::TEXT, 
            $11::TEXT, 
            $12::TEXT, 
            $13::NUMERIC
            ) ON CONFLICT ON CONSTRAINT token_prices_pkey DO update set
            signature = excluded.signature,
            token_a_price_usd = excluded.token_a_price_usd,
            token_b_price_usd = excluded.token_b_price_usd,
            token_a_price_usd_formatted = excluded.token_a_price_usd_formatted,
            token_b_price_usd_formatted = excluded.token_b_price_usd_formatted, 
            signer = excluded.signer,
            ubo = excluded.ubo,
            pool_address = excluded.pool_address,
            usd_total_pool = excluded.usd_total_pool,
            crawled = now()::timestamp with time zone
            ;",
            )
            .await
            .unwrap();

        let tesitng = client
            .query(
                &stmt,
                &[
                    &input.signature,
                    &input.token_ref_address,
                    &input.token_address,
                    &Decimal::from_str(&input.token_price_usd).unwrap(),
                    &Decimal::from_str(&input.token_ref_price_usd).unwrap(),
                    &Decimal::from_str(&input.token_price_usd_formatted).unwrap(),
                    &Decimal::from_str(&input.token_ref_price_usd_formatted).unwrap(),
                    &datetime_param,
                    &"feed80ec-c187-47f5-8684-41931fc780e9".to_string(),
                    &input.signer,
                    &input.ubo,
                    &input.pool_address,
                    &Decimal::from_str(&input.usd_total_pool).unwrap(),
                    // &Json::<TokenAmountsPriced>(input.token_a_usd),
                    // &Json::<TokenAmountsPriced>(input.token_b_usd),
                    // &Json::<TokenAmounts>(input.token_amounts_a),
                    // &Json::<TokenAmounts>(input.token_amounts_b),
                ],
            )
            .await;
        // .unwrap();

        return Ok(());
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

    pub async fn get_usd_price_sol_inn(&self, transaction_datetime: String) -> DbResult<Decimal> {
        let statement =
            "SELECT * FROM token_prices_temp WHERE token_address = $1 AND conversion_ref = 'USD'
                    order by abs(extract(epoch from (timestamp - $2))) limit 1";

        let rolar: DateTime<Utc> = chrono::DateTime::from_str(&transaction_datetime).unwrap();

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

        return Ok(dolar2);
    }

    // pub async fn test_sender_inn(&self, testing: bool) -> DbResult<bool> {
    //     let result = self.sender.send(testing).await;

    //     return Ok(result.unwrap());
    // }

    // fn invoke<T, F: std::future::Future<Output = DbResult<T>>>(&self, f: F) -> DbResult<T> {
    //     // `block_on()` panics if called within an asynchronous execution context. Whereas
    //     // `block_in_place()` only panics if called from a current_thread runtime, which is the
    //     // lesser evil.
    //     tokio::task::block_in_place(move || self.runtime.as_ref().expect("runtime").block_on(f))
    // }
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
