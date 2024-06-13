use std::collections::HashMap;

use num_bigfloat::BigFloat;

use crate::{
    actions::{ActionFields, CtAction, CtActionFormatted, TransferFields},
    interfaces::{ValueChange, ValueChangeFormatted},
};

pub fn parse_value_changes_to_transfers(
    address_token_changes: Vec<ValueChange>,
    signer: &String,
) -> Vec<CtAction> {
    let mut token_transfered_keys = HashMap::new();
    let mut token_transfered_keys_new: HashMap<String, Vec<ValueChange>> = HashMap::new();

    for value_change in address_token_changes.iter() {
        // address_token_change.owner = signer.clone();
        let key_1 = value_change.to.clone().unwrap_or("_".to_string());
        let key_2 = value_change.from.clone().unwrap_or("_".to_string());
        let mut from_to_key = key_1.clone() + "_" + &key_2.clone();

        if (key_1 < key_2) {
            from_to_key = key_2.clone() + "_" + &key_1.clone();
        }

        if (!token_transfered_keys.contains_key(&from_to_key)) {

            // let mut value_change = value_change.clone();
            // value_change.from = Some(key_2.clone());
            // value_change.to = Some(key_1.clone());
            // token_transfered_keys.insert(from_to_key.clone(), value_change.clone());
            // continue;
        }

        if (token_transfered_keys_new.contains_key(&from_to_key)) {
            let value_changes = token_transfered_keys_new.get_mut(&from_to_key).unwrap();
            value_changes.push(value_change.clone());
            continue;
        }

        token_transfered_keys_new.insert(from_to_key.clone(), vec![value_change.clone()]);
        token_transfered_keys.insert(from_to_key, value_change.clone());
    }

    // let combined_json = serde_json::to_string(&token_transfered_keys_new).unwrap();

    let testing: Vec<CtAction> = token_transfered_keys
        .values()
        .into_iter()
        .map(|item| {
            let fields = ActionFields::CtTransfer(TransferFields {
                tokens_transferred: item.balance_changes.clone(),
                tokens_transferred_new: vec![],
                // amount: item.amount,
                amount_usd: item.amount_usd,
                from: item.from.clone(),
                to: item.to.clone(),
                router_events: Vec::new(),
                testing: true,
                // mint: item.mint.clone(),
                // key: from_key.clone(),
                // value_transferred: value.amount,
                // value_transferred_usd: value.amount_usd,
                // mint: value.mint.clone(),
            });

            let mut address = Vec::new();
            if (item.from.is_some()) {
                address.push(item.from.clone().unwrap());
            }
            if (item.to.is_some()) {
                address.push(item.to.clone().unwrap());
            }

            // item.format()
            CtAction {
                action_type: "cttransfer".to_string(),
                protocol_name: None,
                protocol_id: None,
                protocol: None,
                addresses: address,
                event_ids: Vec::new(),
                ubo: None,
                fields: fields,
            }
        })
        .collect();

    // let values: Vec<CtActionFormatted> = testing.into_iter().map(|item| item.format()).collect();

    // println!("amounts: {:?} values: {:#?}", values.len(), values);

    let testing2: Vec<CtAction> = token_transfered_keys_new
        .values()
        .into_iter()
        .map(|item| {
            let usd_total = item.iter().fold(BigFloat::from(0), |acc, x| {
                acc.add(&x.amount_usd.clone().unwrap_or(BigFloat::from(0)))
            });

            let fields = ActionFields::CtTransfer(TransferFields {
                tokens_transferred: vec![],
                tokens_transferred_new: item.clone(),
                // amount: BigFloat::from(0),
                // amount_usd: Some(BigFloat::from(0)),
                from: item[0].from.clone(),
                to: item[0].to.clone(),
                router_events: Vec::new(),
                testing: true,
                amount_usd: Some(usd_total),
                // mint: "mint".to_string(),
                // mint: item.mint.clone(),
                // key: from_key.clone(),
                // value_transferred: value.amount,
                // value_transferred_usd: value.amount_usd,
                // mint: value.mint.clone(),
            });

            let mut address = Vec::new();
            // if (item.from.is_some()) {
            //     address.push(item.from.clone().unwrap());
            // }
            // if (item.to.is_some()) {
            //     address.push(item.to.clone().unwrap());
            // }

            // item.format()
            CtAction {
                action_type: "cttransfer".to_string(),
                protocol_name: None,
                protocol_id: None,
                protocol: None,
                addresses: address,
                event_ids: Vec::new(),
                ubo: None,
                fields: fields,
            }
        })
        .collect();

    return testing2;
}
