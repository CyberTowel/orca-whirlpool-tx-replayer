use std::collections::HashMap;

use crate::{
    interfaces::{
        BalanceChange, TransactionBase, TransactionDescription, TransactionParsedResponse,
    },
    token_parser::parse_balance_changes,
};
use chrono::DateTime;
use serde_json::{json, Value};
use solana_sdk::transaction;
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedTransactionWithStatusMeta,
    UiTransactionTokenBalance,
};

impl TransactionBase {
    pub fn new(
        rpc_transaction: &EncodedTransactionWithStatusMeta,
        block_time: i64,
        block_number: u64,
    ) -> TransactionBase {
        // let token_a_address: &str = "So11111111111111111111111111111111111111112";
        // let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

        let v = json!(rpc_transaction.transaction);
        let account_keys = v["message"]["accountKeys"].as_array().unwrap();

        let transaction_clone_fees = rpc_transaction.clone();
        let transaction_clone_meta = rpc_transaction.clone();

        let transaction_meta = transaction_clone_fees.meta.unwrap();

        let signer = find_signer(account_keys);

        let testing = account_keys
            .to_vec()
            .iter()
            .map(|value| value["pubkey"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

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

        let datetime = DateTime::from_timestamp(block_time, 0)
            .unwrap()
            .to_rfc3339();

        let signature = json!(rpc_transaction.transaction);

        let _signature = signature["signatures"][0].as_str().unwrap().to_string();

        let fee = transaction_meta.fee;

        // let token_changes = rpc_transaction.met

        // println!("{:?}", changes_by_owner);
        // println!("{:?}", changes_by_token_account_address);

        let (changes_by_owner, changes_by_token_account_address) =
            parse_balance_changes(&transaction_clone_meta, &testing.clone());

        let transaction = TransactionBase {
            signer: signer,
            ubo: ubo.to_string(),
            from: singer_c,
            block_timestamp: block_time,
            block_datetime: datetime,
            hash: _signature,

            // token_a_address: token_a_address.to_string(),
            // token_b_address: token_b_address.to_string(),
            to: None,
            addresses: testing.clone(),
            block_number: block_number,
            chain_id: 10001,
            state: "parsed".to_string(),
            description: TransactionDescription {
                title: "todo".to_string(),
                subtitle: "todo".to_string(),
                emoji: "📈".to_string(),
                transaction_type: "todo".to_string(),
            },
            spam_transaction: false,
            contract_address: Vec::new(),
            fees: Vec::new(),
            fees_total: fee,
            changes_by_owner: changes_by_owner,
            changes_by_token_account_address: changes_by_token_account_address,
            // token_amounts: token_amounts,
        };

        return transaction;

        // pub fn parse_transaction_actions(&self) {
        //     // let actions = parse_token_changes_to_swaps(self);
        //     // let token_a_address
        // }

        pub fn find_signer(account_keys: &Vec<Value>) -> String {
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
    }

    // fn get_transaction_datetime(transaction: &EncodedTransactionWithStatusMeta) -> String {
    //     let transaction_datetime = DateTime::from_timestamp(transaction.block_time.unwrap(), 0)
    //         .unwrap()
    //         .to_rfc3339();

    //     return transaction_datetime;
    // }

    // pub fn parse_pool_create_instruction(){

    //     let mut testing;

    //     for item in
}
