use std::collections::HashMap;

use chrono::format;
use num_bigfloat::BigFloat;
use serde::{Deserialize, Serialize};

use crate::interfaces::{BalanceChange, BalanceChangedFormatted, TokenChanges, TokenChangesMap};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CtAction {
    pub action_type: String,
    pub protocol_name: Option<String>,
    pub protocol_id: Option<String>,
    pub protocol: Option<String>,
    pub addresses: Vec<String>,
    pub event_ids: Vec<String>,
    pub u_bwallet_address: Option<String>,
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
    pub tokens_from: Vec<BalanceChangedFormatted>,
    pub tokens_to: Vec<BalanceChangedFormatted>,
    pub swap_hops: Vec<String>,
    pub router_events: Vec<String>,
    pub testing: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SwapFields {
    // tokens_fee: Vec<TokenChanges>,
    pub tokens_from: Vec<BalanceChange>,
    pub tokens_to: Vec<BalanceChange>,
    pub swap_hops: Vec<String>,
    pub router_events: Vec<String>,
    pub testing: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TransferFields {
    // tokens_fee: Vec<TokenChanges>,
    pub tokens_transferred: Vec<BalanceChange>,
    pub router_events: Vec<String>,
    pub testing: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TransferFieldsFormatted {
    // tokens_fee: Vec<TokenChanges>,
    pub tokens_transferred: Vec<BalanceChangedFormatted>,
    pub router_events: Vec<String>,
    pub testing: bool,
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

impl SwapFields {
    fn format(&self) -> SwapFieldsFormatted {
        let tokens_from: Vec<BalanceChangedFormatted> = self
            .tokens_from
            .iter()
            .map(|balance_change| balance_change.format())
            .collect();

        let tokens_to: Vec<BalanceChangedFormatted> = self
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
            u_bwallet_address: self.u_bwallet_address.clone(),
            action: self.fields.format(),
        }
    }
}

pub fn parse_token_changes_to_transfers(
    address_token_changes: TokenChangesMap,
    // transaction_from: Option<String>,
) -> Vec<CtAction> {
    let mut actions: Vec<CtAction> = Vec::new();

    for (key, value) in address_token_changes {
        let with_values: Vec<BalanceChange> = value
            .iter()
            .filter(|(_token_address, balance_change)| {
                if !balance_change.difference.is_zero() {
                    return true;
                }
                return false;
            })
            .map(|(_token_address, balance_change)| {
                let mut balance_change_r = balance_change.clone();

                // if (balance_change_r.fee.is_some()) {
                // println!(
                //     "Balance change before: {:#?}",
                //     balance_change_r.difference.to_string()
                // );

                // println!(
                //     "Balance change fee: {:#?}",
                //     balance_change_r.fee.clone().unwrap().to_string()
                // );

                //balance_change_r.fee.unwrap();

                // println!(
                //     "Balance change after: {:#?}",
                //     balance_change_r.difference.to_string()
                // );
                // }

                // balance_change_r.difference -= BigFloat::from_f64(2500.0);

                balance_change_r
                // balance_change.clone()
            })
            .collect();

        if (with_values.is_empty()) {
            continue;
        }

        let fields = ActionFields::CtTransfer(TransferFields {
            tokens_transferred: with_values,
            router_events: Vec::new(),
            testing: true,
        });

        actions.push(CtAction {
            action_type: "cttransfer".to_string(),
            protocol_name: None,
            protocol_id: None,
            protocol: None,
            addresses: vec![key.to_lowercase()],
            event_ids: Vec::new(),
            u_bwallet_address: None,
            fields: fields,
        });
    }

    return actions;
}

pub fn parse_token_changes_to_swaps(
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

        if (tokens_from.is_empty() || tokens_to.is_empty()) {
            // used_ref.get_mut(&key).unwrap().set
            continue;
        }

        used_ref.retain(|key, inner_map| {
            inner_map.retain(|inner_key, _value| {
                if (tokens_from
                    .iter()
                    .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string())
                    || tokens_to
                        .iter()
                        .any(|t| t.mint == inner_key.to_string() && t.owner == key.to_string()))
                {
                    false
                } else {
                    true
                }
            });

            return true;
        });

        let fields = ActionFields::CtSwap(SwapFields {
            tokens_from,
            tokens_to,
            swap_hops: Vec::new(),
            router_events: Vec::new(),
            testing: true,
        });

        actions.push(CtAction {
            action_type: "ctswap".to_string(),
            protocol_name: None,
            protocol_id: None,
            protocol: None,
            addresses: vec![key.to_lowercase()],
            event_ids: Vec::new(),
            u_bwallet_address: None,
            fields: fields,
        });
    }

    return (actions, used_ref);
}
