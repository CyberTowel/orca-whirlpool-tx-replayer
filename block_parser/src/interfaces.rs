use std::collections::HashMap;

use num_bigfloat::BigFloat;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionDescription {
    pub title: String,
    pub subtitle: String,
    pub emoji: String,
    pub transaction_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFees {
    pub amount: String,
    pub fee_type: String,
}

pub enum ParsedTransaction {
    Parsed(TransactionParsed),
    ParsedResponse(TransactionParsedResponse),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionBase {
    pub signer: String,
    pub ubo: String,
    pub block_timestamp: i64,
    pub block_datetime: String,
    pub hash: String,
    pub addresses: Vec<String>,
    pub block_number: u64,
    pub chain_id: i16,
    pub from: String,
    pub to: Option<String>,
    pub state: String,
    pub description: TransactionDescription,
    pub spam_transaction: bool,
    pub contract_address: Vec<String>,
    pub fees: Vec<TransactionFees>,
    pub fees_total: u64,
    pub changes_by_owner: HashMap<String, HashMap<String, BalanceChange>>,
    pub changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChange>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionParsed {
    pub signer: String,
    pub ubo: String,
    pub block_timestamp: i64,
    pub block_datetime: String,
    pub hash: String,
    pub addresses: Vec<String>,
    pub block_number: u64,
    pub chain_id: i16,
    pub from: String,
    pub to: Option<String>,
    pub state: String,
    pub description: TransactionDescription,
    pub spam_transaction: bool,
    pub contract_address: Vec<String>,
    pub fees: Vec<TransactionFees>,
    pub fees_total: u64,
    pub token_prices: Option<PriceItem>,
    pub changes_by_owner: HashMap<String, HashMap<String, BalanceChange>>,
    pub changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChange>>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionParsedResponse {
    pub signer: String,
    pub ubo: String,
    pub block_timestamp: i64,
    pub block_datetime: String,
    pub hash: String,
    pub addresses: Vec<String>,
    pub block_number: u64,
    pub chain_id: i16,
    pub from: String,
    pub to: Option<String>,
    pub state: String,
    pub description: TransactionDescription,
    pub spam_transaction: bool,
    pub contract_address: Vec<String>,
    pub fees: Vec<TransactionFees>,
    pub fees_total: u64,
    pub token_prices: Option<Vec<PriceItemResponse>>,
    // pub changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChangedFormatted>>, // pub actions: Vec<Action>,
    pub changes_by_owner_formatted: HashMap<String, HashMap<String, BalanceChangedFormatted>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PriceItem {
    pub signature: String,
    pub token_quote_address: String,
    pub token_base_address: String,

    pub token_new_price_18: BigFloat,
    pub token_new_price_in_token_quote_18: BigFloat,
    pub token_new_price_fixed: BigFloat,
    pub token_new_price_in_token_quote_fixed: BigFloat,

    pub token_trade_price_18: BigFloat,
    pub token_trade_price_in_token_quote_18: BigFloat,
    pub token_trade_price_fixed: BigFloat,
    pub token_trade_price_in_token_quote_fixed: BigFloat,

    pub usd_total_pool: BigFloat,

    pub datetime: String,
    pub signer: String,
    pub ubo: String,
    pub pool_address: String,
    pub block_number: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    action_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PriceItemResponse {
    pub token_quote_address: String,
    pub token_base_address: String,

    // pub token_new_price_18: String,
    // pub token_new_price_in_token_quote_18: String,
    // pub token_new_price_fixed: String,
    // pub token_new_price_in_token_quote_fixed: String,
    pub token_price_usd_18: String,
    pub token_trade_price_in_token_quote_18: String,
    pub token_price_usd_fixed: String,
    pub token_trade_price_in_token_quote_fixed: String,

    pub usd_total_pool_18: String,
    pub pool_address: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceChange {
    pub owner: String,
    pub mint: String,
    pub balance_pre: BigFloat,
    pub balance_post: BigFloat,
    pub difference: BigFloat,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceChangedFormatted {
    pub owner: String,
    pub mint: String,
    pub balance_pre: String,
    pub balance_post: String,
    pub difference: String,
}
