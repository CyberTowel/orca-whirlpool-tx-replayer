use std::collections::HashMap;

use crate::{
    actions::{ActionFields, CtAction, SwapFields},
    interfaces::ValueChange,
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

            if (elements[0].amount.is_positive() && elements[1].amount.is_negative()) {
                elements.reverse();
                // let temp = elements[0].clone();
                // elements[0] = elements[1].clone();
                // elements[1] = temp;
            }

            let key_1 = elements[0].to.clone().unwrap_or("_".to_string());
            let key_2 = elements[1].to.clone().unwrap_or("_".to_string());
            let mut from_to_key = key_1.clone() + "_" + &key_2.clone();

            if (key_1 < key_2) {
                from_to_key = key_2.clone() + "_" + &key_1.clone();
            }

            if (tokens_keys_used.contains_key(&from_to_key)) {
                continue;
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

    let testing32 = other
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

                if (key_1 < key_2) {
                    from_to_key = key_2.clone() + "_" + &key_1.clone();
                }

                return !tokens_keys_used.contains_key(&from_to_key);
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
