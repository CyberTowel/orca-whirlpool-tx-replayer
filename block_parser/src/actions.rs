use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use num::complex::ComplexFloat;
use num_bigfloat::BigFloat;
use serde::{Deserialize, Serialize};
use solana_sdk::{blake3::Hash, signer};

use crate::{
    interfaces::{
        BalanceChange, BalanceChangedFormatted, TokenChangesMap, ValueChange, ValueChangeFormatted,
    },
    token_parser::get_rounded_amount,
};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CtAction {
    pub action_type: String,
    pub protocol_name: Option<String>,
    pub protocol_id: Option<String>,
    pub protocol: Option<String>,
    pub addresses: Vec<String>,
    pub event_ids: Vec<String>,
    pub ubo: Option<String>,
    // pub fields: ActionFields,
    pub fields: ActionFields,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ActionFields {
    CtSwap(SwapFields),
    CtTransfer(TransferFields),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "action_type", content = "fields", rename_all = "lowercase")]
pub enum ActionFieldsFormatted {
    CtSwap(SwapFieldsFormatted),
    CtTransfer(TransferFieldsFormatted),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CtActionFormatted {
    pub protocol_name: Option<String>,
    pub protocol_id: Option<String>,
    pub protocol: Option<String>,
    pub addresses: Vec<String>,
    pub event_ids: Vec<String>,
    pub u_bwallet_address: Option<String>,
    #[serde(flatten)]
    pub action: ActionFieldsFormatted,
    // pub fields: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SwapFieldsFormatted {
    // tokens_fee: Vec<TokenChanges>,
    pub tokens_from: Vec<ValueChangeFormatted>,
    pub tokens_to: Vec<ValueChangeFormatted>,
    pub swap_hops: Vec<String>,
    pub router_events: Vec<String>,
    pub testing: bool,
    pub to: Option<String>,
    pub from: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SwapFields {
    // tokens_fee: Vec<TokenChanges>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tokens_from: Vec<ValueChange>,
    pub tokens_to: Vec<ValueChange>,
    pub swap_hops: Vec<String>,
    pub router_events: Vec<String>,
    pub testing: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TransferFields {
    // tokens_fee: Vec<TokenChanges>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: BigFloat,
    pub amount_usd: Option<BigFloat>,
    pub tokens_transferred: Vec<BalanceChange>,
    pub router_events: Vec<String>,
    pub testing: bool,
    pub mint: String,
    // pub key: String,
    // pub value_transferred: BigFloat,
    // pub value_transferred_usd: Option<BigFloat>,
    // pub mint: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TransferFieldsFormatted {
    // tokens_fee: Vec<TokenChanges>,
    pub tokens_transferred: Vec<BalanceChangedFormatted>,
    pub router_events: Vec<String>,
    pub testing: bool,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: String,
    pub amount_usd: Option<String>,
    pub mint: String,
    // pub value_transferred: String,
    // pub value_transferred_usd: Option<String>,
    // pub mint: String,
}

impl ActionFields {
    fn format(&self) -> ActionFieldsFormatted {
        let values = match self {
            ActionFields::CtSwap(fields) => ActionFieldsFormatted::CtSwap(fields.format()),
            ActionFields::CtTransfer(fields) => ActionFieldsFormatted::CtTransfer(fields.format()),
        };

        values
    }
}

impl ValueChange {
    pub fn format(&self) -> ValueChangeFormatted {
        let amount = get_rounded_amount(self.amount, 18);
        let amount_diff = match self.amount_diff {
            Some(x) => Some(get_rounded_amount(x, 18)),
            None => None,
        };

        ValueChangeFormatted {
            from: self.from.clone(),
            to: self.to.clone(),
            mint: self.mint.clone(),
            amount: amount,
            amount_usd: match self.amount_usd {
                Some(x) => Some(get_rounded_amount(x, 18)),
                None => None,
            },
            balance_changes: self
                .balance_changes
                .clone()
                .iter()
                .map(|bc| bc.format())
                .collect(),
            amount_diff: amount_diff,
        }
    }
}

impl SwapFields {
    fn format(&self) -> SwapFieldsFormatted {
        let tokens_from: Vec<ValueChangeFormatted> = self
            .tokens_from
            .iter()
            .map(|balance_change| balance_change.format())
            .collect();

        let tokens_to: Vec<ValueChangeFormatted> = self
            .tokens_to
            .iter()
            .map(|balance_change| balance_change.format())
            .collect();

        SwapFieldsFormatted {
            tokens_from,
            tokens_to,
            swap_hops: self.swap_hops.clone(),
            router_events: self.router_events.clone(),
            testing: self.testing,
            to: self.to.clone(),
            from: self.from.clone(),
        }
    }
}

impl TransferFields {
    fn format(&self) -> TransferFieldsFormatted {
        let tokens_transferred: Vec<BalanceChangedFormatted> = self
            .tokens_transferred
            .iter()
            .map(|balance_change| balance_change.format())
            .collect();

        TransferFieldsFormatted {
            tokens_transferred,
            router_events: self.router_events.clone(),
            testing: self.testing,
            from: self.from.clone(),
            to: self.to.clone(),
            amount: get_rounded_amount(self.amount, 18),
            amount_usd: match self.amount_usd {
                Some(x) => Some(get_rounded_amount(x, 18)),
                None => None,
            },
            mint: self.mint.clone(),
            // value_transferred: get_rounded_amount(self.value_transferred, 18),
            // value_transferred_usd: match self.value_transferred_usd {
            //     Some(x) => Some(get_rounded_amount(x, 18)),
            //     None => None,
            // },
            // mint: self.mint.clone(),
        }
    }
}

impl CtAction {
    pub fn format(&self) -> CtActionFormatted {
        CtActionFormatted {
            protocol_name: self.protocol_name.clone(),
            protocol_id: self.protocol_id.clone(),
            protocol: self.protocol.clone(),
            addresses: self.addresses.clone(),
            event_ids: self.event_ids.clone(),
            u_bwallet_address: self.ubo.clone(),
            action: self.fields.format(),
        }
    }
}

pub fn parse_token_changes_to_transfers(
    address_token_changes: Vec<ValueChange>,
    signer: &String,
    // transaction_from: Option<String>,
) -> Vec<CtAction> {
    let mut actions: Vec<CtAction> = Vec::new();

    // let mut transfer_keys = HashMap::new();

    for value in address_token_changes {
        let mut tokens_transfered = vec![];
        // let mut token_transfered_keys = HashMap::new();

        for transfer in value.balance_changes.clone() {
            //     let from_key =
            //         transfer.value_transferred.abs().to_string() + "##" + &transfer.mint.to_string();

            //     if token_transfered_keys.contains_key(&from_key) {
            //         if transfer.owner == signer.to_string() {
            //             println!("take the duplicate because it's from signer")
            //         } else {
            //             continue;
            //         }
            //     }

            //     token_transfered_keys.insert(from_key, transfer.owner.clone());

            tokens_transfered.push(transfer);
        }

        // let from_key = value.from.clone().unwrap_or("_".to_string())
        //     + "##"
        //     + &value.to.clone().unwrap_or("_".to_string().to_string())
        //     + "##"
        //     + &value.mint
        //     + "##"
        //     + &value.amount.to_string();

        let fields = ActionFields::CtTransfer(TransferFields {
            tokens_transferred: tokens_transfered.clone(),
            amount: value.amount,
            amount_usd: value.amount_usd,
            from: value.from,
            to: value.to,
            router_events: Vec::new(),
            testing: true,
            mint: value.mint.clone(),
            // key: from_key.clone(),
            // value_transferred: value.amount,
            // value_transferred_usd: value.amount_usd,
            // mint: value.mint.clone(),
        });

        // if transfer_keys.contains_key(&from_key) {
        //     continue;
        // }

        // transfer_keys.insert(from_key, true);

        actions.push(CtAction {
            action_type: "cttransfer".to_string(),
            protocol_name: None,
            protocol_id: None,
            protocol: None,
            addresses: vec![value.mint.to_lowercase()],
            event_ids: Vec::new(),
            ubo: None,
            fields: fields,
        });
        // let with_values: Vec<BalanceChange> = value
        //     .iter()
        //     .filter(|(_token_address, balance_change)| {
        //         if !balance_change.difference.is_zero() {
        //             return true;
        //         }
        //         return false;
        //     })
        //     .map(|(_token_address, balance_change)| {
        //         let balance_change_r = balance_change.clone();

        //         balance_change_r
        //         // balance_change.clone()
        //     })
        //     .collect();

        // if with_values.is_empty() {
        //     continue;
        // }

        // let fields = ActionFields::CtTransfer(TransferFields {
        //     tokens_transferred: with_values,
        //     router_events: Vec::new(),
        //     testing: true,
        // });

        // actions.push(CtAction {
        //     action_type: "cttransfer".to_string(),
        //     protocol_name: None,
        //     protocol_id: None,
        //     protocol: None,
        //     addresses: vec![key.to_lowercase()],
        //     event_ids: Vec::new(),
        //     ubo: None,
        //     fields: fields,
        // });
    }

    return actions;
}

pub fn parse_events_to_swap(
    items: Vec<ValueChange>,
) -> (
    Vec<CtAction>,
    Vec<ValueChange>,
    HashMap<String, HashMap<String, Vec<ValueChange>>>,
) {
    let mut actions: Vec<CtAction> = vec![];
    let mut other: Vec<ValueChange> = vec![];

    let mut tokens_mapped_address = HashMap::new();

    for item in items {
        let item_from = item.clone();
        let item_to = item.clone();

        // let from_key = item.from.clone().unwrap_or("_".to_string())
        //     + "##"
        //     + &item.to.clone().unwrap_or("_".to_string().to_string())
        //     + "##"
        //     + &item.mint
        //     + "##"
        //     + &item.amount.to_string();

        // let to_key = item.to.clone().unwrap_or("_".to_string())
        //     + "##"
        //     + &item.from.clone().unwrap_or("_".to_string().to_string())
        //     + "##"
        //     + &item.mint
        //     + "##"
        //     + &item.amount.to_string();

        // println!("{:#?}", from_key);
        // if (tokens_keys_used.contains_key(&from_key) || tokens_keys_used.contains_key(&to_key)) {
        //     println!("key already used");
        //     continue;
        // }

        // println!("from_key: {:#?}", from_key);
        // println!("to_key  : {:#?}", to_key);

        // tokens_keys_used.insert(from_key.clone(), true);
        // tokens_keys_used.insert(to_key.clone(), true);

        // println!("tokens_keys_used {:#?}", tokens_keys_used);

        if item_from.from.is_some() {
            tokens_mapped_address
                .entry(item_from.clone().from.unwrap().to_string())
                .or_insert_with(HashMap::new)
                .entry(item_from.mint.clone())
                .or_insert_with(Vec::new)
                .push(item_from);
        }

        // if tokens_keys_used.contains_key(&from_key) || tokens_keys_used.contains_key(&to_key) {
        //     println!("key already used");
        //     continue;
        // }

        // address -> received (when to?)
        // address -> sent (when from?)
        // then check

        if (item_to.to.is_some()) {
            tokens_mapped_address
                .entry(item_to.clone().to.unwrap().to_string())
                .or_insert_with(HashMap::new)
                .entry(item_to.mint.clone())
                .or_insert_with(Vec::new)
                .push(item_to);
        }
    }

    // return (actions, other, tokens_mapped_address);

    let mut tokens_keys_used = HashMap::new();

    for (address, value) in tokens_mapped_address.clone() {
        if value.len() > 1 {
            let mut elements = value
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<ValueChange>>();

            if (elements[0].amount.is_positive() && elements[1].amount.is_negative()) {
                elements.reverse();
                // let temp = elements[0].clone();
                // elements[0] = elements[1].clone();
                // elements[1] = temp;
            }

            let fields = ActionFields::CtSwap(SwapFields {
                tokens_from: vec![elements[0].clone()],
                tokens_to: vec![elements[1].clone()],
                router_events: Vec::new(),
                swap_hops: Vec::new(),
                testing: true,
                from: elements[0].to.clone(),
                to: elements[1].to.clone(),
            });

            let from_key = elements[0].from.clone().unwrap_or("_".to_string())
                + "##"
                + &elements[0]
                    .to
                    .clone()
                    .unwrap_or("_".to_string().to_string())
                + "##"
                + &elements[0].mint
                + "##"
                + &elements[0].amount.to_string();

            let to_key = elements[1].from.clone().unwrap_or("_".to_string())
                + "##"
                + &elements[1]
                    .to
                    .clone()
                    .unwrap_or("_".to_string().to_string())
                + "##"
                + &elements[1].mint
                + "##"
                + &elements[1].amount.to_string();

            tokens_keys_used.insert(from_key, true);
            tokens_keys_used.insert(to_key, true);

            actions.push(CtAction {
                action_type: "ctswap".to_string(),
                protocol_name: None,
                protocol_id: None,
                protocol: None,
                addresses: vec!["todo".to_string()],
                event_ids: Vec::new(),
                ubo: None,
                fields: fields,
            });
            // tokens_mapped_address.remove(&address);
        } else {
            let tesitng = value
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<ValueChange>>();

            // other.(tesitng);
            other.extend(tesitng);
        }
    }

    let testing32 = other
        .into_iter()
        .filter({
            |item| {
                let to_key = item.from.clone().unwrap_or("_".to_string())
                    + "##"
                    + &item.to.clone().unwrap_or("_".to_string().to_string())
                    + "##"
                    + &item.mint
                    + "##"
                    + &item.amount.to_string();

                return !tokens_keys_used.contains_key(&to_key);
            }
        })
        .collect();

    return (actions, testing32, tokens_mapped_address);

    // for item in items {
    //     if item.difference.is_positive() {
    //         // actions.push(item);
    //     } else {
    //         // other.push(item);
    //     }
    // }
}

pub fn to_archive_parse_token_changes_to_swaps(
    address_token_changes: TokenChangesMap,
    // transaction_from: Option<String>,
) -> (Vec<CtAction>, TokenChangesMap) {
    let mut used_ref = address_token_changes.clone();
    let mut actions: Vec<CtAction> = Vec::new();

    for (key, value) in address_token_changes {
        let tokens_from: Vec<BalanceChange> = value
            .iter()
            .filter(|(_token_address, balance_change)| {
                balance_change.difference.is_positive() && !balance_change.difference.is_zero()
            })
            .map(|(_token_address, balance_change)| balance_change.clone())
            .collect();

        let tokens_to: Vec<BalanceChange> = value
            .iter()
            .filter(|(_, balance_change)| balance_change.difference.is_negative())
            .map(|(_token_address, balance_change)| balance_change.clone())
            .collect();

        if tokens_from.is_empty() || tokens_to.is_empty() {
            // used_ref.get_mut(&key).unwrap().set
            continue;
        }

        used_ref.retain(|key, inner_map| {
            inner_map.retain(|inner_key, _value| {
                if tokens_from
                    .iter()
                    .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string())
                    || tokens_to
                        .iter()
                        .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string())
                {
                    false
                } else {
                    true
                }
            });

            return true;
        });

        // let fields = ActionFields::CtSwap(SwapFields {
        //     tokens_from,
        //     tokens_to,
        //     swap_hops: Vec::new(),
        //     router_events: Vec::new(),
        //     testing: true,
        // });

        // actions.push(CtAction {
        //     action_type: "ctswap".to_string(),
        //     protocol_name: None,
        //     protocol_id: None,
        //     protocol: None,
        //     addresses: vec![key.to_lowercase()],
        //     event_ids: Vec::new(),
        //     u_bwallet_address: None,
        //     fields: fields,
        // });
    }

    return (actions, used_ref);
}

fn is_within_range(_value: String, _target: String) -> bool {
    let value = BigFloat::from_str(&_value).unwrap();
    let target = BigFloat::from_str(&_target).unwrap();

    let lower_bound = target.mul(&BigFloat::from(0.9));
    let upper_bound = target.mul(&BigFloat::from(1.1));
    value >= lower_bound && value <= upper_bound
}

pub fn combine_sol_tokens(items: Vec<ValueChange>) -> (Vec<ValueChange>, Vec<ValueChange>) {
    let mut sol_tokens = vec![];
    let mut other = vec![];
    items.iter().for_each(|item| {
        if (item.mint == "sol" || item.mint == "So11111111111111111111111111111111111111112") {
            sol_tokens.push(item.clone());
        } else {
            other.push(item.clone());
        }
    });

    // let mut testing_sol: Vec<&ValueChange> = vec![];

    // Create a HashSet to store unique amounts
    // let mut unique_amounts = HashSet::new();

    // // Filter the vector to keep only unique amounts
    // sol_tokens.retain(|s| {
    //     println!("{:#?}", s.amount.to_string());
    //     unique_amounts.insert(s.amount.to_string())
    // });

    // // Print the filtered vector
    // println!("length after {:?}", sol_tokens.len());

    // Create a HashSet to store unique amounts within the 10% range
    let mut unique_amounts: HashSet<String> = HashSet::new();

    // Filter the vector to keep only unique amounts within the 10% range
    // let filtered_vec: Vec<_> = sol_tokens
    //     .into_iter()
    //     .filter(|s| {
    //         unique_amounts.get(&s.amount.to_string()).is_none()
    //             || unique_amounts
    //                 .iter()
    //                 .any(|a| is_within_range(s.amount.to_string(), a.to_string()))
    //     })
    //     .inspect(|s| {
    //         unique_amounts.insert(s.amount.to_string());
    //     })
    //     .collect();

    let filtered_vec: Vec<_> = sol_tokens
        .clone()
        .into_iter()
        .filter(|s| {
            let within_range = unique_amounts
                .iter()
                .any(|a| is_within_range(s.amount.to_string(), a.to_string()));
            if !within_range {
                unique_amounts.insert(s.amount.to_string());
            }
            !within_range
        })
        .collect();

    // let doarlasdf: Vec<&ValueChange> = sol_tokens
    //     .iter()
    //     .filter(|item| {
    //         println!("{:#?}", item.format().amount);

    //         let exisitng_item = testing_sol.iter().find(|next_item: &&&mut ValueChange| {
    //             if next_item.amount == next_item.amount {
    //                 println!("found");
    //                 return false;
    //             } else {
    //                 return true;
    //             }
    //         });

    //         if (!exisitng_item.is_some()) {
    //             println!("not found ");
    //             testing_sol.push(item.clone());
    //         }

    //         return exisitng_item.is_some();

    //         // if exisitng_item.is_some() {
    //         //     println!("found");
    //         //     return
    //         // }

    //         // testing_sol.push(item.clone());
    //     })
    //     .collect();

    other.extend(filtered_vec);

    return (other.clone(), sol_tokens);
}

pub fn combine_token_transfers(
    address_token_changes: HashMap<String, HashMap<String, BalanceChange>>,
) -> Vec<ValueChange> {
    // println!("{:#?}", changes);

    // let mut testing = HashMap::new();

    let mut used_ref = address_token_changes.clone();
    let mut actions: Vec<CtAction> = Vec::new();

    // for (key, value) in address_token_changes {
    // let tokens_from: Vec<BalanceChange> = value
    //     .iter()
    //     .filter(|(_token_address, balance_change)| {
    //         balance_change.difference.is_positive() && !balance_change.difference.is_zero()
    //     })
    //     .map(|(_token_address, balance_change)| balance_change.clone())
    //     .collect();

    // let tokens_to: Vec<BalanceChange> = value
    //     .iter()
    //     .filter(|(_, balance_change)| balance_change.difference.is_negative())
    //     .map(|(_token_address, balance_change)| balance_change.clone())
    //     .collect();

    // if tokens_from.is_empty() || tokens_to.is_empty() {
    //     // used_ref.get_mut(&key).unwrap().set
    //     continue;
    // }

    used_ref.retain(|key, inner_map| {
        let mut keep = false;
        inner_map.retain(|inner_key, _value| {
            if _value.difference.is_zero()
            // if tokens_from
            //     .iter()
            //     .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string())
            //     || tokens_to
            //         .iter()
            //         .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string())
            {
                keep = keep || false;
                return false;
            } else {
                keep = true;
                true
            }
        });

        return keep;
    });

    let mut changes_by_token: HashMap<String, Vec<ValueChange>> = HashMap::new();

    for (test, value) in used_ref.clone() {
        for (key, value) in value {
            let exists_entry = changes_by_token.get_mut(&key.to_string());

            if (exists_entry.is_some()) {
                let value_to_found = value.value_transferred.abs();

                let value_to_find_upper = value_to_found.clone().mul(&BigFloat::from(1.1));
                let value_to_find_lower = value_to_found.clone().mul(&BigFloat::from(0.9));

                // let dolar = exists_entry.unwrap();

                let existing_by_token = exists_entry.unwrap().into_iter().find(|vc| {
                    let from_to_cond = if (value.value_transferred.is_positive()) {
                        vc.to.is_none()
                    } else {
                        vc.from.is_none()
                    };

                    if vc.amount.abs() < value_to_find_upper
                        && vc.amount.abs() > value_to_find_lower
                        && from_to_cond
                    {
                        // vc.balance_changes.push(value.format());
                        true
                    } else {
                        false
                    }
                });

                if (existing_by_token.is_some()) {
                    let existing_by_token = existing_by_token.unwrap();

                    let existing_value = existing_by_token.amount;
                    let new_value = value.value_transferred;
                    let difference: BigFloat = if existing_value.abs() > new_value.abs() {
                        existing_value.abs() - new_value.abs()
                    } else {
                        new_value.abs() - existing_value.abs()
                    };

                    if value.value_transferred.is_positive() {
                        if existing_by_token.to.is_some() {
                            println!("to is already set, implement logic to handle this");
                        }
                        existing_by_token.to = Some(value.owner.to_string());
                        existing_by_token.amount_diff = Some(difference);
                    } else {
                        if existing_by_token.from.is_some() {
                            println!("from is already set, implement logic to handle this");
                        }
                        existing_by_token.from = Some(value.owner.to_string());
                        existing_by_token.amount_diff = Some(difference);
                    }
                    existing_by_token.balance_changes.push(value);
                    continue;
                }
            }

            let mut from = None;
            let mut to = None;

            if (value.value_transferred.is_positive()) {
                to = Some(value.owner.to_string());
            } else {
                from = Some(value.owner.to_string());
            }

            let vc = ValueChange {
                from: from,
                to: to,
                mint: key.to_string(),
                amount: value.value_transferred.abs(),
                amount_usd: match value.value_transferred_usd {
                    Some(x) => Some(x.abs()),
                    None => None,
                },
                amount_diff_usd: None,
                balance_changes: vec![value],
                amount_diff: None,
            };

            // If the token doesn't exist, insert a new key-value pair
            changes_by_token
                .entry(key.to_string())
                .or_insert_with(Vec::new)
                .push(vc);
        }
    }

    let combined = changes_by_token.values().flatten().cloned().collect();

    return combined;
    // let mut changes_by_token = HashMap::new();

    // let mut result = HashMap::new();

    // for (key, value) in changes {
    //     let entry = result.entry(key.clone()).or_insert_with(HashMap::new);
    //     entry
    //         .entry(value.clone())
    //         .or_insert_with(Vec::new)
    //         .push(value);
    // }

    // for (account, change) in changes {
    //     // println!("{:#?}", change);

    //     // for (token, balance_change) in change {
    //     //     println!("{:#?}", balance_change);
    //     //     let mut owner_entry = changes_by_token.entry(balance_change.mint);
    //     //     // *owner_entry = balance_change
    //     //     *owner_entry = balance_change.clone();
    //     // }
    // }

    // let result_test = serde_json::to_string(&result).unwrap();

    // println!("{:#?}", result_test);
}
