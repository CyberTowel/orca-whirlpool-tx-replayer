mod raydium_saver;
use crate::pool_state::get_pool_meta;
use crate::token_db::{DbClientPoolManager, DbPool};
use crate::token_parser::PoolVars;
use clap::Parser;
use raydium_saver::raydium::{
    batch_process_signatures, get_paginated_singatures, RpcPool, RpcPoolManager,
};

pub mod pool_state;
pub mod token_db;
pub mod token_parser;
pub mod transaction;
pub mod transaction_parser;

#[derive(Debug)]
pub struct TestTransaction {
    siganture: String,
    pool_id: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    /// maximum depth to which sub-directories should be explored
    sample_rate: Option<usize>,

    #[clap(long)]
    start_at: Option<String>,

    #[clap(long)]
    pool_id: Option<String>,

    #[clap(long)]
    rpc_type: Option<String>,

    #[clap(long)]
    rate_limit: Option<usize>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // let limit_to_set = args.rate_limit.unwrap_or(1000);

    println!("Args set: {:?}", args);

    let mgr = RpcPoolManager {
        rpc_type: args.rpc_type,
    };

    let db_mgr: DbClientPoolManager = DbClientPoolManager {};

    let pool: deadpool::managed::Pool<RpcPoolManager> =
        RpcPool::builder(mgr).max_size(1000).build().unwrap();

    let db_pool = DbPool::builder(db_mgr).max_size(1000).build().unwrap();

    // let limiter = RateLimiter::new(limit_to_set, Duration::from_secs(1));

    let testing_mode = false;

    let testing_singatures: Vec<TestTransaction> = vec![
        TestTransaction {
            siganture: "KAmSgZzgX6ZehgKk8MGjsbE4TLKFeN4Z77B5bqeGrWz2DxUYHrrpTTUjGPdzcmhMAQ1EH9qL9ShFzdh4fkt8dN9".to_string(),
            pool_id: "DpVCqPBjehK8cJjRnRg2kmyhZpdhUh7EoEzvKtxTEi3L".to_string(),
        },
        TestTransaction {
            siganture:"rprLhWtWRWvRnmzPPYFVH5PL6aE2iST9sphXoDnze1CDz14GvpYPZFSNKBeieQ4U8PKNfU173pUL8ErJgwjp4ow".to_string(),
            pool_id:"58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string()
        },
        TestTransaction {
            siganture: "55NZBEgP5PWfGxB1rqLBeUKFZsTwjnkiEVQfAxdpAFRj6pCLMzWNXTdX7J96A6hvHasuyVPQT3RERgFaYCnqLi9U".to_string(),
            pool_id: "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string(),
        },
        // TestTransaction {
        //     siganture: "rprLhWtWRWvRnmzPPYFVH5PL6aE2iST9sphXoDnze1CDz14GvpYPZFSNKBeieQ4U8PKNfU173pUL8ErJgwjp4ow".to_string(),
        //     pool_id: "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(),
        // },
        //  TestTransaction {
        //     siganture: "55NZBEgP5PWfGxB1rqLBeUKFZsTwjnkiEVQfAxdpAFRj6pCLMzWNXTdX7J96A6hvHasuyVPQT3RERgFaYCnqLi9U".to_string(),
        //     pool_id:"8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR".to_string()
        // }
    ];

    if testing_mode {
        println!("Testing mode enabled");
        for item in testing_singatures {
            let poolvars = PoolVars {
                pool_id: item.pool_id.to_string(),
                // pool_coin_token_account: pool_coin_token_account.to_string(),
                // amm_target_orders: amm_target_orders.to_string(),
                // token_a_address: token_a_address.to_string(),
                // token_b_address: token_b_address.to_string(),
            };

            let signatures_to_use = vec![item.siganture];
            // let poolvars

            // let pool_state = get_pool_state(poolvars.pool_id.clone());

            let pool_meta = get_pool_meta(&poolvars.pool_id);

            batch_process_signatures(
                signatures_to_use,
                &pool,
                // &limiter,
                &db_pool,
                &pool_meta,
                &poolvars,
            )
            .await;
        }

        return;
    }

    let pool_id = args
        .pool_id
        .unwrap_or("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string()); //"";

    // values IDK
    // let pool_id = "8gptfZ8bkT2Z1gMv38VpxarFfCXZPCykFKjGUkYJnfCR";
    // let signature =
    //     "55NZBEgP5PWfGxB1rqLBeUKFZsTwjnkiEVQfAxdpAFRj6pCLMzWNXTdX7J96A6hvHasuyVPQT3RERgFaYCnqLi9U";

    // let pool_coin_token_account = "Ffo9MEhfH5tBBkZMi1vWVpZLqmbDKvEWJhW3XyMQz4QY";
    // let amm_target_orders = "EM9ebwJyrenPmgXQyn9aR5X2tiJssrVPwLSZXxmg2dLy";
    // let token_a_address: &str = "So11111111111111111111111111111111111111112";
    // let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

    // values SOL USDC
    // let pool_id = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    // let signature =
    //     "rprLhWtWRWvRnmzPPYFVH5PL6aE2iST9sphXoDnze1CDz14GvpYPZFSNKBeieQ4U8PKNfU173pUL8ErJgwjp4ow";

    // let pool_coin_token_account = "DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz";
    // let amm_target_orders = "HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz"; // for this one it's - Pool Pc Token Account
    // let token_a_address: &str = "So11111111111111111111111111111111111111112";
    // let token_b_address: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    let poolvars = PoolVars {
        pool_id: pool_id.to_string(),
    };

    let pool_meta = get_pool_meta(&poolvars.pool_id);

    let mut _has_more = true;
    let mut _batch = 0;

    let mut before_signature: Option<String> = args.start_at.clone();

    while _has_more == true {
        _batch += 1;

        _has_more = false;

        // testing_singatures.push(signature.to_string());

        let pool_c = pool.clone();

        let start = std::time::Instant::now();

        let signatures_to_use = {
            let (items_to_process, next_item) =
                get_paginated_singatures(&pool_id, pool_c, before_signature, args.sample_rate)
                    .await;

            before_signature = next_item.clone();
            _has_more = next_item.is_some();
            items_to_process
        };

        if (signatures_to_use.len() == 0) {
            println!("No more signatures to process");
            break;
        }

        batch_process_signatures(
            signatures_to_use,
            &pool,
            // &limiter,
            &db_pool,
            &pool_meta,
            &poolvars,
        )
        .await;

        let elapsed = start.elapsed();
        println!("Batch {} took {:?}", _batch, elapsed);
    }

    println!("No more transactions to process");
}
