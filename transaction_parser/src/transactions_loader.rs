use chrono::prelude::*;
use moka::future::Cache;
use serde_json::json;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig, config::program, program_error, signature::Signature,
};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiParsedInstruction,
    UiTransactionEncoding,
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

pub async fn get_transction(
    signature: String,
    pool_id: Option<String>,
    rpc_connection: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: Cache<String, PoolMeta>,
) {
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
        return;
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
    )
    .await;
}

pub async fn init(
    signature: String,
    pool_id: Option<String>,
    rpc_connection: &RpcClient,
    rpc_connection_build: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: &Cache<String, PoolMeta>,
    transaction: &EncodedTransactionWithStatusMeta,
    block_time: i64,
    block_number: u64,
) {
    // std::thread::sleep(std::time::Duration::from_secs(10));

    // println!("done sleeping start process, {}", signature);
    // return;
    // tokio::time::sleep(Duration::from_secs(sleep_duraction as u64)).await;

    // let sol_price_db = "1400000000000000000000".to_string();

    let transaction_datetime = DateTime::from_timestamp(block_time, 0)
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

            if (dolar_selit.len() > 0) {
                // return None;
                Some(dolar_selit[1].to_string())
            } else {
                None
            }
        }
    };

    if (pool_id_to_get_opt.is_none()) {
        // println!("Pool id to get is none for signature: {}", signature);
        return;
    }

    let pool_id_to_get = pool_id_to_get_opt.unwrap();

    // let pool_meta = get_pool_meta(&pool_id_to_get, rpc_connection);

    let pool_info_cache = my_cache.get(&pool_id_to_get).await;

    let pool_meta_req = if pool_info_cache.is_some() {
        let info = pool_info_cache.unwrap();
        // println!("=========== Pool info from cache loaded for pool",);
        Some(info)
    } else {
        let info_req = get_pool_meta(&pool_id_to_get, rpc_connection_build).await;

        if info_req.is_none() {
            // println!(
            //     "Error getting pool info for pool {}",
            //     pool_id_to_get.to_string()
            // );
            return;
        }

        let info = info_req.unwrap();

        my_cache
            .insert(pool_id_to_get.to_string(), info.clone())
            .await;

        Some(info)
    };

    if (!pool_meta_req.is_some()) {
        println!("Error getting pool meta for pool {}", pool_id_to_get);
        return;
    }

    let pool_meta: PoolMeta = pool_meta_req.unwrap();

    let transaction_parsed = transaction::Transaction::new(transaction, block_time, block_number);

    let token_amounts_req = get_token_amounts(
        &transaction,
        &transaction_parsed.account_keys,
        &transaction_parsed.ubo,
        &pool_meta.quote_mint.to_string(),
        &pool_meta.base_mint.to_string(),
        &pool_meta.quote_vault.to_string(),
        &pool_meta.base_vault.to_string(),
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
        // decimals_correct, // pool_state,
    );

    if (token_amounts_req.is_none()) {
        println!("Error getting token amounts for pool {}", pool_id_to_get);
        return;
    }

    let token_amounts = token_amounts_req.unwrap();

    let sol_price_db = db_client
        .get_usd_price_sol(transaction_parsed.datetime)
        .unwrap();

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

    let item_to_save = PriceItem {
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
        pool_address: pool_id_to_get.clone(),
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

    // let price_item_c = item_to_save.clone();

    // let reponse = db_client.save_token_values(item_to_save);

    // let tpo_values_a = parse_token_price_oracle_values(
    //     transaction_parsed.ubo.to_string(),
    //     transaction_parsed.signer.to_string(),
    //     pool_id_to_get.to_string(),
    //     pool_meta.base_mint.to_string(),
    //     &token_amounts.token_amounts_quote,
    //     &swap_token_amounts_priced.token_amounts_priced_a,
    //     &signature,
    // );

    // let tpo_values_b = parse_token_price_oracle_values(
    //     transaction_parsed.ubo.to_string(),
    //     transaction_parsed.signer.to_string(),
    //     pool_id_to_get.to_string(),
    //     pool_meta.quote_mint.to_string(),
    //     &token_amounts.token_amounts_base,
    //     &swap_token_amounts_priced.token_amounts_priced_b,
    //     &signature,
    // );

    // let response_token_usd_a = db_client.insert_token_usd_values(&signature, &tpo_values_a);

    // if response_token_usd_a.is_err() {
    //     println!(
    //         "Error saving token usd values to db: {:#?}",
    //         response_token_usd_a
    //     );
    // }

    // let response_token_usd_b = db_client.insert_token_usd_values(&signature, &tpo_values_b);

    // if response_token_usd_b.is_err() {
    //     println!(
    //         "Error saving token usd values to db: {:#?}",
    //         response_token_usd_b
    //     );
    // }

    // if reponse.is_err() {
    //     println!("Error saving to db: {:#?}", reponse);
    // }

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
                    UiInstruction::Compiled(ix) => {
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
                                serde_json::Value::String(data) => {
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
                            if (d.program_id == "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {
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

fn parse_market_from_data(data: String, block_time: i64) -> Option<String> {
    return None;
    // let bytes = match bs58::decode(data).into_vec() {
    //     Ok(b) => b,
    //     Err(_) => return None,
    // };
    // let mut slice: &[u8] = &bytes[16..];

    // let event: Result<MarketMetaDataLog, Error> =
    //     anchor_lang::AnchorDeserialize::deserialize(&mut slice);

    // match event {
    //     Ok(e) => {
    //         let datetime = to_timestampz(block_time as u64);
    //         let new_market = OpenBookMarketMetadata::from_event(e, datetime);
    //         Some(new_market)
    //     }
    //     _ => None,
    // }
}
