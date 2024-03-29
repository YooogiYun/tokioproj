use bytes::Bytes;
use mini_redis::client;
use tokio::{sync::mpsc, sync::oneshot};

// provided by the requester and used bt the manager task to send
// the command response back to the requester
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    GET {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    SET {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    // create a new channel with capacity of at most 32.
    println!("start of client");
    let (tx, mut rx) = mpsc::channel(32);

    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        println!("start of manager");

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                GET { key, resp } => {
                    println!("Cmd is GET, key is {}", &key);
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                SET { key, value, resp } => {
                    println!("Cmd is SET, key is {},value is {:?}", &key, &value);
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Spawn two tasks, one setting a value and other querying for key that was
    // set.
    let t1 = tokio::spawn(async move {
        // tx.send("Sending from first handle").await.unwrap();
        println!("Start of task 1");

        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::GET {
            key: "foo".into(),
            resp: resp_tx,
        };

        // send the GET request
        tx.send(cmd).await.unwrap();
        println!("Task 1: Send done.");

        // Await the response
        let resp = resp_rx.await;
        println!("Task 1: GOT = {:?}", resp);
    });

    let t2 = tokio::spawn(async move {
        // tx2.send("Sending from scecond handle").await.unwrap();
        println!("Start of task 2");

        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::SET {
            key: "foo".into(),
            value: "bar".into(),
            resp: resp_tx,
        };

        // send the SET request
        tx2.send(cmd).await.unwrap();
        println!("Task 2: Send done.");

        // Await the response
        let resp = resp_rx.await;
        println!("Task 2: GOT = {:?}", resp);
    });

    t1.await.unwrap();
    t2.await.unwrap();

    manager.await.unwrap();
}

// #[tokio::main]
// async fn main() {
//     // estabish a connection to the server
//     let mut client = client::connect("127.0.0.1:6379").await.unwrap();

//     // spawn two tasks, one gets a key, the other sets a key
//     let t1 = tokio::spawn(async {
//         let res = client.get("foo").await;
//     });

//     let t2 = tokio::spawn(async {
//         client.set("foo", "bar".into()).await;
//     });

//     t1.await.unwrap();
//     t2.await.unwrap();
// }
