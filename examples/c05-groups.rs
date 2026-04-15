use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1:6379")?;
    let stream_name = "stream-c05";

    // -- Writer Task
    let mut con_writer = client.get_multiplexed_async_connection().await?;
    let writer_handle = tokio::spawn(async move {
        println!("WRITER - started");
        for i in 0..10 {
            let id: String = con_writer
                .xadd("stream-c05", "*", &[("val", &i.to_string())])
                .await
                .expect("XADD Fail");
            println!("WRITER - sent 'val: {i}' with id: {id}");
            sleep(Duration::from_millis(200)).await;
        }
    });

    // -- Create groups
    let group_01 = "group_01";
    create_group(&client, stream_name, group_01).await?;
    let group_02 = "group_02";
    create_group(&client, stream_name, group_02).await?;

    // -- Run consumers
  
    let consumer_g01_a_handle = run_consumer(&client, stream_name.to_string(), group_01.to_string(), "consumer_g01_a".to_string()).await?;
    let consumer_g01_b_handle = run_consumer(&client, stream_name.to_string(), group_01.to_string(), "consumer_g01_b".to_string()).await?;
    let consumer_g02_a_handle = run_consumer(&client, stream_name.to_string(), group_02.to_string(), "consumer_g02_a".to_string()).await?;

    // -- Wait for the tasks
    writer_handle.await?;
    consumer_g01_a_handle.await?;
    consumer_g01_b_handle.await?;
    consumer_g02_a_handle.await?;
	// we used consumers to decrease waiting time for task and handle more reqs .
	// when we have increasing no of users we create new consumers not groups .
	// Groups is for handeling new tasks not users .

	// previously we used reader_handler because it was in main fn
	// but now we put reader responsibilty in consumer fn . 
    // -- Clean up the stream
    let mut con = client.get_multiplexed_async_connection().await?;
    let count: i32 = con.del(stream_name).await?;
    println!("Stream '{stream_name}' deleted ({count} key).");

    Ok(())
}

pub async fn run_consumer(
    client: &Client,
    stream_name: String,
    group_name: String,
    consumer_name: String,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
	// joinHandle : calls main after task finished , without joinHandle main will process task in millSec
	// which will lead to dead readers before even reading one msg 
    let mut con_reader = client.get_multiplexed_async_connection().await?;
    let reader_handle = tokio::spawn(async move {
        let consumer = consumer_name;
        let s_name = stream_name;
        let g_name = group_name;

        println!("READER - started ({consumer})");
        
        loop {
            let options = StreamReadOptions::default()
                .count(1)
                .block(2000)
                .group(&g_name, &consumer);

            let res: Option<StreamReadReply> = con_reader
                .xread_options(&[&s_name], &[">"], &options)
                .await
                .expect("Fail to xread");

            if let Some(reply) = res {
                for stream_key in reply.keys {
                    for stream_id in stream_key.ids {
                        println!(
                            "READER - {g_name} - {consumer} - read: id: {} - fields: {:?}",
                            stream_id.id, stream_id.map
                        );
                        sleep(Duration::from_millis(400)).await;
                        
                        let res_ack: Result<(), _> = con_reader.xack(&s_name, &g_name, &[stream_id.id]).await;
                        if let Err(e) = res_ack {
                            println!("XREADGROUP - ERROR ACK: {e}");
                        } else {
                            println!("XREADGROUP - ACK OK");
                        }
                    }
                }
            } else {
                println!("READER - timeout, assuming writer is done.");
                break;
            }
        }
        println!("READER - finished");
    });
    Ok(reader_handle)
}

pub async fn create_group(client: &Client, stream_name: &str, group_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let group_create_res: Result<(), _> = con.xgroup_create_mkstream(stream_name, group_name, "0").await;
    if let Err(err) = group_create_res {
        if let Some("BUSYGROUP") = err.code() {
            println!("XGROUP - group '{group_name}' ALREADY created");
        } else {
            return Err(err.into());
        }
    } else {
        println!("XGROUP - group '{group_name}' created");
    }
    Ok(())
}