use core::num;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        // process(socket).await; // it's only for one socket at same time.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

fn get_sharded_db(num_shard: usize) -> ShardedDb {
    let mut sharded_db = Vec::with_capacity(num_shard);
    for _ in 0..num_shard {
        sharded_db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(sharded_db)
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(cmd) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(cmd).unwrap() {
            Set(data) => {
                let mut db = db.lock().unwrap();
                db.insert(data.key().to_string(), data.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(data) => {
                let db = db.lock().unwrap();
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
