pub mod block_parser;
pub mod interfaces;
mod pool_state;
pub mod rpc_pool_manager;
pub mod token_db;
pub mod token_parser;
pub mod transaction;
pub mod transactions_loader;

pub fn add(signature: String) {
    println!("Testing {}", signature);
}

#[cfg(test)]
mod tests {

    // #[test]
    // pub fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
