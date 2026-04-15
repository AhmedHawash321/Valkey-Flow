use redis::{Client, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::open("redis://127.0.0.1:6379")?;

    let mut con = client.get_connection()?;

    let _: () = con.set("my_key", 42)?;

    let res: i32 = con.get("my_key")?;

    println!("my key result is: {res}");
    Ok(())
}