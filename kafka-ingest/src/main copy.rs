use kafka::consumer::Consumer;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let brokers: Vec<String> = vec![
        "b-1-public.prodmskcluster.gc4adq.c21.kafka.us-east-1.amazonaws.com:9196".to_string(),
        "b-2-public.prodmskcluster.gc4adq.c21.kafka.us-east-1.amazonaws.com:9196".to_string(),
        "b-3-public.prodmskcluster.gc4adq.c21.kafka.us-east-1.amazonaws.com:9196".to_string(),
    ];

    let mut consumer = Consumer::from_hosts(brokers)
        .with_topic_partitions("test-topic3".to_owned(), &[0, 1])
        .with_client_id("client_id".to_owned())
        // .with_fallback_offset(FetchOffset::Earliest)
        .with_group("test-group".to_owned())
        // .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create();

    if (consumer.is_err()) {
        println!("Error: {:?}", consumer.err());
    }
    // .unwrap();

    // loop {
    //     for ms in consumer.poll().unwrap().iter() {
    //         for m in ms.messages() {
    //             println!("{:?}", m);
    //         }
    //         let result = consumer.consume_messageset(ms);
    //     }
    //     consumer.commit_consumed().unwrap();
    // }
}
