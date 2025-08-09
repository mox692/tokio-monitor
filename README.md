# tokio-monitor
This is a fork project of [tokio](https://github.com/tokio-rs/tokio) with additional runtime monitoring support.

⚠️ It is still experimental, so I recommend to take caution before using this library in production. You can check the diffs between this fork and upstream tokio in this [link](https://github.com/mox692/tokio-monitor/compare/upstream...master).


# How to use
Just replace the dependency:
```diff
[dependencies]
-  tokio = { version = "1", features = ["full"] }
+ tokio-monitor = { git = "https://github.com/mox692/tokio-monitor", features = ["full"]}
```

Here is a simple example:

```rust,ignore
fn main() {
    use std::{
        fs::File,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use tokio::runtime::{FlightRecorder, PerfettoFlightRecorder};

    async fn foo() {
        let mut handles = vec![];
        for i in 0..10 {
            handles.push(tokio::task::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_micros(i * 10)).await;
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    let mut file = File::create("./test.pftrace").unwrap();
    let mut recorder = PerfettoFlightRecorder::new();
    recorder.initialize();
    recorder.start();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("tokio-runtime-worker-{}", id)
        })
        .build()
        .unwrap();

    rt.block_on(async {
        tokio::spawn(async { foo().await }).await.unwrap();
    });

    recorder.flush_trace(&mut file);
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
