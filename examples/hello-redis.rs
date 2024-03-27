use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("Hello", "world".into()).await?;

    println!(r#"Set ##Hello:world done."#);

    let result = client.get("Hello").await?;

    println!("Got {:?}:", result);

    Ok(())
}
