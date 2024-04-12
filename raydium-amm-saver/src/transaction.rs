use chrono::DateTime;
use serde_json::{json, Value};

use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

#[derive(Debug)]
pub struct Transaction {
    pub is_a_parrot: bool,
    // rpc_data: &'a EncodedConfirmedTransactionWithStatusMeta,
    pub signer: String,
    pub ubo: String,
    pub block_timestamp: i64,
    pub datetime: String,
    // pub token_amounts: TokenAmounts,
    pub token_a_address: String,
    pub token_b_address: String,
    pub account_keys: Vec<Value>,
}

impl Transaction {
    pub fn new(rpc_transaction: &EncodedConfirmedTransactionWithStatusMeta) -> Transaction {
        let token_a_address: &str = "So11111111111111111111111111111111111111112";
        let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

        let v = json!(rpc_transaction.transaction.transaction);
        let account_keys = v["message"]["accountKeys"].as_array().unwrap();

        let signer = find_signer(account_keys);

        let instructions = v["message"]["instructions"].as_array().unwrap();

        let dca_instruction = instructions.iter().find(|&x| {
            let program_id = x["programId"].as_str().unwrap();
            let data = x["data"].as_str();

            let date_test = match data {
                Some(x) => x.to_lowercase(),
                None => "".to_string(),
            };

            program_id == "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M"
                && date_test == "ujjfjrldfld"
        });

        let singer_c = signer.clone();

        let ubo = match dca_instruction {
            Some(x) => x["accounts"].as_array().unwrap()[2].as_str().unwrap(),
            None => &singer_c,
        };

        let datetime = get_transaction_datetime(rpc_transaction);

        let transaction = Transaction {
            is_a_parrot: true,
            signer: signer,
            ubo: ubo.to_string(),
            block_timestamp: rpc_transaction.block_time.unwrap(),
            datetime: datetime,
            token_a_address: token_a_address.to_string(),
            token_b_address: token_b_address.to_string(),
            account_keys: account_keys.clone(),
            // token_amounts: token_amounts,
        };

        return transaction;
    }
}

fn find_signer(account_keys: &Vec<Value>) -> String {
    let mut signer: String = "".to_string();

    for item in account_keys {
        let is_signer = item["signer"].as_bool().unwrap();

        if is_signer {
            signer = item["pubkey"].as_str().unwrap().to_string();
            break;
        }
    }

    return signer;
}

fn get_transaction_datetime(transaction: &EncodedConfirmedTransactionWithStatusMeta) -> String {
    let transaction_datetime = DateTime::from_timestamp(transaction.block_time.unwrap(), 0)
        .unwrap()
        .to_rfc3339();

    return transaction_datetime;
}
