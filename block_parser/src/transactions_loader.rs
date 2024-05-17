use crate::interfaces::Transaction;
use chrono::prelude::*;
use moka::future::Cache;
use num_bigfloat::E;
use rust_decimal::Decimal;
use serde_json::json;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedTransactionWithStatusMeta, UiInnerInstructions,
    UiInstruction, UiParsedInstruction, UiTransactionEncoding,
};
use std::str::FromStr;

use crate::{
    pool_state::get_pool_meta,
    token_db::TokenDbClient,
    token_parser::{
        get_price, get_token_amounts, parse_token_amounts_new, parse_token_price_oracle_values,
        PoolMeta, PriceItem,
    },
    transaction,
};

pub fn testing_nested() {
    println!("Testing nested");
}
#[derive(Debug)]
pub struct TransactionError {
    pub message: String,
}

pub async fn get_transction(
    signature: String,
    pool_id: Option<String>,
    rpc_connection: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: Cache<String, Option<PoolMeta>>,
) -> Result<Transaction, TransactionError> {
    let rpc_config: RpcTransactionConfig = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: Some(CommitmentConfig::finalized()),
        max_supported_transaction_version: Some(1),
    };

    let transaction_req = rpc_connection
        .get_transaction_with_config(&Signature::from_str(&signature).unwrap(), rpc_config)
        .await;

    if transaction_req.is_err() {
        println!(
            "Error in transaction: {:#?} {:#?}",
            signature, transaction_req
        );
        return Err(TransactionError {
            message: "Error in transaction".to_string(),
        });
    }

    let confirmed_tx = transaction_req.unwrap();

    let transaction = EncodedTransactionWithStatusMeta {
        transaction: confirmed_tx.transaction.transaction,
        meta: confirmed_tx.transaction.meta,
        version: confirmed_tx.transaction.version,
    };

    let block_time = confirmed_tx.block_time.unwrap();
    let block_number = confirmed_tx.slot;

    init(
        signature,
        pool_id,
        rpc_connection,
        rpc_connection,
        db_client,
        &my_cache,
        &transaction,
        block_time,
        block_number,
        None,
    )
    .await
}

pub enum Error {
    Msg(String),
}

pub async fn init(
    signature: String,
    pool_id: Option<String>,
    _rpc_connection: &RpcClient,
    rpc_connection_build: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: &Cache<String, Option<PoolMeta>>,
    transaction: &EncodedTransactionWithStatusMeta,
    block_time: i64,
    block_number: u64,
    sol_price_18: Option<Decimal>,
) -> Result<Transaction, TransactionError> {
    // let sol_price_db =

    // std::thread::sleep(std::time::Duration::from_secs(10));

    // println!("done sleeping start process, {}", signature);
    // return;
    // tokio::time::sleep(Duration::from_secs(sleep_duraction as u64)).await;

    // let sol_price_db = "1400000000000000000000".to_string();

    let _transaction_datetime = DateTime::from_timestamp(block_time, 0)
        .unwrap()
        .to_rfc3339();

    // println!(
    //     "Init transaction: {:#?}, timestamp: {}",
    //     signature, transaction_datetime
    // );

    // println!("Transaction: {:#?}", transaction);

    let v = json!(transaction.transaction);
    // let account_keys = v["message"]["accountKeys"].as_array().unwrap();

    // let signer = find_signer(account_keys);
    // let testing_pool_id = parse_pool_create_instruction(transaction.transaction);

    let instructions = v["message"]["instructions"].as_array().unwrap();

    let transactions_meta = transaction.clone().meta.unwrap(); // v["message"].as_array().unwrap();

    let pool_id_to_get_opt: Option<String> = if pool_id.is_some() {
        pool_id.clone() //.unwrap()
    } else {
        let init_instruction = instructions.iter().find(|&x| {
            let program_id = x["programId"].as_str().unwrap();

            program_id == "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        });

        if init_instruction.is_some() {
            // println!(
            //     "{:#?} {:#?} {:#?}",
            //     init_instruction.unwrap().as_object().unwrap()["data"],
            //     init_instruction.unwrap().as_object().unwrap()["data"]
            //         .as_str()
            //         .unwrap(),
            //     init_instruction
            // );

            let pool_id_instruction = init_instruction.unwrap().as_object().unwrap()["accounts"][1]
                .as_str()
                .unwrap();
            Some(pool_id_instruction.to_string())
            // pool_id_instruction.as_str().unwrap().to_string()
        } else {
            let dolar_selit = find_raydium_inner_instruction(&transactions_meta.inner_instructions);

            // println!("Inner instruction accounts: {:#?}", dolar_selit);

            if dolar_selit.len() > 0 {
                // return None;
                Some(dolar_selit[1].to_string())
            } else {
                None
            }
        }
    };

    let transaction_parsed = Transaction::new(transaction, block_time, block_number);

    if pool_id_to_get_opt.is_none() {
        // println!("Pool id to get is none for signature: {}", signature);
        return Ok(transaction_parsed);
        // return Err(Error::Msg("Error in transaction".to_string()));
    }

    let pool_id_to_get = pool_id_to_get_opt.unwrap();

    let pool_id_clone = pool_id_to_get.clone();

    let pool_info_cache = my_cache.get(&pool_id_to_get).await;

    let pool_meta_req = if pool_info_cache.is_some() {
        let info = pool_info_cache.unwrap();
        // println!(
        //     "=========== Pool info from cache loaded for pool {}",
        //     pool_id_to_get
        // );
        Some(info)
    } else {
        // my_cache.insert(pool_id_to_get.to_string(), testing).await;

        // let meta_test = PoolMeta {
        //     base_decimal: 6,
        //     base_lot_size: 10000000,
        //     base_need_take_pnl: 0,
        //     base_total_pnl: 0,
        //     base_total_deposited: 0,
        //     base_vault: Pubkey::from_str("HUM2zGxzxUZ5hgc3AWdXBykK35NRSUgCYhAaJcVhrp8n").unwrap(),
        //     base_mint: Pubkey::from_str("HfHU9YS9hH5buRKDfjEyMopfP9QtuDznR1wqUpzJtWHT").unwrap(),
        //     quote_decimal: 9,
        //     quote_lot_size: 100,
        //     quote_need_take_pnl: 0,
        //     quote_total_pnl: 0,
        //     quote_total_deposited: 1715371200,
        //     quote_vault: Pubkey::from_str("9QVgsXhHMW6Ksky2jFHoJwJR4WRVFca8ZjuEE5cwtNi").unwrap(),
        //     quote_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
        // };

        // my_cache
        //     .insert(pool_id_to_get.to_string(), meta_test.clone())
        //     .await;

        let info = my_cache
            .get_with(pool_id_to_get.clone(), async move {
                // println!("Task {pool_id_to_get} inserting a value.");
                let inf_req = get_pool_meta(&pool_id_to_get, rpc_connection_build).await;
                // Arc::new(vec![0u8; TEN_MIB])

                if inf_req.is_none() {
                    // println!(
                    //     "Error getting pool info for pool {}",
                    //     pool_id_to_get.to_string()
                    // );
                    return None;
                }

                Some(inf_req.unwrap())
            })
            .await;

        // let info_req = get_pool_meta(&pool_id_to_get, rpc_connection_build).await;

        // if info_req.is_none() {
        //     // println!(
        //     //     "Error getting pool info for pool {}",
        //     //     pool_id_to_get.to_string()
        //     // );
        //     return;
        // }

        // let info = info_req.unwrap();

        // println!(
        //     "=========== Pool info loaded for pool without cache {}",
        //     pool_id_clone
        // );

        // my_cache
        //     .insert(pool_id_to_get.to_string(), info.clone())
        //     .await;

        // println!(
        //     "=========== Pool info inserted to cache for pool {:#?}",
        //     info
        // );

        Some(info)
    };

    if !pool_meta_req.is_some() {
        // println!("Error getting pool meta for pool {}", pool_id_clone);
        return Ok(transaction_parsed);
    }

    let pool_meta_opt = pool_meta_req.unwrap();

    if !pool_meta_opt.is_some() {
        // println!("Error getting pool meta for pool opt {}", pool_id_clone);
        return Ok(transaction_parsed);
    }

    let pool_meta = pool_meta_opt.unwrap();

    let transaction_parsed_meta = transaction_parsed.clone();

    let sol_price_db = if sol_price_18.is_some() {
        sol_price_18.unwrap()
        // println!("Sol price 18: {:#?}", sol_price_18.unwrap().to_string());
    } else {
        println!("get sol price per transaction");

        db_client
            .get_usd_price_sol(transaction_parsed_meta.block_datetime)
            .unwrap()
    };

    let token_amounts_req = get_token_amounts(
        &transaction,
        &transaction_parsed_meta.addresses,
        &transaction_parsed_meta.ubo,
        &pool_meta.quote_mint.to_string(),
        &pool_meta.base_mint.to_string(),
        &pool_meta.quote_vault.to_string(),
        &pool_meta.base_vault.to_string(),
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
        // decimals_correct, // pool_state,
    );

    if token_amounts_req.is_none() {
        // println!("Error getting token amounts for pool {}", pool_id_to_get);
        return Ok(transaction_parsed);
    }

    let token_amounts = token_amounts_req.unwrap();

    // println!("Sol price: {:#?}", sol_price_db.to_string());

    let token_prices = get_price(
        token_amounts.token_new_price_in_token_quote_18,
        token_amounts.token_trade_price_in_token_quote_18,
        &pool_meta.quote_mint.to_string(),
        &sol_price_db.to_string(),
        // decimals_correct,
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
    )
    .unwrap();

    let swap_token_amounts_priced = parse_token_amounts_new(
        &token_amounts,
        &token_prices,
        // price.token_price_usd_18,
        // price.token_ref_price_usd_18,
        // pool_state.quote_decimal,
        // pool_state.base_decimal,
    );

    let datetime = DateTime::from_timestamp(transaction_parsed.block_timestamp, 0)
        .unwrap()
        .to_rfc3339();

    if transactions_meta.err.is_some() {
        return Ok(transaction_parsed);
    }

    let price_item_to_save = PriceItem {
        signature: signature.to_string(),
        token_quote_address: pool_meta.quote_mint.to_string(),
        token_base_address: pool_meta.base_mint.to_string(),

        token_new_price_18: token_prices.token_new_price_18,
        token_new_price_fixed: token_prices.token_new_price_fixed,
        token_new_price_in_token_quote_18: token_prices.token_new_price_in_token_quote_18,
        token_new_price_in_token_quote_fixed: token_prices.token_new_price_in_token_quote_fixed,

        token_trade_price_18: token_prices.token_trade_price_18,
        token_trade_price_fixed: token_prices.token_trade_price_fixed,
        token_trade_price_in_token_quote_18: token_prices.token_trade_price_in_token_quote_18,
        token_trade_price_in_token_quote_fixed: token_prices.token_trade_price_in_token_quote_fixed,

        datetime: datetime,
        signer: transaction_parsed.signer.to_string(),
        ubo: transaction_parsed.ubo.to_string(),
        pool_address: pool_id_clone.clone(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18,
        block_number: transaction_parsed.block_number.to_string(),
    };

    // println!("item_to_save: {:#?}", item_to_save);
    //     println!(
    //         "price update for token {:#?} ->  {:#?}
    // signature: {}
    // ====================", // token_new_price_18: {:#?}
    //         // token_new_price_fixed: {:#?}
    //         // token_new_price_in_token_quote_18: {:#?}
    //         // token_new_price_in_token_quote_fixed: {:#?}
    //         item_to_save.token_base_address.to_string(),
    //         // item_to_save.token_new_price_18.to_f64().to_string(),
    //         item_to_save.token_new_price_fixed.to_f64().to_string(),
    //         item_to_save.signature,
    //         // item_to_save
    //         //     .token_new_price_in_token_quote_18
    //         //     .to_f64()
    //         //     .to_string(),
    //         // item_to_save
    //         //     .token_new_price_in_token_quote_fixed
    //         //     .to_f64()
    //         //     .to_string(),
    //     );

    let _price_item_c = price_item_to_save.clone();

    let reponse = db_client.save_token_values(price_item_to_save);

    let tpo_values_a = parse_token_price_oracle_values(
        transaction_parsed_meta.ubo.to_string(),
        transaction_parsed_meta.signer.to_string(),
        pool_id_clone.to_string(),
        pool_meta.base_mint.to_string(),
        &token_amounts.token_amounts_quote,
        &swap_token_amounts_priced.token_amounts_priced_a,
        &signature,
    );

    let tpo_values_b = parse_token_price_oracle_values(
        transaction_parsed_meta.ubo.to_string(),
        transaction_parsed_meta.signer.to_string(),
        pool_id_clone.to_string(),
        pool_meta.quote_mint.to_string(),
        &token_amounts.token_amounts_base,
        &swap_token_amounts_priced.token_amounts_priced_b,
        &signature,
    );

    let response_token_usd_a = db_client.insert_token_usd_values(&signature, &tpo_values_a);

    if response_token_usd_a.is_err() {
        println!(
            "Error saving token usd values to db: {:#?}",
            response_token_usd_a
        );
    }

    let response_token_usd_b = db_client.insert_token_usd_values(&signature, &tpo_values_b);

    if response_token_usd_b.is_err() {
        println!(
            "Error saving token usd values to db: {:#?}",
            response_token_usd_b
        );
    }

    if reponse.is_err() {
        println!("Error saving to db: {:#?}", reponse);
    }

    return Ok(transaction_parsed);

    // println!("done")

    // return (
    //     signature.to_string(),
    //     price_item_c.datetime,
    //     "success".to_string(),
    // );

    // println!("Token amounts: {:#?}", swap_token_amounts_priced);
}

fn find_raydium_inner_instruction(
    inner_instructions: &OptionSerializer<Vec<UiInnerInstructions>>,
) -> Vec<std::string::String> {
    let mut inner_instruction_accounts: Vec<String> = Vec::new();

    match &inner_instructions {
        OptionSerializer::Some(ixs) => {
            ixs.iter().for_each(|x| {
                x.instructions.iter().for_each(|i| match i {
                    UiInstruction::Compiled(_ix) => {
                        println!("testing 5");
                        panic!("inplement this UiParsedInstruction Compiled")
                        // println!("Data test: {:#?}", ix);
                        // let maybe_market = parse_market_from_data(ix.data.clone(), block_time);
                        // match maybe_market {
                        //     Some(market) => {}
                        //     None => {}
                        // }
                    }
                    UiInstruction::Parsed(ix) => match ix {
                        UiParsedInstruction::Parsed(x) => match x.parsed.get("data") {
                            Some(d) => match d {
                                serde_json::Value::String(_data) => {
                                    // println!("Data test: {:#?}", d);

                                    panic!("inplement this UiParsedInstruction")
                                    // let maybe_market =
                                    //     parse_market_from_data(data.to_string(), block_time);
                                    // match maybe_market {
                                    //     Some(market) => markets_vector.push(market),
                                    //     None => {}
                                    // }
                                }
                                _ => {}
                            },
                            None => {}
                        },
                        UiParsedInstruction::PartiallyDecoded(d) => {
                            if d.program_id == "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" {
                                inner_instruction_accounts.extend(d.to_owned().accounts);
                            }

                            // let maybe_market = parse_market_from_data(d.data.clone(), block_time);

                            // match maybe_market {
                            //     Some(market) => markets_vector.push(market),
                            //     None => {}
                            // }
                        }
                    },
                })
            });
        }
        OptionSerializer::None => {}
        OptionSerializer::Skip => {}
    };

    inner_instruction_accounts
}
