use core::hash;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use num::complex::ComplexFloat;
use num_bigfloat::BigFloat;
use serde::{de::value, Deserialize, Serialize};
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

pub fn combine_sol_tokens(
    items: Vec<ValueChange>,
    token_account_owner: HashMap<String, String>,
) -> (Vec<ValueChange>, Vec<ValueChange>) {
    let mut sol_tokens = vec![];
    let mut other = vec![];

    items.iter().for_each(|item| {
        if (item.mint == "sol" || item.mint == "So11111111111111111111111111111111111111112") {
            sol_tokens.push(item.clone());
        } else {
            other.push(item.clone());
        }
    });

    let items_mapped = sol_tokens
        .clone()
        .iter_mut()
        .map(|item| {
            if (item.mint == "sol") {
                let testing = token_account_owner.get(&item.to.clone().unwrap_or("_".to_string()));

                if (testing.is_some()) {
                    item.to = Some(testing.unwrap().to_string());
                }
            }
            item.clone()
        })
        .collect::<Vec<ValueChange>>();

    (items_mapped, other)
}

pub fn combine_token_transfers(
    address_token_changes: HashMap<String, HashMap<String, BalanceChange>>,
    token_account_owners: HashMap<String, String>,
    hash: String,
) -> (
    HashMap<std::string::String, HashMap<std::string::String, BalanceChange>>,
    Vec<BalanceChange>,
    Vec<ValueChange>,
) {
    let used_ref = remove_zero_balance(address_token_changes.clone());

    let (balance_changes, removed_sol_tokens) =
        cobine_sol_transfers(used_ref, token_account_owners, hash);

    let combined = combine_to_value_changes(balance_changes.clone());

    return (balance_changes, removed_sol_tokens, combined);

    // combine sol transfers
}

fn remove_zero_balance(
    mut used_ref: HashMap<String, HashMap<String, BalanceChange>>,
) -> HashMap<String, HashMap<String, BalanceChange>> {
    //remove zero balances
    used_ref.retain(|_key, inner_map| {
        let mut keep = false;
        inner_map.retain(|_inner_key, _value| {
            if _value.value_transferred.is_zero() {
                keep = keep || false;
                return false;
            } else {
                keep = true;
                true
            }
        });

        return keep;
    });

    return used_ref;
}

fn cobine_sol_transfers(
    mut used_ref: HashMap<String, HashMap<String, BalanceChange>>,
    token_account_owners: HashMap<String, String>,
    hash: String,
) -> (
    HashMap<String, HashMap<String, BalanceChange>>,
    Vec<BalanceChange>,
) {
    let mut removed_sol_tokens = vec![];

    let mut sol_tokens = vec![];

    used_ref.clone().into_iter().for_each(|(key, value)| {
        let mut other = vec![];

        value.iter().for_each(|item| {
            if item.1.mint == "So11111111111111111111111111111111111111112" {
                sol_tokens.push(item.1.clone());
            } else {
                other.push(item.1.clone());
            }
        });
    });

    used_ref.retain(|_key, inner_map| {
        let mut keep = false;
        inner_map.retain(|_inner_key, value| {
            if value.mint != "sol" {
                keep = true;
                return true;
            }

            // value.mint = "So11111111111111111111111111111111111111112".to_string();

            let value_to_found = value.value_transferred;

            let value_to_find_upper = value_to_found.clone().mul(&BigFloat::from(1.1));
            let value_to_find_lower = value_to_found.clone().mul(&BigFloat::from(0.9));

            let tk_account = value.owner.to_string();

            let dolar = token_account_owners.get(&tk_account);

            let existing_by_token = sol_tokens.iter().find(|vc| {
                let owner_cond = dolar.is_some() && dolar.unwrap().to_string() == vc.owner;

                if vc.value_transferred < value_to_find_upper
                    && vc.value_transferred > value_to_find_lower
                    && owner_cond
                {
                    // vc.balance_changes.push(value.format());
                    true
                } else {
                    false
                }
            });

            if (existing_by_token.is_some()) {
                removed_sol_tokens.push(value.clone());
                return false;
            }

            keep = true;
            true
        });

        return keep;
    });

    let mut testing: HashMap<String, HashMap<String, BalanceChange>> = HashMap::new();

    used_ref.clone().into_iter().for_each(|(owner, mut value)| {
        // let mut other = vec![];

        let mut new: HashMap<String, BalanceChange> = HashMap::new();

        value.iter().for_each(|item| {
            // if item.1.mint != "sol" {
            //     item.1.mint = "So11111111111111111111111111111111111111112".to_string();
            // }

            let mut cloned = item.1.clone();

            if (cloned.mint == "sol") {
                cloned.mint = "So11111111111111111111111111111111111111112".to_string();
            }

            let key = cloned.mint.to_string();

            let exisiting_item = new.get_mut(&key);

            if (exisiting_item.is_some()) {
                let item = exisiting_item.unwrap(); //.clone();

                item.value_transferred = item.value_transferred + &cloned.value_transferred;
                item.difference = item.difference + &cloned.difference;

                if (item.value_transferred_usd.is_some() && cloned.value_transferred_usd.is_some())
                {
                    let value_1 = item.value_transferred_usd.unwrap();
                    let value_2 = cloned.value_transferred_usd.unwrap();
                    item.value_transferred_usd = Some(value_1 + &value_2);
                }

                if (item.difference_usd.is_some() && cloned.difference_usd.is_some()) {
                    let value_1 = item.difference_usd.unwrap();
                    let value_2 = cloned.difference_usd.unwrap();
                    item.difference_usd = Some(value_1 + &value_2);
                }

                item.inner_changes = Some(vec![item.clone(), cloned.clone()]);

                return;
            }

            new.insert(cloned.mint.to_string(), cloned);
        });

        testing.insert(owner, new);
    });

    return (testing, removed_sol_tokens);
}

fn combine_to_value_changes(
    used_ref: HashMap<String, HashMap<String, BalanceChange>>,
) -> Vec<ValueChange> {
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

    let combined: Vec<ValueChange> = changes_by_token.values().flatten().cloned().collect();

    return combined;
}
