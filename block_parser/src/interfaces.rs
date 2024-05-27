use std::{clone, collections::HashMap};

use deadpool_postgres::Transaction;
use num_bigfloat::BigFloat;

use crate::actions::{CtAction, CtActionFormatted};

#[derive(serde::Deserialize, Debug, clone::Clone)]
pub struct ArrayMapRequest {
    // an_array: Vec<String>,
    pub expand: Option<Vec<String>>,
    // testing: bool,
    // a_map: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenChanges {
    pub values: TokenChangesMap,
    // pub by_token_account: MappedTokenChanges,
}

pub type TokenChangesMap =
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>;

// pub type TokenChangesMapPriced =
//     HashMap<std::string::String, HashMap<std::string::String, BalanceChangePriced>>;

pub type TokenChangesMapFormatted =
    HashMap<std::string::String, HashMap<std::string::String, BalanceChangedFormatted>>;

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
    pub amount_bf: BigFloat,
    pub description: String,
    pub token: String,
    pub payer: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFeesFormatted {
    pub amount: String,
    pub description: String,
    pub token: String,
    pub payer: String,
}

impl TransactionFees {
    pub fn new(
        amount: String,
        amount_18: BigFloat,
        description: String,
        token: String,
        payer: String,
    ) -> Self {
        Self {
            amount,
            amount_bf: amount_18,
            description,
            token,
            payer,
        }
    }

    pub fn format(&self) -> TransactionFeesFormatted {
        TransactionFeesFormatted {
            amount: self.amount.clone(),
            // amount_18: self.amount_bf.to_f64().to_string(),
            description: self.description.clone(),
            token: self.token.clone(),
            payer: self.payer.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CtTransaction {
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
    // pub changes_by_owner: HashMap<String, HashMap<String, BalanceChange>>,
    pub token_changes_token_account: TokenChanges,
    pub token_changes_owner: TokenChanges,
    pub tokens: Vec<String>,
    pub token_prices: Option<HashMap<String, String>>,
    pub actions: Vec<CtAction>,
}

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// // pub struct TransactionParsed {
// //     pub transaction: CtTransaction,
// //     pub actions: Vec<Action>,
// // }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionParsedResponse {
    pub signer: String,
    pub ubo: String,
    pub block_timestamp: i64,
    pub block_datetime: String,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    pub block_number: u64,
    pub chain_id: i16,
    pub from: String,
    pub to: Option<String>,
    pub state: String,
    pub description: TransactionDescription,
    pub spam_transaction: bool,
    pub contract_address: Vec<String>,
    pub fees: Vec<TransactionFeesFormatted>,
    pub fees_total: u64,
    // pub token_prices: Option<Vec<PriceItemResponse>>,
    // pub changes_by_token_account_address: HashMap<String, HashMap<String, BalanceChangedFormatted>>, // pub actions: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_changes_owner: Option<TokenChangesMapFormatted>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_changes_token_account: Option<TokenChangesMapFormatted>,
    pub tokens: Vec<String>,
    pub actions: ActionsFormatted,
}

pub type Actions = Vec<CtAction>;
pub type ActionsFormatted = Vec<CtActionFormatted>;

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

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct Action {
//     action_type: String,
// }

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
    pub balance_pre_usd: Option<BigFloat>,
    pub balance_post_usd: Option<BigFloat>,
    pub balance_post: BigFloat,
    pub difference: BigFloat,
    pub difference_usd: Option<BigFloat>,
    pub decimals: u8,
    pub fee: Option<Vec<TransactionFees>>,
    pub value_change: BigFloat,
    pub value_change_usd: Option<BigFloat>,
}

// #[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct BalanceChangePriced {
//     pub owner: String,
//     pub mint: String,
//     pub balance_pre: BigFloat,
//     pub balance_pre_priced: Option<BigFloat>,
//     pub balance_post: BigFloat,
//     pub difference: BigFloat,
// }

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceChangedFormatted {
    pub owner: String,
    pub mint: String,
    pub balance_pre: String,
    pub balance_pre_usd: Option<String>,
    pub balance_post: String,
    pub balance_post_usd: Option<String>,
    pub difference: String,
    pub value_change: String,
    pub value_change_usd: Option<String>,
    pub difference_usd: Option<String>,
    pub fee: Option<Vec<TransactionFeesFormatted>>,
}
