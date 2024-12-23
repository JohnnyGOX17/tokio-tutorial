use bytes::Bytes;
use mini_redis::Command::{self, Get, Set};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

// alias to help
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening...");

    // A hashmap to store data and can be hared across many tasks/threads
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The second item of the return tuple contains the IP and port of the new connection
        let (socket, ipaddr) = listener.accept().await.unwrap();

        // Clone the handle to the hash map.
        let db = db.clone();
        println!("Accepted from {}", ipaddr);

        // Spawn a new task for each inbound socket, and socket is moved to the new task and
        // processed there. Note while this looks like standard threads, Tokio tasks are "green
        // threads" in that they provide concurrency (not parallelism) within a single thread.
        // `tokio::spawn` returns a `JoinHandle` which a caller may wait/join on (similar to
        // regular threads) using `.await` and get a `Result` back. Spawning a task submits it to
        // the Tokio scheduler. A task could be moved between different runtime threads, even after
        // being spawned. Similar to threads, data/types used within a task must have `'static`
        // lifetimes, where there are no references to data owned outside the task. If data is to
        // be shared between tasks, can use other synchronization primitives like `Arc`.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    // The `Connection` lets us read/write redis **frames** instead of byte streams. The
    // `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // lock (acquire mutex) before using shared hashmap
                let mut db = db.lock().unwrap();

                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                // lock (acquire mutex) before using shared hashmap
                let db = db.lock().unwrap();

                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}
