use std::fmt::format;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use tokio::net::TcpStream;

const MAX_TIMES: u8 = 12;

#[tokio::main]
// client.rs
async fn main() -> io::Result<()> {
    let mut client = TcpStream::connect("127.0.0.1:6142").await?;

    let times = write_data(&mut client, MAX_TIMES).await?;

    // reconnect when the connection is closed even it not reach the MAX_TIMES
    if times < MAX_TIMES {
        write_data(&mut client, MAX_TIMES - times).await?;
    }

    Ok(())
}

async fn write_data(client: &mut TcpStream, times: u8) -> io::Result<u8> {
    let mut x = 0;

    while x < times {
        let data = format!("No{}. Data from cilent: {}", x, "Hello");
        // println!("data len: {}", &data.as_bytes().len());
        client.write(data.as_bytes()).await?;
        x = x + 1;
        // println!("{}", x);
        // try to read the response from server
        let mut buffer = [0; 1024];
        match client.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                // if the count of bytes is 0, the connection is closed
                eprintln!("Connection closed by peer");
                break;
            }
            Ok(n) => {
                println!(
                    "Received response: {}",
                    String::from_utf8_lossy(&buffer[..n])
                );
            }
            Err(e) => {
                eprintln!("Failed to read response: {}", e);
                break;
            }
        }
    }
    Ok(x)
}
