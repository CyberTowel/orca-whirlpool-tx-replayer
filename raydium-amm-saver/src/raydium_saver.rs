pub mod raydium {

    use arl::RateLimiter;
    use async_trait::async_trait;
    use chrono::prelude::*;
    use deadpool::managed::{self, Metrics};

    use num_bigfloat::{BigFloat, RoundingMode};

    // use deadpool::managed::{Manager, Object, Pool};
    use deadpool::managed::RecycleResult;
    use rust_decimal::prelude::*;
    use solana_client::{
        rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
        rpc_response::RpcConfirmedTransactionStatusWithSignature,
    };
    use solana_sdk::signature::Signature;
    use solana_transaction_status::{UiTransactionEncoding, UiTransactionTokenBalance};

    use crate::raydium_saver::pg_saving::PriceDbItem;
    use crate::token_db::{DbClientPoolManager, TokenDbClient};

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
                parse_signature(&signature.signature, &connection, &db_client);
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

        let transaction_block_timestamp = transaction.as_ref().unwrap().block_time.unwrap();

        println!(
            "======================================= signature: {}, time: {} ========================================",
            signature,
            DateTime::from_timestamp(transaction_block_timestamp, 0).unwrap()
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
            })
            .unwrap()
            .ui_token_amount
            .amount;

        let transaction_datetime = DateTime::from_timestamp(transaction_block_timestamp, 0)
            .unwrap()
            .to_rfc3339();

        let usd_price_nearest = db_client.get_usd_price_sol(transaction_datetime).unwrap();

        let price = get_price(
            amount_token_a_test,
            amount_token_b_test,
            &usd_price_nearest.to_string(),
        );

        let (price_usd, price_token_ref) = price.unwrap();

        let item: PriceDbItem = PriceDbItem {
            price_token_ref: price_token_ref.to_string(),
            price_usd: price_usd.to_string(),
            datetime: DateTime::from_timestamp(transaction_block_timestamp, 0)
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
    pub enum Error {}

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

    pub async fn _save_price_to_db(price_item: PriceDbItem, pool: &Pool<Manager>) {
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

        if testing.is_err() {
            println!("error saving item, {:?}", testing.unwrap());
        }
    }
}
