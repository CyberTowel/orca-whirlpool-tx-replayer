mod replay_saver;
use clap::Parser;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use replay_saver::instruction_callback::parse_instruction;
use solana_client::rpc_client::RpcClient;
use std::collections::HashMap;
use tokio::task::JoinSet;
use tokio_postgres::{Error, NoTls};
use whirlpool_replayer::{ReplayUntil, SlotCallback, WhirlpoolReplayer};

use chrono::prelude::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, id = "directory")]
    cache_dir: Option<String>,

    #[clap(short, long, id = "filename")]
    save_as: Option<String>,

    #[clap(long, id = "slot")]
    stop_slot: Option<u64>,

    #[clap(long, id = "lieke")]
    lieke: Option<String>,

    #[clap(long, id = "blockHeight")]
    stop_block_height: Option<u64>,

    #[clap(long, id = "blockTime")]
    stop_block_time: Option<i64>,

    #[clap(id = "path|url")]
    storage: String,

    #[clap(id = "yyyymmdd")]
    yyyymmdd: String,
}

fn create_pool() -> Pool {
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    // let token_address = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

    let rpc_connection = RpcClient::new_with_commitment(
        // cluster,
        "https://solana-mainnet.api.syndica.io/api-token/31LXqG31wuwf82G821o7odUPqZnuxHjkaeCtsbDmVFyorPVtZgcTt3fd9to6CNEaMMRHMwJHASa4WQsttc15zhLwnLbZ8qNTQxekxymxfhSFzda3mhpp4F95xLmZKqjPueVMBWCdYUA32dPCjm8w9SzSebRWtmocZVs1m9KsbFq4MGvgsKtxYJvc86QEqJtdzcn82BVcpsXV7Cmbr4oL3j37yyi8RfLGCDdoQo2mUKC8xDPocCB4rMsb8PM7JB8kLsPWEdCeGsfwb66wBMVGyT8zr9fZsB6fxJvMjgP5W1xyL2BnCVRZ1dotGawiwung88pxuy84o1tpTpmJWHqwFdxHKCWQwxXeJysZ81DzCY3X9nVdxbMpUnz9tJVzFMSwxNomKFT925ogVNgYHYzV2TCBYSKyj53s8xiKZU6X4nAGXFkpTRXGHbnAvi8cRB9cPXaQyc2Yad6GxUeCTyPQqPJ8fZ8gHZmPCF9UKv836Ao93AawumPL1e4RdLScW".to_string(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );

    // let token_decimals =
    //     replay_saver::instruction_callback::get_decimals_from_token(&rpc_connection, token_address);

    // let token_info =
    //     replay_saver::instruction_callback::get_token_metadata(&rpc_connection, token_address);

    // println!("balance accounts: {:?}", token_decimals);
    // println!("token_info, {:?}", token_info);

    // println!("args: {:?}", args.lieke);

    let base_path_or_url: String = args.storage;
    let yyyymmdd: String = args.yyyymmdd;

    let until_condition = if args.stop_slot.is_some() {
        ReplayUntil::Slot(args.stop_slot.unwrap())
    } else if args.stop_block_height.is_some() {
        ReplayUntil::BlockHeight(args.stop_block_height.unwrap())
    } else if args.stop_block_time.is_some() {
        ReplayUntil::BlockTime(args.stop_block_time.unwrap())
    } else {
        ReplayUntil::End
    };

    let slot_callback: Option<SlotCallback> = Some(|_slot| {
        // let timestamp = "1524820690".parse::<i64>().unwrap();

        let naive = DateTime::from_timestamp(_slot.block_time, 0);

        let testing = naive.unwrap().format("%Y-%m-%d %H:%M:%S");
        // let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc)

        // let time_formatted = naive.format("%Y-%m-%d %H:%M:%S");

        println!(
            "processing slot: {} (block_height={} block_time={}, time={}) ...",
            _slot.slot, _slot.block_height, _slot.block_time, testing
        );
    });

    let mut replayer = if base_path_or_url.starts_with("https://") {
        if args.cache_dir.is_some() {
            let cache_dir = args.cache_dir.unwrap();
            WhirlpoolReplayer::build_with_remote_file_storage_with_local_cache(
                &base_path_or_url,
                &yyyymmdd,
                &cache_dir,
                false,
            )
        } else {
            WhirlpoolReplayer::build_with_remote_file_storage(&base_path_or_url, &yyyymmdd)
        }
    } else {
        WhirlpoolReplayer::build_with_local_file_storage(&base_path_or_url, &yyyymmdd)
    };

    println!("start processing instructions");

    replayer.replay(until_condition, slot_callback, parse_instruction);

    let mut token_decimals: HashMap<String, u8> = HashMap::new();

    for item in replayer.parsed_instructions.iter() {
        if !token_decimals.contains_key(&item.token_a) {
            let decimals = replay_saver::instruction_callback::get_decimals_from_token(
                &rpc_connection,
                &item.token_a,
            );
            token_decimals.insert(item.token_a.clone(), decimals);
        }

        if !token_decimals.contains_key(&item.token_b) {
            let decimals = replay_saver::instruction_callback::get_decimals_from_token(
                &rpc_connection,
                &item.token_b,
            );
            token_decimals.insert(item.token_b.clone(), decimals);
        }
    }

    println!("token_decimals, {:#?}", token_decimals);

    // println!("replayer results , {:#?}", replayer.parsed_instructions);

    let pool = create_pool();

    let mut instructions_to_save = JoinSet::new();

    for testing in replayer.parsed_instructions.iter_mut() {
        let decimals_a = token_decimals.get(&testing.token_a).unwrap();
        let decimals_b = token_decimals.get(&testing.token_b).unwrap();

        let price = replay_saver::instruction_callback::get_price_from_tick_price(
            testing.sqrt_price_pre,
            *decimals_a,
            *decimals_b,
        );

        testing.price = price.to_string();
        // println!("testing, {:#?}", price);
    }

    if args.lieke == Some("test".to_string()) {
        println!("Dont in database");
        return Ok(());
    }

    println!("save in database");

    for instruction in replayer.parsed_instructions {
        let pool_inner = pool.clone();
        instructions_to_save.spawn(async move {
            let client = pool_inner.get().await.unwrap();

            // tick_spacing_pre,
            // tick_spacing_post
            // $4::SMALLINT,
            // $4::SMALLINT
            let stmt = client
                .prepare_cached(
                    "INSERT INTO sol_whirlpool_events (
                    signature, 
                    tick_spacing, 
                    instruction_type, 
                    fee_rate, 
                    pool_address, 
                    amount_in, 
                    amount_out,
                    sqrt_price_pre, 
                    sqrt_price_post, 
                    token_a, 
                    token_b,
                    price_formatted, 
                    date_time
            ) VALUES ($1::TEXT, 
                    $2::SMALLINT, 
                    $3::TEXT, 
                    $4::SMALLINT, 
                    $5::TEXT,
                    $6::BIGINT, 
                    $7::BIGINT,
                    $8::BIGINT,
                    $9::BIGINT, 
                    $10::TEXT,
                    $11::TEXT,
                    $12::TEXT, 
                    $13::TEXT
                    )",
                )
                .await
                .unwrap();

            client
                .query(
                    &stmt,
                    &[
                        &instruction.signature,
                        &instruction.tick_spacing,
                        &instruction.instruction_type,
                        &instruction.fee_rate,
                        &instruction.pool_address,
                        &instruction.amount_in,
                        &instruction.amount_out,
                        &instruction.sqrt_price_pre,
                        &instruction.sqrt_price_post,
                        &instruction.token_a,
                        &instruction.token_b,
                        &instruction.price,
                        &instruction.date_time,
                    ],
                )
                .await
                .unwrap();

            instruction.signature
        });
    }

    let mut crawled_signatures: Vec<String> = Vec::new();

    while let Some(res) = instructions_to_save.join_next().await {
        let idx = res.unwrap();
        crawled_signatures.push(idx);
    }

    println!("saved, {} items", crawled_signatures.len());

    Ok(())
}
