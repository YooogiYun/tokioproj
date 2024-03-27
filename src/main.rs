use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // process(socket).await; // it's only for one socket at same time.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    let mut db = HashMap::new();

    let mut connection = Connection::new(socket);

    while let Some(cmd) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(cmd).unwrap() {
            Set(data) => {
                db.insert(data.key().to_string(), data.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(data) => {
                if let Some(value) = db.get(data.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented! {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
