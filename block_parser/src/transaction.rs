use std::collections::{HashMap, HashSet};

use crate::{
    actions::parse_token_changes_to_swaps,
    interfaces::{
        CtTransaction,
        TokenChanges,
        TransactionDescription,
        TransactionParsedResponse,
        // TransactionParsed,
        // TransactionParsedResponse,
    },
    token_parser::BalanceHolder,
};
use chrono::DateTime;
use serde_json::{json, Value};

use solana_transaction_status::EncodedTransactionWithStatusMeta;

pub mod innner_test {
    use std::collections::HashMap;

    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    use crate::{
        actions::{CtAction, CtActionFormatted, SwapFieldsFormatted},
        interfaces::{
            BalanceChange, BalanceChangedFormatted, TokenChanges, TokenChangesMapFormatted,
        },
        token_parser::{
            calc_token_usd_total, get_rounded_amount, parse_balance_changes_new, BalanceHolder,
        },
    };

    impl BalanceChange {
        pub fn format(&self) -> BalanceChangedFormatted {
            // let formatted_test = self.balance_post_usd.

            let token_values_formatted = BalanceChangedFormatted {
                owner: self.owner.to_string(),
                mint: self.mint.to_string(),
                balance_post: get_rounded_amount(self.balance_post, 18),
                balance_pre_usd: match self.balance_pre_usd {
                    Some(x) => Some(get_rounded_amount(x, 18)),
                    None => None,
                },
                difference_usd: match self.difference_usd {
                    Some(x) => Some(get_rounded_amount(x, 18)),
                    None => None,
                },
                balance_post_usd: match self.balance_post_usd {
                    Some(x) => Some(get_rounded_amount(x, 18)),
                    None => None,
                },
                balance_pre: get_rounded_amount(self.balance_pre, 18),
                difference: get_rounded_amount(self.difference, 18),
            };

            return token_values_formatted;
        }
    }

    impl CtAction {
        pub fn format(&self) -> CtActionFormatted {
            let tokens_from_formatted: Vec<BalanceChangedFormatted> = self
                .fields
                .tokens_from
                .iter()
                .map(|value| {
                    let formatted = value.format();
                    return formatted;
                })
                .collect();

            let tokens_to_formatted: Vec<BalanceChangedFormatted> = self
                .fields
                .tokens_to
                .iter()
                .map(|value| {
                    let formatted = value.format();
                    return formatted;
                })
                .collect();

            let action_formatted = CtActionFormatted {
                action_type: self.action_type.to_string(),
                protocol_name: self.protocol_name.clone(),
                protocol_id: self.protocol_id.clone(),
                protocol: self.protocol.clone(),
                addresses: self.addresses.clone(),
                event_ids: self.event_ids.clone(),
                u_bwallet_address: self.u_bwallet_address.clone(),
                fields: SwapFieldsFormatted {
                    tokens_from: tokens_from_formatted,
                    tokens_to: tokens_to_formatted,
                    swap_hops: self.fields.swap_hops.clone(),
                    router_events: self.fields.router_events.clone(),
                    testing: self.fields.testing,
                },
            };
            return action_formatted;
        }

        // return self.clone();
    }

    impl TokenChanges {
        pub fn new(
            transaction: &EncodedTransactionWithStatusMeta,
            account_keys: &Vec<String>,
            balance_holder: BalanceHolder,
        ) -> Self {
            let token_changes =
                parse_balance_changes_new(transaction, account_keys, balance_holder);

            // let address = "test".to_string();

            Self {
                values: token_changes,
            }
        }

        pub fn set_prices(&mut self, prices: HashMap<String, String>) {
            let balance_changes_priced: HashMap<
                std::string::String,
                HashMap<std::string::String, BalanceChange>,
            > = self
                .values
                .clone()
                .into_iter()
                .map(|(key, foo)| {
                    (
                        key,
                        foo.into_iter()
                            .map(|(key, value)| {
                                let token_price_o = prices.get(&key.to_string());

                                let balance_pre_priced = calc_token_usd_total(
                                    value.balance_pre,
                                    token_price_o,
                                    value.decimals,
                                );

                                let balance_post_priced = calc_token_usd_total(
                                    value.balance_post,
                                    token_price_o,
                                    value.decimals,
                                );

                                let differnce_priced = calc_token_usd_total(
                                    value.difference,
                                    token_price_o,
                                    value.decimals,
                                );

                                // let value = BigFloat::from_str(&value.balance_post).unwrap();
                                (
                                    key,
                                    BalanceChange {
                                        owner: value.owner.to_string(),
                                        mint: value.mint.to_string(),
                                        balance_post: value.balance_post,
                                        balance_pre_usd: balance_pre_priced,
                                        balance_pre: value.balance_pre,
                                        difference: value.difference,
                                        difference_usd: differnce_priced,
                                        decimals: value.decimals,
                                        balance_post_usd: balance_post_priced,
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect();

            self.values = balance_changes_priced;

            // println!("{:#?}", balance_changes_priced);

            // let changes_prices = self.values.clone();

            // for (key, value) in changes_prices.iter() {
            //     for (key, value) in value.iter() {
            //         let balance_change = value;
            //         let token_address = key.to_string();

            //         let price = prices.get(&token_address);

            //         // balance_change.

            //         // match price {
            //         //     Some(x) => {
            //         //         let price = x.parse::<f64>().unwrap();
            //         //         balance_change.balance_post = price;
            //         //     }
            //         //     None => {
            //         //         balance_change.price = 0.0;
            //         //     }
            //         // }
            //     }
            // }

            // println!("Testing, {:#?}, prices: {:#?}", self.values, prices);
        }

        pub fn format(&self) -> TokenChangesMapFormatted {
            // let values = self.token_changes

            let mut changes_by_owner_formatted = HashMap::new();

            for (key, value) in self.values.iter() {
                let address = key.to_string();

                let mut values = HashMap::new();

                for (key, value) in value.iter() {
                    let token_address = key.to_string();
                    let balance_change = value;
                    let token_values_formatted = balance_change.format();
                    // let token_values_formatted = BalanceChangedFormatted {
                    //     owner: balance_change.owner.to_string(),
                    //     mint: balance_change.mint.to_string(),
                    //     balance_post: get_rounded_amount(balance_change.balance_post, 18),
                    //     balance_pre_usd: match balance_change.balance_pre_usd {
                    //         Some(x) => Some(get_rounded_amount(x, 18)),
                    //         None => None,
                    //     },
                    //     difference_usd: match balance_change.difference_usd {
                    //         Some(x) => Some(get_rounded_amount(x, 18)),
                    //         None => None,
                    //     },
                    //     balance_post_usd: match balance_change.balance_post_usd {
                    //         Some(x) => Some(get_rounded_amount(x, 18)),
                    //         None => None,
                    //     },
                    //     balance_pre: get_rounded_amount(balance_change.balance_pre, 18),
                    //     difference: get_rounded_amount(balance_change.difference, 18),
                    // };

                    values.insert(token_address, token_values_formatted);
                    // println!("{:#?}", testing)
                }

                changes_by_owner_formatted.insert(address, values);
            }

            return changes_by_owner_formatted;

            // let token_values_formatted = BalanceChangedFormatted {
            //     owner: balance_change.owner.to_string(),
            //     mint: balance_change.mint.to_string(),
            //     balance_post: get_rounded_amount(balance_change.balance_post, 18),
            //     balance_pre: get_rounded_amount(balance_change.balance_pre, 18),
            //     difference: get_rounded_amount(balance_change.difference, 18),
            // };
        }
    }
}

impl CtTransaction {
    pub fn testing(&self) -> String {
        return self.block_timestamp.to_string();
    }

    pub fn new(
        rpc_transaction: &EncodedTransactionWithStatusMeta,
        block_time: i64,
        block_number: u64,
    ) -> Self {
        // let token_a_address: &str = "So11111111111111111111111111111111111111112";
        // let token_b_address: &str = "CymqTrLSVZ97v87Z4W3dkF4ipZE1kYyeasmN2VckUL4J";

        let v = json!(rpc_transaction.transaction);
        let account_keys = v["message"]["accountKeys"].as_array().unwrap();

        let transaction_clone_fees = rpc_transaction.clone();
        let transaction_clone_meta = rpc_transaction.clone();

        let transaction_meta = transaction_clone_fees.meta.unwrap();

        let signer = self::find_signer(account_keys);

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

        let token_changes_owner =
            TokenChanges::new(&transaction_clone_meta, &testing, BalanceHolder::Owner);

        let token_changes_token_accounts = TokenChanges::new(
            &transaction_clone_meta,
            &testing,
            BalanceHolder::TokenAccount,
        );

        // let unique_tokens =

        let tokens_set: HashSet<_> = token_changes_owner
            .values
            .values()
            .flat_map(|inner| inner.keys())
            .cloned()
            .collect();

        let tokens = tokens_set.into_iter().collect::<Vec<_>>();

        // let formatted = changes_by_owner_new.format();

        // let (changes_by_owner, changes_by_token_account_address) =
        //     parse_balance_changes(&transaction_clone_meta, &testing.clone());

        Self {
            signer: signer,
            ubo: ubo.to_string(),
            from: singer_c,
            block_timestamp: block_time,
            block_datetime: datetime,
            hash: _signature,
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
            token_changes_owner,
            // token_changes_new: changes_by_owner_new,
            token_changes_token_account: token_changes_token_accounts,
            tokens,
            token_prices: None,
            actions: Vec::new(),
            // token_amounts: token_amounts,
        }
    }

    pub fn set_token_prices(&mut self, token_prices: HashMap<String, String>) {
        self.token_prices = Some(token_prices);
    }

    pub fn format(&self) -> TransactionParsedResponse {
        TransactionParsedResponse {
            signer: self.signer.clone(),
            ubo: self.ubo.clone(),
            block_timestamp: self.block_timestamp,
            block_datetime: self.block_datetime.clone(),
            hash: self.hash.clone(),
            addresses: self.addresses.clone(),
            block_number: self.block_number,
            chain_id: self.chain_id,
            from: self.from.clone(),
            to: self.to.clone(),
            state: self.state.clone(),
            description: self.description.clone(),
            spam_transaction: self.spam_transaction,
            contract_address: self.contract_address.clone(),
            fees: self.fees.clone(),
            fees_total: self.fees_total,
            token_changes_owner: self.token_changes_owner.format(),
            token_changes_token_account: self.token_changes_token_account.format(),
            tokens: self.tokens.clone(),
            actions: self.actions.iter().map(|value| value.format()).collect(),
            // token_prices: self.token_prices.clone(),
        }
    }

    pub fn set_prices_to_token_changes(&mut self, token_prices: HashMap<String, String>) {
        self.token_prices = Some(token_prices.clone());
        self.token_changes_owner.set_prices(token_prices.clone());
        self.token_changes_token_account
            .set_prices(token_prices.clone());

        // self.token_changes_token_account.set_prices(token_prices);
    }

    pub fn create_actions(&mut self) {
        // let mut actions = Vec::new();

        let swaps = parse_token_changes_to_swaps(self.token_changes_owner.values.clone());

        self.actions = swaps;
        // self.set_actions(swaps)
        // return actions;
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
