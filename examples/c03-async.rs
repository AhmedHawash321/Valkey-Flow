use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::time::sleep;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1:6379")?;
    // we didnt get connection like past 2 files .. Notice later we will use (getmultiplexed)
	let stream_name = "stream-c03";

	// -- Writer Task
	let mut con_writer = client.get_multiplexed_async_connection().await?;
    // tokio::spawen take a part of task and worked it in parallel or concurrent .
    // move means any variable outside block will completely transed to be owned by the task .
	// Result is like try/catch in js .
	let writer_handle = tokio::spawn(async move {
		println!("WRITER - started");
		for i in 0..5 { // why 5 ? its for testing since we put sleep every 200ms so 5 tries will cost us 1s
			// to make sure tasks are working together concurruncy .
			let id: String = con_writer
				.xadd(stream_name, "*", &[("val", &i.to_string())])
				.await
				.expect("XADD Fail"); // in case error or intruption happend throw this msg
			println!("WRITER - sent 'val: {i}' with id: {id}");
			sleep(Duration::from_millis(200)).await; // tell task to sleep between each 2 msgs
		}
	});

	// -- Reader Task
    // we created a new con here because we used spawn move in last task
    // which lead to an error if we dont use new con in the new reader task , because of the ownership .
	let mut con_reader = client.get_multiplexed_async_connection().await?;
	let reader_handle = tokio::spawn(async move {
		println!("READER - started");
		let mut last_id = "0-0".to_string();// starting with 1st msg in stream .
		let options = StreamReadOptions::default().count(1).block(2000);
		// count(1) : means redis will give us only 1 msg each time . 
		// block is important not to get into busy waiting condition .

		loop {
			let res: Option<StreamReadReply> = con_reader
			// it means any res in 2 s it will be some , if not then it will give none .
				.xread_options(&[stream_name], &[&last_id], &options)
				.await
				.expect("Fail to xread");

			if let Some(reply) = res {
				for stream_key in reply.keys {
					for stream_id in stream_key.ids { // as it might be more than 1 msg each time . 
						println!("READER - read: id: {} - fields: {:?}", stream_id.id, stream_id.map);
						println!("READER - SLEEP 800ms");
						sleep(Duration::from_millis(800)).await;
						// notice here writer send every 200ms , and reader respond every 800ms
						// this is called consumer lag , as redis here will work as Buffer .

						last_id = stream_id.id; // update new id , reader will ask redis last data after this id ,
						// so we never read msg twice .
					}
				}
			} else {
				println!("READER - timeout, assuming writer is done.");
				break;
			}
		}
		println!("READER - finished");
	});

	// -- Wait for the tasks
	writer_handle.await?;
	reader_handle.await?;
	// we are telling main fun not to end the program unless write, read finish first .

	// -- Clean up the stream
	let mut con = client.get_multiplexed_async_connection().await?;
	let count: i32 = con.del(stream_name).await?;
	println!("Stream '{stream_name}' deleted ({count} key).");

	Ok(())
}