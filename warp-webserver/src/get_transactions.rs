use moka::future::Cache;
use warp::{
    reject::{custom, Reject},
    reply, Filter, Reply,
};

use crate::User;
use block_parser::{
    interfaces::{PriceItemResponse, TransactionParsed, TransactionParsedResponse},
    rpc_pool_manager::{RpcPool, RpcPoolManager},
    token_db::DbPool,
    token_parser::{get_rounded_amount, PoolMeta},
    transactions_loader,
};

pub async fn get_user(signature: String) -> impl Reply {
    let user = User {
        name: "Test".to_string(),
        age: 30,
        signature,
    };

    // let testing = transactions_loader::get_transction(signature.clone(), None).await;

    warp::reply::json(&user)
}

pub async fn handler(
    pool: RpcPool,
    db_pool: DbPool,
    cache: Cache<String, Option<PoolMeta>>,
    signature: String,
) -> Result<impl Reply, warp::Rejection> {
    let user = User {
        name: "Test".to_string(),
        age: 30,
        signature: "test".to_string(),
    };
    // let custom_rejection = custom("This is a custom rejection".to_string());

    let rpc_connect = pool.get().await.unwrap(); // Get a connection from the pool
    let token_db_connect = db_pool.get().await.unwrap(); // Get a connection from the pool

    let testing = transactions_loader::get_transction(
        signature.clone(),
        None,
        &rpc_connect,
        &token_db_connect,
        cache,
    )
    .await;

    if testing.is_err() {
        return Ok(warp::reply::json(&{}));
    }

    let transaction = testing.unwrap();

    let transaction_formatted = parse_parsed_to_formatted(transaction);

    Ok(warp::reply::json(&transaction_formatted))
}

fn parse_parsed_to_formatted(transaction_parsed: TransactionParsed) -> TransactionParsedResponse {
    let token_prices_response = if (transaction_parsed.token_prices.is_some()) {
        let token_prices = transaction_parsed.token_prices.unwrap();

        let item = PriceItemResponse {
            token_quote_address: token_prices.token_quote_address,
            token_base_address: token_prices.token_base_address,

            // token_new_price_18: get_rounded_amount(token_prices.token_new_price_18, 0),
            // token_new_price_in_token_quote_18: get_rounded_amount(
            //     token_prices.token_new_price_in_token_quote_18,
            //     0,
            // ),
            // token_new_price_fixed: get_rounded_amount(token_prices.token_new_price_fixed, 18),

            // token_new_price_in_token_quote_fixed: get_rounded_amount(
            //     token_prices.token_new_price_in_token_quote_fixed,
            //     18,
            // ),
            token_price_usd_18: get_rounded_amount(token_prices.token_trade_price_18, 0),
            token_trade_price_in_token_quote_18: get_rounded_amount(
                token_prices.token_trade_price_in_token_quote_18,
                0,
            ),
            token_price_usd_fixed: get_rounded_amount(token_prices.token_trade_price_fixed, 18),
            token_trade_price_in_token_quote_fixed: get_rounded_amount(
                token_prices.token_trade_price_in_token_quote_fixed,
                18,
            ),

            usd_total_pool_18: get_rounded_amount(token_prices.usd_total_pool, 0),
            pool_address: token_prices.pool_address,
        };

        let vec = vec![item];

        Some(vec)
    } else {
        None
    };

    let transaction_response = TransactionParsedResponse {
        signer: transaction_parsed.signer,
        ubo: transaction_parsed.ubo,
        block_timestamp: transaction_parsed.block_timestamp,
        block_datetime: transaction_parsed.block_datetime,
        hash: transaction_parsed.hash,
        addresses: transaction_parsed.addresses,
        block_number: transaction_parsed.block_number,
        chain_id: transaction_parsed.chain_id,
        from: transaction_parsed.from,
        to: transaction_parsed.to,
        state: transaction_parsed.state,
        description: transaction_parsed.description,
        spam_transaction: transaction_parsed.spam_transaction,
        contract_address: transaction_parsed.contract_address,
        fees: transaction_parsed.fees,
        fees_total: transaction_parsed.fees_total,
        token_prices: token_prices_response,
    };

    return transaction_response;
}
