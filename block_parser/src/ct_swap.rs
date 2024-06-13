use std::collections::HashMap;

use num_bigfloat::BigFloat;
use serde_json::Value;

use crate::{
    actions::{ActionFields, CtAction, SwapFields},
    interfaces::{ValueChange, ValueChangeFormatted},
};

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

    let mut tokens_mapped_address_new: HashMap<String, HashMap<String, Vec<ValueChange>>> =
        HashMap::new();

    let mut tokens_keys_used = HashMap::new();

    for item in items {
        let item_from = item.clone();
        let item_to = item.clone();

        if item_from.from.is_some() {
            tokens_mapped_address
                .entry(item_from.clone().from.unwrap().to_string())
                .or_insert_with(HashMap::new)
                .entry(item_from.mint.clone())
                .or_insert_with(Vec::new)
                .push(item_from);
        }

        if (item_to.to.is_some()) {
            tokens_mapped_address
                .entry(item_to.clone().to.unwrap().to_string())
                .or_insert_with(HashMap::new)
                .entry(item_to.mint.clone())
                .or_insert_with(Vec::new)
                .push(item_to);
        }
    }

    for (address, value) in tokens_mapped_address.clone() {
        if value.len() > 1 {
            let mut elements = value
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<ValueChange>>();

            let elements_formatted: Vec<ValueChangeFormatted> =
                elements.clone().iter().map(|vc| vc.format()).collect();

            let elements_formatted_json = serde_json::to_string(&elements_formatted).unwrap();

            let address_received: Vec<ValueChange> = elements
                .iter()
                .filter(|item| item.to.clone().unwrap_or("".to_string()) == address.clone())
                // .map(|vc)
                .cloned()
                .collect();

            let address_sent: Vec<ValueChange> = elements
                .iter()
                .filter(|item| item.from.clone().unwrap_or("".to_string()) == address.clone())
                // .map(|vc| vc.format())
                .cloned()
                .collect();

            // if (elements[0].amount.is_positive() && elements[1].amount.is_negative()) {
            //     elements.reverse();
            //     // let temp = elements[0].clone();
            //     // elements[0] = elements[1].clone();
            //     // elements[1] = temp;
            // }

            if (address_received.len() <= 0 || address_sent.len() <= 0) {
                // println!(
                //     "address_received length: {}, address_sent length: {} so not a swap",
                //     address_received.len(),
                //     address_sent.len()
                // );

                // let tesitng = value
                //     .values()
                //     .flatten()
                //     .cloned()
                //     .collect::<Vec<ValueChange>>();

                // other.(tesitng);
                other.extend(address_received);
                other.extend(address_sent);

                continue;
            }

            let key_1 = address_sent[0].to.clone().unwrap_or("_".to_string());
            let key_2 = address_received[0].to.clone().unwrap_or("_".to_string());
            let mut from_to_key = key_1.clone() + "_" + &key_2.clone();

            if (key_1 < key_2) {
                from_to_key = key_2.clone() + "_" + &key_1.clone();
            }

            if (tokens_keys_used.contains_key(&from_to_key)) {
                continue;
            }

            let usd_total_sent = address_sent
                .clone()
                .iter()
                .fold(BigFloat::from(0), |acc, x| {
                    acc.add(&x.amount_usd.clone().unwrap_or(BigFloat::from(0)))
                });

            let usd_total_received = address_received
                .clone()
                .iter()
                .fold(BigFloat::from(0), |acc, x| {
                    acc.add(&x.amount_usd.clone().unwrap_or(BigFloat::from(0)))
                });

            let fields = ActionFields::CtSwap(SwapFields {
                from_to_key: from_to_key.clone(),
                tokens_from: address_sent.clone(),
                tokens_to: address_received.clone(),
                router_events: Vec::new(),
                swap_hops: Vec::new(),
                testing: true,
                from: address_sent[0].to.clone(),
                to: address_received[0].to.clone(),
                tokens_from_total_usd: Some(usd_total_sent),
                tokens_to_total_usd: Some(usd_total_received),
                // elements: elements_formatted.clone(),
                // address_received,
                // address_sent,
            });

            // let from_key = elements[0].from.clone().unwrap_or("_".to_string())
            //     + "##"
            //     + &elements[0]
            //         .to
            //         .clone()
            //         .unwrap_or("_".to_string().to_string())
            //     + "##"
            //     + &elements[0].mint
            //     + "##"
            //     + &elements[0].amount.to_string();

            // let to_key = elements[1].from.clone().unwrap_or("_".to_string())
            //     + "##"
            //     + &elements[1]
            //         .to
            //         .clone()
            //         .unwrap_or("_".to_string().to_string())
            //     + "##"
            //     + &elements[1].mint
            //     + "##"
            //     + &elements[1].amount.to_string();

            // tokens_keys_used.insert(from_key, true);
            // tokens_keys_used.insert(to_key, true);

            tokens_keys_used.insert(from_to_key, true);

            let addresses = match fields.clone() {
                ActionFields::CtSwap(SwapFields {
                    tokens_from,
                    tokens_to,
                    router_events,
                    swap_hops,
                    testing,
                    from,
                    to,
                    from_to_key,
                    tokens_from_total_usd,
                    tokens_to_total_usd,
                    // address_received,
                    // address_sent,
                }) => {
                    let mut testing = Vec::new();
                    if (from.is_some()) {
                        testing.push(from.unwrap());
                    }

                    if (to.is_some()) {
                        testing.push(to.unwrap());
                    }

                    testing
                }

                _ => vec!["todo".to_string()],
            };

            actions.push(CtAction {
                action_type: "ctswap".to_string(),
                protocol_name: None,
                protocol_id: None,
                protocol: None,
                addresses: addresses,
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

    let testing32: Vec<ValueChange> = other
        .into_iter()
        .filter({
            |item| {
                // let to_key = item.from.clone().unwrap_or("_".to_string())
                //     + "##"
                //     + &item.to.clone().unwrap_or("_".to_string().to_string())
                //     + "##"
                //     + &item.mint
                //     + "##"
                //     + &item.amount.to_string();

                let key_1 = item.to.clone().unwrap_or("_".to_string());
                let key_2 = item.from.clone().unwrap_or("_".to_string());
                let mut from_to_key = key_1.clone() + "_" + &key_2.clone();

                if key_1 < key_2 {
                    from_to_key = key_2.clone() + "_" + &key_1.clone();
                }

                if tokens_keys_used.contains_key(&from_to_key) {
                    return false;
                }

                tokens_keys_used.insert(from_to_key.clone(), true);

                return true;
            }
        })
        .collect();

    let lipsum = testing32
        .iter()
        .map(|item| item.format())
        .collect::<Vec<ValueChangeFormatted>>();

    return (actions, testing32, tokens_mapped_address);

    // for item in items {
    //     if item.difference.is_positive() {
    //         // actions.push(item);
    //     } else {
    //         // other.push(item);
    //     }
    // }
}
