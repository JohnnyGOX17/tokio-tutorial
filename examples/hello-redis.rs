use mini_redis::{client, Result};

// Async fn's must be executed by a runtime, which provides async task scheduling, evented I/O,
// timers, etc. `#[tokio::main]` macro turns our `async fn main()` into a synchronous `fn main()`
#[tokio::main]
async fn main() -> Result<()> {
    // Asynchronously open a TCP connection to the mini-redis address.
    // Though it looks like a synchronous call, the `await` call of the `async` function yields
    // control back to to the main thread while we wait for the operation (e.g. a possibly lengthy
    // TCP connection operation that needs external network I/O) to complete.
    // Note in Rust, async operations are "lazy", they are not run until the `.wait` is used.
    let mut myclient = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    myclient.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = myclient.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}
