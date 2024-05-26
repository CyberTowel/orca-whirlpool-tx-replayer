use crate::{
    interfaces::{
        CtTransaction,
        PriceItem,
        // TransactionParsed
    },
    rpc_pool_manager::RpcPool,
    token_db::{get_token_prices_from_token_changes, DbPool},
};
use chrono::prelude::*;
use moka::future::Cache;

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
        PoolMeta,
    },
};

pub fn testing_nested() {}
#[derive(Debug)]
pub struct TransactionError {
    pub message: String,
}

pub async fn get_transction(
    signature: String,
    // pool_id: Option<String>,
    rpc_connection: &RpcClient,
    // db_client: &TokenDbClient,
    // my_cache: Cache<String, Option<PoolMeta>>,
) -> Result<CtTransaction, TransactionError> {
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

    let mut _transaction_base = CtTransaction::new(&transaction, block_time, block_number);

    _transaction_base.create_actions();

    return Ok(_transaction_base);
}

pub enum Error {
    Msg(String),
}

pub async fn to_replace_parse_transaction_and_save_values(
    signature: String,
    pool_id: Option<String>,
    _rpc_connection: &RpcClient,
    rpc_connection_build: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: &Cache<String, Option<PoolMeta>>,
    transaction: &EncodedTransactionWithStatusMeta,
    block_time: i64,
    block_number: u64,
    sol_price_18: Option<String>,
) {
    let _transaction_parsed = to_archive_get_parsed_transaction(
        signature,
        pool_id,
        _rpc_connection,
        rpc_connection_build,
        db_client,
        my_cache,
        transaction,
        block_time,
        block_number,
        sol_price_18,
    );
}

// pub async fn get_transaction_base(
//     transaction: &EncodedTransactionWithStatusMeta,
//     block_time: i64,
//     block_number: u64,
// ) -> Result<CtTransaction, TransactionError> {
//     let transaction_base = CtTransaction::new(transaction, block_time, block_number);

//     return Ok(transaction_base);
// }

// pub async fn get_token_pricing();

pub async fn to_archive_get_parsed_transaction(
    signature: String,
    pool_id: Option<String>,
    _rpc_connection: &RpcClient,
    rpc_connection_build: &RpcClient,
    db_client: &TokenDbClient,
    my_cache: &Cache<String, Option<PoolMeta>>,
    transaction: &EncodedTransactionWithStatusMeta,
    block_time: i64,
    block_number: u64,
    sol_price_18: Option<String>,
    // transaction_base: CtTransaction,
) -> Result<CtTransaction, TransactionError> {
    let transaction_base = CtTransaction::new(transaction, block_time, block_number);

    return Ok(transaction_base);

    // let testing = transaction_base.token_changes_new.format();
    let _transaction_datetime = DateTime::from_timestamp(block_time.clone(), 0)
        .unwrap()
        .to_rfc3339();

    let v = json!(transaction.transaction);

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
            let pool_id_instruction = init_instruction.unwrap().as_object().unwrap()["accounts"][1]
                .as_str()
                .unwrap();
            Some(pool_id_instruction.to_string())
        } else {
            let dolar_selit = find_raydium_inner_instruction(&transactions_meta.inner_instructions);

            if dolar_selit.len() > 0 {
                // return None;
                Some(dolar_selit[1].to_string())
            } else {
                None
            }
        }
    };

    // let actions = TransactionBase::parse_transaction_actions(&transaction_base);

    if pool_id_to_get_opt.is_none() {
        // let transaction_parsed = parse_base_to_parsed(transaction_base, None);

        return Ok(transaction_base);
        // return Err(Error::Msg("Error in transaction".to_string()));
    }

    let pool_id_to_get = pool_id_to_get_opt.unwrap();

    let pool_id_clone = pool_id_to_get.clone();

    let pool_info_cache = my_cache.get(&pool_id_to_get).await;

    let pool_meta_req = if pool_info_cache.is_some() {
        let info = pool_info_cache.unwrap();
        Some(info)
    } else {
        let info = my_cache
            .get_with(pool_id_to_get.clone(), async move {
                let inf_req = get_pool_meta(&pool_id_to_get, rpc_connection_build).await;
                // Arc::new(vec![0u8; TEN_MIB])

                if inf_req.is_none() {
                    return None;
                }

                Some(inf_req.unwrap())
            })
            .await;

        Some(info)
    };

    if !pool_meta_req.is_some() {
        return Ok(transaction_base);
    }

    let pool_meta_opt = pool_meta_req.unwrap();

    if !pool_meta_opt.is_some() {
        return Ok(transaction_base);
    }

    let pool_meta = pool_meta_opt.unwrap();

    let transaction_parsed_meta = transaction_base.clone();

    let sol_price_db = if sol_price_18.is_some() {
        sol_price_18.unwrap()
    } else {
        db_client
            .get_token_price_usd(
                &transaction_parsed_meta.block_datetime,
                "So11111111111111111111111111111111111111112".to_string(),
            )
            .unwrap()
    };

    let token_amounts_req = get_token_amounts(
        &transaction_base,
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
        return Ok(transaction_base);
    }

    let token_amounts = token_amounts_req.unwrap();

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

    let datetime = DateTime::from_timestamp(transaction_base.block_timestamp, 0)
        .unwrap()
        .to_rfc3339();

    if transactions_meta.err.is_some() {
        return Ok(transaction_base);
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
        signer: transaction_base.signer.to_string(),
        ubo: transaction_base.ubo.to_string(),
        pool_address: pool_id_clone.clone(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18,
        block_number: transaction_base.block_number.to_string(),
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

    // let transaction_parsed = parse_base_to_parsed(transaction_base, Some(_price_item_c));

    return Ok(transaction_base);

    // return (
    //     signature.to_string(),
    //     price_item_c.datetime,
    //     "success".to_string(),
    // );
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
                        panic!("inplement this UiParsedInstruction Compiled")
                    }
                    UiInstruction::Parsed(ix) => match ix {
                        UiParsedInstruction::Parsed(x) => match x.parsed.get("data") {
                            Some(d) => match d {
                                serde_json::Value::String(_data) => {
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

// fn parse_base_to_parsed(
//     transaction_base: CtTransaction,
//     _price_item_c: Option<PriceItem>,
// ) -> TransactionParsed {
//     let transaction_parsed = TransactionParsed {
//         signer: transaction_base.signer,
//         ubo: transaction_base.ubo,
//         block_timestamp: transaction_base.block_timestamp,
//         block_datetime: transaction_base.block_datetime,
//         hash: transaction_base.hash,
//         addresses: transaction_base.addresses,
//         block_number: transaction_base.block_number,
//         chain_id: transaction_base.chain_id,
//         from: transaction_base.from,
//         to: transaction_base.to,
//         state: transaction_base.state,
//         description: transaction_base.description,
//         spam_transaction: transaction_base.spam_transaction,
//         contract_address: transaction_base.contract_address,
//         fees: transaction_base.fees,
//         fees_total: transaction_base.fees_total,
//         token_prices: _price_item_c,
//         token_changes_owner: transaction_base.token_changes_owner,
//         token_changes_token_account: transaction_base.token_changes_token_account,
//         actions: Vec::new(),
//     };

//     return transaction_parsed;
// }

pub async fn get_transaction_priced(
    pool: RpcPool,
    db_pool: DbPool,
    cache: Cache<String, Option<PoolMeta>>,
    signature: String,
) -> Result<CtTransaction, TransactionError> {
    println!("starting get_transaction_priced, {}", signature.clone());
    let rpc_connect = pool.get().await.unwrap(); // Get a connection from the pool

    let rpc_response = get_transction(
        signature.clone(),
        // None,
        &rpc_connect,
        // &token_db_connect,
        // cache,
    )
    .await;

    if rpc_response.is_err() {
        return Err(TransactionError {
            message: "Error in transaction".to_string(),
        });
        // return Ok(warp::reply::json(&{}));
    }

    let mut transaction = rpc_response.unwrap();
    let transaction_values_c = transaction.clone();

    let token_prices = get_token_prices_from_token_changes(
        transaction_values_c.block_datetime,
        transaction_values_c.tokens,
        db_pool,
    )
    .await;

    // transaction.set_token_prices(token_prices);
    transaction.set_prices_to_token_changes(token_prices);
    transaction.create_actions();
    Ok(transaction)
}
