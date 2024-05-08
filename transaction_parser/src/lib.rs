mod pool_state;
pub mod rpc_pool_manager;
mod token_parser;
pub mod transaction;
pub mod transactions_loader;

pub fn add(signature: String) {
    println!("Testing {}", signature);
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // pub fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
