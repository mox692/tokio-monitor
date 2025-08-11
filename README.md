# tokio-monitor
This is a fork project of [tokio](https://github.com/tokio-rs/tokio) with additional runtime monitoring support.

⚠️ It is still experimental, so I recommend to take caution before using this library in production. You can check the diffs between this fork and upstream tokio in this [link](https://github.com/mox692/tokio-monitor/compare/upstream...master).


# How to use
Just replace the dependency:
```diff
[dependencies]
-  tokio = { version = "1", features = ["full"] }
+ tokio = { package = "tokio-monitor", version = "1", features = ["full"]}
```

Here is a simple example (you can also refer to `examples/flight-recorder.rs`):

```rust,ignore
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        run().await;
    });
}

async fn run() {
    // Initialize the flight recorder

    let flight_recorder = tokio::runtime::Handle::current().flight_recorder();

    flight_recorder.initialize();
    flight_recorder.start();

    // Spawn some tasks
    let mut handles = Vec::new();
    for i in 0..100 {
        handles.push(tokio::spawn(async move {
            // Simulate some work
            tokio::time::sleep(std::time::Duration::from_micros(i * 100)).await;
            println!("Task {} completed", i);
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Flush the trace to a file
    let mut file = std::fs::File::create("./test.pftrace").unwrap();
    flight_recorder.flush_trace(&mut file);
}
```

The trace output would be created in your current directory `./trace.pftrace`. Then you
can use [Perfetto-UI](https://ui.perfetto.dev/) to visualize the trace.


# Features

### Backtrace Features

See `examples/flight-recorder-backtrace.rs`.

### Symbolize

```bash
RUST_BACKTRACE=1 cargo run --package examples --example symbolize-perfetto -- ./test.pftrace ./target/debug/examples/flight-recorder-backtrace
```
