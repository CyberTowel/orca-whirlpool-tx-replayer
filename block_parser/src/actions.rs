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
    pub fields: ActionFields,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ActionFields {
    SwapFields(SwapFields),
}

pub enum ActionFieldsFormatted {
    SwapFields(SwapFieldsFormatted),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CtActionFormatted {
    pub action_type: String,
    pub protocol_name: Option<String>,
    pub protocol_id: Option<String>,
    pub protocol: Option<String>,
    pub addresses: Vec<String>,
    pub event_ids: Vec<String>,
    pub u_bwallet_address: Option<String>,
    pub fields: SwapFieldsFormatted,
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

pub fn parse_token_changes_to_swaps(
    address_token_changes: TokenChangesMap,
    // transaction_from: Option<String>,
) -> (Vec<CtAction>) {
    // let _other: Vec<AddressTokenChange> = Vec::new();
    let mut actions: Vec<CtAction> = Vec::new();

    for (key, value) in address_token_changes {
        let tokens_from: Vec<BalanceChange> = value
            .iter()
            .filter(|(_token_address, balance_change)| {
                if balance_change.difference.is_positive() && !balance_change.difference.is_zero() {
                    return true;
                }
                return false;
            })
            .map(|(_token_address, balance_change)| balance_change.clone())
            .collect();

        let tokens_to: Vec<BalanceChange> = value
            .iter()
            .filter(|(token_address, balance_change)| {
                if balance_change.difference.is_negative() {
                    return true;
                }
                return false;
            })
            .map(|(token_address, balance_change)| balance_change.clone())
            .collect();

        if (tokens_from.is_empty() || tokens_to.is_empty()) {
            continue;
        }

        let fields = ActionFields::SwapFields(SwapFields {
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

        // println!(
        //     "token_from amount {:#?}, tokens to amount: {}",
        //     tokens_from.len(),
        //     tokens_to.len()
        // );
    }

    return (actions);

    // if  address_token_changes {
    // for _item in changes {
    //     let tokens_from: Vec<TokenChange> = item
    //         .tokens_changed
    //         .iter()
    //         .filter(|t| t.difference < BigDecimal::from(0))
    //         .map(|t| TokenChange {
    //             token: t.token.clone(),
    //             difference: t.difference.clone(),
    //             differnceUi: t.differnceUi.clone(),
    //             priceUSDUi: t.priceUSDUi.clone(),
    //         })
    //         .collect();

    //     let tokens_to: Vec<TokenChange> = item
    //         .tokens_changed
    //         .iter()
    //         .filter(|t| t.difference > BigDecimal::from(0))
    //         .map(|t| TokenChange {
    //             token: t.token.clone(),
    //             difference: t.difference.clone(),
    //             differnceUi: t.differnceUi.clone(),
    //             priceUSDUi: t.priceUSDUi.clone(),
    //         })
    //         .collect();

    //     if tokens_from.is_empty() || tokens_to.is_empty() {
    //         other.push(item);
    //         continue;
    //     }

    //     let swap_action = CtSwap {
    //         action_type: "ctswap".to_string(),
    //         protocol_name: None,
    //         protocol_id: None,
    //         protocol: None,
    //         addresses: vec![item
    //             .owner
    //             .as_ref()
    //             .map(|o| o.to_lowercase())
    //             .unwrap_or_default()],
    //         event_ids: Vec::new(),
    //         u_bwallet_address: transaction_from.clone(),
    //         fields: SwapFields {
    //             tokens_fee: Vec::new(),
    //             tokens_from,
    //             tokens_to,
    //             swap_hops: Vec::new(),
    //             router_events: Vec::new(),
    //             testing: false,
    //         },
    //     };

    //     actions.push(swap_action);
    // }
    // }

    // (actions, other)
}
