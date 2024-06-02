use crate::{
    actions::combine_token_transfers,
    ct_swap::parse_events_to_swap,
    ct_transfer::parse_value_changes_to_transfers,
    interfaces::{
        BalanceChange,
        BalanceChangedFormatted,
        CtTransaction,
        TokenChanges,
        TransactionDescription,
        TransactionFees,
        TransactionParsedResponse,
        ValueChange,
        ValueChangeFormatted, // TransactionParsed,
                              // TransactionParsedResponse,
    },
    token_parser::BalanceHolder,
};
use chrono::DateTime;
use num::ToPrimitive;
use num_bigfloat::BigFloat;
use serde_json::value::Value;
use serde_json::{json, to_string};
use solana_sdk::blake3::Hash;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionTokenBalance};
use std::collections::{HashMap, HashSet};

pub mod innner_test {
    use solana_transaction_status::EncodedTransactionWithStatusMeta;
    use std::collections::HashMap;

    use crate::{
        interfaces::{
            BalanceChange, BalanceChangedFormatted, TokenChanges, TokenChangesMapFormatted,
            TransactionFees,
        },
        token_parser::{
            calc_token_usd_total, get_rounded_amount, parse_balance_changes_new, BalanceHolder,
        },
    };

    impl BalanceChange {
        pub fn format(&self) -> BalanceChangedFormatted {
            let solar_selit = if self.fees.is_some() {
                let testingsdf = self
                    .fees
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|value| value.format())
                    .collect();

                Some(testingsdf)
            } else {
                None
            };

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
                fees: solar_selit,
                value_transferred: get_rounded_amount(self.value_transferred, 18),
                value_transferred_usd: match self.value_transferred_usd {
                    Some(x) => Some(get_rounded_amount(x, 18)),
                    None => None,
                },
                inner_changes: self.inner_changes.clone().map(|value| {
                    value
                        .iter()
                        .map(|value| value.format())
                        .collect::<Vec<BalanceChangedFormatted>>()
                }),
            };

            return token_values_formatted;
        }
    }

    // impl CtAction {
    //     pub fn format(&self) -> CtActionFormatted {
    //         let fields: ActionFieldsFormatted = match &self.fields {
    //             ActionFields::CtSwap(fields) => {
    //                 let tokens_from_formatted: Vec<BalanceChangedFormatted> = fields
    //                     .tokens_from
    //                     .iter()
    //                     .map(|value| {
    //                         let formatted = value.format();
    //                         return formatted;
    //                     })
    //                     .collect();

    //                 let tokens_to_formatted: Vec<BalanceChangedFormatted> = fields
    //                     .tokens_to
    //                     .iter()
    //                     .map(|value| {
    //                         let formatted = value.format();
    //                         return formatted;
    //                     })
    //                     .collect();

    //                 let fields_formatted = SwapFieldsFormatted {
    //                     tokens_from: tokens_from_formatted,
    //                     tokens_to: tokens_to_formatted,
    //                     swap_hops: fields.swap_hops.clone(),
    //                     router_events: fields.router_events.clone(),
    //                     testing: fields.testing,
    //                 };

    //                 let tesitng = ActionFieldsFormatted::CtSwap(fields_formatted);
    //                 tesitng
    //             }

    //             ActionFields::CtTransfer(fields) => {
    //                 let tokens_transferred_formatted: Vec<BalanceChangedFormatted> = fields
    //                     .tokens_transferred
    //                     .iter()
    //                     .map(|value| {
    //                         let formatted = value.format();
    //                         return formatted;
    //                     })
    //                     .collect();

    //                 let fields_formatted = TransferFieldsFormatted {
    //                     tokens_transferred: tokens_transferred_formatted,
    //                     router_events: fields.router_events.clone(),
    //                     testing: fields.testing,
    //                 };

    //                 let tesitng = ActionFieldsFormatted::CtTransfer(fields_formatted);
    //                 tesitng
    //             }
    //         };

    //         let action_formatted = CtActionFormatted {
    //             action_type: self.action_type.to_string(),
    //             protocol_name: self.protocol_name.clone(),
    //             protocol_id: self.protocol_id.clone(),
    //             protocol: self.protocol.clone(),
    //             addresses: self.addresses.clone(),
    //             event_ids: self.event_ids.clone(),
    //             u_bwallet_address: self.u_bwallet_address.clone(),
    //             fields: fields,
    //         };
    //         return action_formatted;
    //     }

    //     // return self.clone();
    // }

    impl TokenChanges {
        pub fn new(
            transaction: &EncodedTransactionWithStatusMeta,
            account_keys: &Vec<String>,
            balance_holder: BalanceHolder,
            fees: HashMap<String, TransactionFees>,
            ubo: &str,
        ) -> Self {
            let token_changes =
                parse_balance_changes_new(transaction, account_keys, balance_holder, fees, ubo);

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

                                let value_changed_price = calc_token_usd_total(
                                    value.value_transferred,
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
                                        fees: value.fees.clone(),
                                        value_transferred: value.value_transferred,
                                        value_transferred_usd: value_changed_price,
                                        inner_changes: value.inner_changes.clone(),
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect();

            self.values = balance_changes_priced;

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

        let singer_c = signer.clone();

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

        let mut ubo: String = signer.clone();

        if dca_instruction.is_some() {
            let x = dca_instruction.unwrap();
            ubo = x["accounts"].as_array().unwrap()[2]
                .as_str()
                .unwrap()
                .to_string();
        } else {
            let instruction = instructions.iter().find(|&x| {
                let program_id = x["programId"].as_str().unwrap();
                let data = x["data"].as_str();

                let date_test = match data {
                    Some(x) => x.to_lowercase(),
                    None => "".to_string(),
                };

                program_id == "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M"
                    && date_test == "Exy129dPfhn".to_lowercase()
            });

            if instruction.is_some() {
                let x = instruction.unwrap();
                ubo = x["accounts"].as_array().unwrap()[6]
                    .as_str()
                    .unwrap()
                    .to_string();
            }
        }

        // let ubo = match dca_instruction {
        //     Some(x) => x["accounts"].as_array().unwrap()[2].as_str().unwrap(),
        //     None => &singer_c,
        // };

        let datetime = DateTime::from_timestamp(block_time, 0)
            .unwrap()
            .to_rfc3339();

        let signature = json!(rpc_transaction.transaction);

        let _signature = signature["signatures"][0].as_str().unwrap().to_string();

        let fee = transaction_meta.fee;

        let fee_big_float = BigFloat::from_u64(fee); //.mul(&BigFloat::from(10).pow(&BigFloat::from(9)));

        let transactie_fee = TransactionFees::new(
            fee.to_string(),
            fee_big_float,
            "Transaction fee".to_string(),
            "sol".to_string(),
            signer.clone(),
        );

        let fees = vec![transactie_fee];

        let fees_map = fees
            .iter()
            .map(|value| {
                (
                    value.payer.clone() + "##" + &value.token.clone(),
                    value.clone(),
                )
            })
            .collect::<HashMap<String, TransactionFees>>();

        let token_changes_owner = TokenChanges::new(
            &transaction_clone_meta,
            &testing,
            BalanceHolder::Owner,
            fees_map.clone(),
            &ubo,
        );
        // let mut serializer = Serializer::new();

        let token_changes_token_accounts = TokenChanges::new(
            &transaction_clone_meta,
            &testing,
            BalanceHolder::TokenAccount,
            fees_map.clone(),
            &ubo,
        );

        let post_token_balances: Option<Vec<UiTransactionTokenBalance>> =
            transaction_meta.post_token_balances.into();

        let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> =
            transaction_meta.pre_token_balances.into();

        let mut token_account_owners = HashMap::new();

        for balance in post_token_balances.unwrap() {
            let owner: Option<String> = balance.owner.clone().into();
            let index_usize = balance.account_index.to_usize().unwrap();
            let pub_key_token_address = account_keys[index_usize].clone();

            let dolar = pub_key_token_address["pubkey"].as_str().unwrap();

            // let testing: Option<&str> = dolar.to_string();

            token_account_owners.insert(dolar.to_string(), owner.unwrap());
            // println!(
            //     "token {:#?}, owner {}",
            //     pub_key_token_address,
            //     owner.unwrap()
            // );
        }

        for balance in pre_token_balances.unwrap() {
            let owner: Option<String> = balance.owner.clone().into();
            let index_usize = balance.account_index.to_usize().unwrap();
            let pub_key_token_address = account_keys[index_usize].clone();

            let dolar = pub_key_token_address["pubkey"].as_str().unwrap();

            // let testing: Option<&str> = dolar.to_string();

            token_account_owners.insert(dolar.to_string(), owner.unwrap());
            // println!(
            //     "token {:#?}, owner {}",
            //     pub_key_token_address,
            //     owner.unwrap()
            // );
        }

        // let index_usize = balance.account_index.to_usize().unwrap();

        // let pub_key_token_address = account_keys[index_usize].clone();

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
            fees: fees,
            fees_total: fee,
            token_changes_owner,
            // token_changes_new: changes_by_owner_new,
            token_changes_token_account: token_changes_token_accounts,
            tokens,
            token_prices: None,
            actions: vec![],
            all_actions: vec![],
            changes_by_address: HashMap::new(),
            value_changes: vec![],
            token_account_owners,
            // token_amounts: token_amounts,
        }
    }

    pub fn set_token_prices(&mut self, token_prices: HashMap<String, String>) {
        self.token_prices = Some(token_prices);
    }

    pub fn format(&self, expand: Option<Vec<String>>) -> TransactionParsedResponse {
        let expands = expand.clone().unwrap_or_default();

        let token_changes_owner = if expands.contains(&"token_changes_owner".to_string()) {
            Some(self.token_changes_owner.format())
        } else {
            None
        };

        let token_changes_token_account =
            if expands.contains(&"token_changes_token_account".to_string()) {
                Some(self.token_changes_token_account.format())
            } else {
                None
            };

        let addresses = if expands.contains(&"addresses".to_string()) {
            Some(self.addresses.clone())
        } else {
            None
        };

        // let dolar = combine_token_transfers(self.token_changes_owner.values.clone());

        TransactionParsedResponse {
            // dolar,
            signer: self.signer.clone(),
            ubo: self.ubo.clone(),
            block_timestamp: self.block_timestamp,
            block_datetime: self.block_datetime.clone(),
            hash: self.hash.clone(),
            addresses: addresses,
            block_number: self.block_number,
            chain_id: self.chain_id,
            from: self.from.clone(),
            to: self.to.clone(),
            state: self.state.clone(),
            description: self.description.clone(),
            spam_transaction: self.spam_transaction,
            contract_address: self.contract_address.clone(),
            fees: self.fees.clone().iter().map(|fees| fees.format()).collect(),
            fees_total: self.fees_total,

            token_changes_token_account: token_changes_token_account,
            tokens: self.tokens.clone(),

            // actions: self.actions.clone(),
            // actions: self.actions.iter().map(|value| value.format()).collect(),
            actions: self
                .actions
                .clone()
                .iter()
                .map(|value| value.format())
                .collect(),
            changes_by_address: self.changes_by_address.clone(),
            value_changes: self
                .value_changes
                .clone()
                .iter()
                .map(|value| value.format())
                .collect(),
            token_changes_owner: token_changes_owner,
            all_actions: self
                .all_actions
                .clone()
                .iter()
                .map(|value| value.format())
                .collect(),
            // token_changes_owner: Some(self.token_changes_owner.format()), // token_prices: self.token_prices.clone(),
        }
    }

    pub fn set_prices_to_token_changes(&mut self, token_prices: HashMap<String, String>) {
        self.token_prices = Some(token_prices.clone());
        self.token_changes_owner.set_prices(token_prices.clone());
        self.token_changes_token_account
            .set_prices(token_prices.clone());

        // self.token_changes_token_account.set_prices(token_prices);
    }

    pub fn create_actions(
        &mut self,
    ) -> (
        HashMap<std::string::String, HashMap<std::string::String, BalanceChangedFormatted>>,
        Vec<BalanceChange>,
        Vec<ValueChange>,
    ) {
        let (balance_changes_combined, removed, combined) = combine_token_transfers(
            self.token_changes_owner.values.clone(),
            self.token_account_owners.clone(),
            self.hash.clone(),
        );

        let dolar = balance_changes_combined
            .iter()
            .map(|(key, value)| {
                let testing = value
                    .iter()
                    .map(|(key, value)| {
                        let balance_change = value;
                        let token_address = key.to_string();

                        let token_values_formatted = balance_change.format();

                        return (token_address, token_values_formatted);
                    })
                    .collect::<HashMap<String, BalanceChangedFormatted>>();

                return (key.to_string(), testing);
            })
            .collect::<HashMap<String, HashMap<String, BalanceChangedFormatted>>>();

        let (swaps, other_testing, _changes_by_address) = parse_events_to_swap(combined.clone());

        let transfers = parse_value_changes_to_transfers(other_testing, &self.signer);

        self.all_actions = [&swaps[..], &transfers[..]].concat();

        let testing = self
            .all_actions
            .iter()
            .filter(|x| x.addresses.contains(&self.ubo.to_string()));

        self.actions = testing.cloned().collect();
        return (dolar, removed, combined);
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
