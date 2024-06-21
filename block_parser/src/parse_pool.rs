use deadpool::managed::Object;
use moka::future::Cache;
use serde_json::Value;
use solana_sdk::{signature, signers};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedTransactionWithStatusMeta, UiInnerInstructions,
    UiInstruction, UiParsedInstruction, UiTransactionEncoding,
};

use crate::{
    interfaces::{CtTransaction, PriceItem},
    pool_state::get_pool_meta,
    rpc_pool_manager::RpcPoolManager,
    token_db::DbClientPoolManager,
    token_parser::{
        get_price, get_token_amounts, parse_token_amounts_new, parse_token_price_oracle_values,
        PoolMeta, TokenAmountsSwap, TokenPriceResult,
    },
};

pub async fn parse_pool_price(
    transaction: CtTransaction,
    rpc_connect: Object<RpcPoolManager>,
    db_client: Object<DbClientPoolManager>,
    _cache: Cache<String, Option<PoolMeta>>,
    sol_price_18: Option<String>,
) -> Option<PriceItem> {
    let pool_id_to_get_opt: Option<String> = get_pool_id(
        transaction.instructions.clone(),
        transaction.inner_instructions.clone(),
    );

    if !pool_id_to_get_opt.is_some() {
        // println!("No pool id found in transaction");
        return None;
    }

    let pool_id_to_get = pool_id_to_get_opt.unwrap();

    let pool_id_clone = pool_id_to_get.clone();

    let pool_info_cache = _cache.get(&pool_id_to_get).await;

    let pool_meta_req = if pool_info_cache.is_some() {
        let info = pool_info_cache.unwrap();
        Some(info)
    } else {
        let info = _cache
            .get_with(pool_id_to_get.clone(), async move {
                let inf_req = get_pool_meta(&pool_id_to_get, &rpc_connect).await;
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
        // println!("No pool_meta_req: {}", pool_id_clone);
        return None;
        // return Ok(transaction_base);
    }

    let pool_meta_opt = pool_meta_req.unwrap();

    let pool_meta = pool_meta_opt.unwrap();

    let sol_price_db = if sol_price_18.is_some() {
        sol_price_18.unwrap()
    } else {
        db_client
            .get_token_price_usd(
                &transaction.block_datetime,
                "So11111111111111111111111111111111111111112".to_string(),
            )
            .unwrap()
    };

    let token_amounts_req = get_token_amounts(
        &transaction,
        &transaction.addresses,
        &transaction.ubo,
        &pool_meta.quote_mint.to_string(),
        &pool_meta.base_mint.to_string(),
        &pool_meta.quote_vault.to_string(),
        &pool_meta.base_vault.to_string(),
        pool_meta.quote_decimal,
        pool_meta.base_decimal,
        // decimals_correct, // pool_state,
    );

    if token_amounts_req.is_none() {
        // return Ok(transaction_base);
        return None;
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

    // println!("Token prices: {:#?}", token_prices.format());

    let swap_token_amounts_priced = parse_token_amounts_new(&token_amounts, &token_prices);

    let token_price_values = PriceItem {
        transaction_hash: transaction.hash.to_string(),
        token_address_pool_ref: pool_meta.quote_mint.to_string(),
        token_address: pool_meta.base_mint.to_string(),

        price: token_prices.token_new_price_18,
        price_fixed: token_prices.token_new_price_fixed,
        price_in_token_quote_18: token_prices.token_new_price_in_token_quote_18,
        price_in_token_quote_fixed: token_prices.token_new_price_in_token_quote_fixed,

        trade_price: token_prices.token_trade_price_18,
        trade_price_fixed: token_prices.token_trade_price_fixed,
        trade_price_in_token_quote_18: token_prices.token_trade_price_in_token_quote_18,
        trade_price_in_token_quote_fixed: token_prices.token_trade_price_in_token_quote_fixed,

        datetime: transaction.block_datetime,
        signer: transaction.signer.to_string(),
        ubo: transaction.ubo.to_string(),
        pool_address: pool_id_clone.clone(),
        usd_total_pool: swap_token_amounts_priced.usd_total_pool_18,
        blocknumber: transaction.block_number.to_string(),
    };

    let reponse = db_client.save_token_values(token_price_values.clone());

    if (reponse.is_err()) {
        println!("Error saving price item: {:#?}", reponse);
    }

    let tpo_values_a = parse_token_price_oracle_values(
        transaction.ubo.to_string(),
        transaction.signer.to_string(),
        pool_id_clone.to_string(),
        pool_meta.base_mint.to_string(),
        &token_amounts.token_amounts_quote,
        &swap_token_amounts_priced.token_amounts_priced_a,
        &transaction.hash,
    );

    let tpo_values_b = parse_token_price_oracle_values(
        transaction.ubo.to_string(),
        transaction.signer.to_string(),
        pool_id_clone.to_string(),
        pool_meta.quote_mint.to_string(),
        &token_amounts.token_amounts_base,
        &swap_token_amounts_priced.token_amounts_priced_b,
        &transaction.hash,
    );

    let response_token_usd_a = db_client.insert_token_usd_values(&transaction.hash, &tpo_values_a);

    if response_token_usd_a.is_err() {
        println!("Error saving token usd values: {:#?}", response_token_usd_a);
    }

    let response_token_usd_b = db_client.insert_token_usd_values(&transaction.hash, &tpo_values_b);

    if response_token_usd_b.is_err() {
        println!("Error saving token usd values: {:#?}", response_token_usd_b);
    }

    return Some(token_price_values);
}

pub fn get_pool_id(
    instructions: Vec<Value>,
    inner_instructions: OptionSerializer<Vec<UiInnerInstructions>>,
) -> Option<String> {
    let pool_id_to_get_opt: Option<String> = {
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
            let dolar_selit = find_raydium_inner_instruction(&inner_instructions);

            if dolar_selit.len() > 0 {
                // return None;
                Some(dolar_selit[1].to_string())
            } else {
                None
            }
        }
    };

    return pool_id_to_get_opt;
}

pub fn find_raydium_inner_instruction(
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
