use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;
const NUM_SHARDS: usize = 16;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let db: ShardedDb = get_sharded_db(NUM_SHARDS);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Clone the Arc
        // The clone() method for Arc creates a new Arc instance that shares ownership of the same underlying data as the original Arc.
        // It increases the reference count of the shared data,
        // ensuring that it remains alive as long as there is at least one active reference to it.
        let db = db.clone();

        // process(socket).await; // it's only for one socket at same time.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

fn get_sharded_db(num_shards: usize) -> ShardedDb {
    let mut sharded_db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        sharded_db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(sharded_db)
}

fn hash_key(key: &str) -> usize {
    let mut hasher = DefaultHasher::default();
    key.hash(&mut hasher);
    hasher.finish() as usize
}

async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(cmd) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(cmd).unwrap() {
            Set(data) => {
                // let mut db = db.lock().unwrap();
                let shard_index = hash_key(data.key()) % db.len();
                let mut db = db[shard_index].lock().unwrap();
                db.insert(data.key().to_string(), data.value().clone());
                println!(
                    "Process done: SET (key: {}, value: {:?})",
                    data.key(),
                    data.value()
                );
                Frame::Simple("OK".to_string())
            }
            Get(data) => {
                // let db = db.lock().unwrap();
                let shard_index = hash_key(data.key()) % db.len();
                let db = db[shard_index].lock().unwrap();
                if let Some(value) = db.get(data.key()) {
                    println!(
                        "Process done: GET (key: {},value: {:?})",
                        data.key(),
                        value.clone()
                    );
                    Frame::Bulk(value.clone().into())
                } else {
                    println!("Process done: GET (key: {},value: NUll)", data.key());
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented! {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
