use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct CtSwap {
    action_type: String,
    protocol_name: Option<String>,
    protocol_id: Option<String>,
    protocol: Option<String>,
    addresses: Vec<String>,
    event_ids: Vec<String>,
    u_bwallet_address: Option<String>,
    // fields: SwapFields,
}

// #[derive(Deserialize, Serialize, Debug)]
// struct SwapFields {
//     tokens_fee: Vec<TokenChange>,
//     tokens_from: Vec<TokenChange>,
//     tokens_to: Vec<TokenChange>,
//     swap_hops: Vec<String>,
//     router_events: Vec<String>,
//     testing: bool,
// }

pub fn parse_token_changes_to_swaps(
    address_token_changes: Option<Vec<String>>,
    // transaction_from: Option<String>,
) {
    // let _other: Vec<AddressTokenChange> = Vec::new();
    let _actions: Vec<CtSwap> = Vec::new();

    if let Some(changes) = address_token_changes {
        for _item in changes {
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
        }
    }

    // (actions, other)
}
