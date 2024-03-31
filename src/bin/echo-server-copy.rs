use bytes::buf;
use tokio::io::{AsyncWriteExt, BufReader};

use tokio::io::{self, AsyncReadExt};

use tokio::net::TcpListener;

#[tokio::main]
//server.rs
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (mut rd, mut wr) = socket.split();

            loop {
                let mut buffer = vec![0; 1024];
                if rd.read(&mut buffer).await.is_err() {
                    eprintln!("Fail to read value");
                    return;
                };
                let str_value = match String::from_utf8(buffer) {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("Error on parsing value: {}", e);
                        return;
                    }
                };
                println!("Server GOT from reader: {}", str_value);
                let mut reader = BufReader::new(str_value.as_bytes());
                if io::copy(&mut reader, &mut wr).await.is_err() {
                    eprintln!("Fail to echo value");
                    return;
                };
            }
        });

        // tokio::spawn(async move {
        //     let mut buf = vec![0; 1024];
        //     loop {
        //         let reader = socket.read(&mut buf).await;
        //         println!(
        //             "Reader read done. The value in the buffer is: {:?}",
        //             String::from_utf8_lossy(&mut buf[..])
        //         );
        //         match reader {
        //             // Return value of `Ok(0)` signifies that the remote has
        //             // closed
        //             Ok(0) => return,
        //             Ok(n) => {
        //                 // Copy the data back to socket
        //                 if socket.write_all(&buf[..n]).await.is_err() {
        //                     // Unexpected socket error. There isn't much we can
        //                     // do here so just stop processing.
        //                     return;
        //                 }
        //             }
        //             Err(_) => {
        //                 // Unexpected socket error. There isn't much we can do
        //                 // here so just stop processing.
        //                 return;
        //             }
        //         }
        //     }
        // });
    }
}
