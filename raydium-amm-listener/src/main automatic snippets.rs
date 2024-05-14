// use flume::{unbounded, Receiver, Sender};
// use scheduler::consumer;
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::Arc;
// use std::time::Duration;
// use tokio::sync::Mutex;
// use tokio::task;
// mod scheduler;

use flume::{Receiver, Sender};
use std::thread;

fn main() {
    // Create a new unbounded SPMC channel
    let (sender, receiver) = flume::unbounded();

    // Spawn multiple consumer threads
    let receivers = receiver
        .clone()
        .iter()
        .take(3)
        .map(|r| {
            thread::spawn(move || {
                consumer(r);
            })
        })
        .collect::<Vec<_>>();

    // Producer thread
    tokio::spawn(async move || {
        producer(sender);
    });

    // Wait for consumer threads to finish
    for receiver in receivers {
        receiver.join().unwrap();
    }
}

fn producer(sender: Sender<u64>) {
    for i in 0..10 {
        sender.send(i).unwrap();
    }
}

fn consumer(receiver: Receiver<u64>) {
    while let Ok(value) = receiver.recv() {
        println!("Received: {}", value);
    }
}

// async fn producer(tx: Sender<usize>, counter: Arc<AtomicUsize>) {
//     let start_number = counter.load(Ordering::SeqCst);
//     // let mut number = start_number;
//     // for _ in 0..3 {
//     println!("Producer sending: {}", start_number);
//     tx.send_async(start_number).await.unwrap();
//     // number += 1;
//     tokio::time::sleep(Duration::from_secs(1)).await;
//     // }
// }

// #[derive(Debug)]
// enum Message {
//     Data(/* your data type */),
//     Completed,
// }

// #[tokio::main]
// async fn main() {
//     let (tx, rx) = unbounded();

//     let (data_tx, data_rx): (Sender<Message>, Receiver<Message>) = unbounded();

//     let initial_number = 20000;
//     let counter = Arc::new(AtomicUsize::new(initial_number)); // Start after initial tasks

//     // Spawn producers
//     // for i in 1..=3 {
//     let tx_clone = tx.clone();
//     let counter_clone = counter.clone();
//     tokio::spawn(async move {
//         // let block_to_get = counter.fetch_add(1, Ordering::SeqCst);
//         producer(tx_clone, counter_clone).await;
//     });
//     // }

//     // Spawn consumers
//     for i in 1..=2 {
//         // let tx_clone = tx.clone();
//         let rx_clone = rx.clone();
//         let counter_clone = counter.clone();
//         let tesitng = data_tx.clone();
//         tokio::spawn(async move {
//             let start_number = counter.load(Ordering::SeqCst);
//             consumer(rx_clone, i, start_number).await;
//             println!("Task completed!");
//             tesitng.send(Message::Completed).unwrap();
//         });
//     }

//     // Receive messages from the channel
//     while let Ok(msg) = data_rx.recv() {
//         println!("Received message: {:#?}", msg);
//         // match msg {
//         //     Message::Data(data) => {
//         //         // Handle data
//         //     }
//         //     Message::Completed => {
//         //         // Handle completion event
//         //         println!("Task completed!");
//         //     }
//         // }
//     }

//     // Let the system run for a while
//     // tokio::time::sleep(Duration::from_secs(10)).await;
// }
