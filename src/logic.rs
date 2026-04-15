use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub async fn run_consumer(
    client: &redis::Client,
    stream_name: String,
    group_name: String,
    consumer_name: String,
    token: CancellationToken,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut con_reader = client.get_multiplexed_async_connection().await?;
    
    let reader_handle = tokio::spawn(async move {
        let s = stream_name;
        let g = group_name;
        let c = consumer_name;

        println!("READER - started ({c})");

        loop {
            // temp value error to avoid select .
            let options = StreamReadOptions::default()
                .count(1)
                .block(2000)
                .group(g.clone(), c.clone());

            let keys = [s.as_str()]; // (String Slices)
            let ids = [">"];

            tokio::select! {
                // إشارة الإغلاق
                _ = token.cancelled() => {
                    println!("READER - {c} receives shutdown signal.");
                    break;
                }

                // reading process .. to figure the right types 
                res = con_reader.xread_options(&keys, &ids, &options) => {
                    match res {
                        Ok(Some(reply)) => {
                            let reply: StreamReadReply = reply;
                            for stream_key in reply.keys {
                                for stream_id in stream_key.ids {
                                    println!("READER - {g} - {c} - read: id: {}", stream_id.id);
                                    
                                    sleep(Duration::from_millis(400)).await;

                                    let res_ack: Result<(), _> = con_reader.xack(&s, &g, &[stream_id.id]).await;
                                    if let Err(err) = res_ack {
                                        println!("XREADGROUP - ERROR ACK: {err}");
                                    } else {
                                        println!("XREADGROUP - ACK OK");
                                    }
                                }
                            }
                        }
                        Ok(None) => continue,
                        Err(e) => {
                            println!("READER ERROR: {e}");
                            break;
                        }
                    }
                }
            }
        }
        println!("READER - {c} shutdown complete clean.");
    });

    Ok(reader_handle)
}

pub async fn create_group(
    client: &Client, 
    stream_name: &str, 
    group_name: &str
) -> Result<(), Box<dyn std::error::Error>> {
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

pub struct ValkeyConfig {
    pub url: String,
    pub stream: String,
    pub g1: String,
    pub g2: String,
}

impl ValkeyConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok(); 
        Self {
            url: std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            stream: std::env::var("PRIMARY_STREAM").unwrap_or_else(|_| "stream-c05".to_string()),
            g1: std::env::var("GROUP_ONE").unwrap_or_else(|_| "group_01".to_string()),
            g2: std::env::var("GROUP_TWO").unwrap_or_else(|_| "group_02".to_string()),
        }
    }
}