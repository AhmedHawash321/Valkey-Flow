use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use valkey_flow;

#[tokio::test]
async fn test_full_stream_flow() {
    let cfg = valkey_flow::ValkeyConfig::from_env();
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let stream_name = "test_stream_logic";
    let group_name = "test_group_logic";
    let token = CancellationToken::new();

    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let _: () = redis::cmd("DEL")
        .arg(&stream_name)
        .query_async(&mut con)
        .await
        .unwrap();
    // cmd "DEL" : it delete any old data , to start in cleanliness .

    valkey_flow::create_group(&client, stream_name, group_name)
        .await
        .expect("Failed to create group");
    // we call create group fn to create consumer group .

    let _: String = redis::cmd("XADD")
        .arg(&stream_name)
        .arg("*")
        .arg("val")
        .arg("test_123")
        // we play sender role as we send real msg to server with "test123", now its waiting for read .
        .query_async(&mut con)
        .await
        .unwrap();

    let handle = valkey_flow::run_consumer(
        &client,
        stream_name.to_string(),
        group_name.to_string(),
        "test_consumer".to_string(),
        token.clone(),
    )
    // run_consumer will get msg then print it , finally and most imp it will make ACK(tell server msg had been received) .
    .await
    .expect("Failed to run consumer");

    // Sleep for 1 sec as we need time for reading msgs , and making ACK .
    sleep(Duration::from_millis(1000)).await;

    let pending: redis::streams::StreamPendingReply = redis::cmd("XPENDING")
        .arg(&stream_name)
        .arg(&group_name)
        .query_async(&mut con)
        .await
        .unwrap();

    token.cancel();
    let _ = handle.await;
    assert_eq!(
        pending.count(),
        0,
        "There are still pending messages! ACK might have failed."
    );
    // final Step : Here We are asking if there are any pending msgs , if it done correctly server will
    // give us 0 , assert_eq! : make sure that return is 0 if not test will fail .

    println!("Full flow test passed: Message sent, received, and acknowledged!");
}
