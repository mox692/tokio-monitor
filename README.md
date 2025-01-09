# tokio-monitor
This is a fork project of [tokio](https://github.com/tokio-rs/tokio) with additional runtime monitoring support.

⚠️ It is still experimental, so I recommend to take caution before using this library in production. You can check the diffs between this fork and upstream tokio in this [link](https://github.com/mox692/tokio-monitor/compare/upstream...master).


# How to use
Just replace the dependency:
```diff
[dependencies]
- tokio = "1"
+ tokio-monitor = { git = "https://github.com/mox692/tokio-monitor" }
```

Here is a simple example:

```rust,ignore
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing_perfetto::external::tokio::TokioPerfettoLayerBuilder;
use tracing_subscriber::prelude::*;

fn main() {
    let layer = TokioPerfettoLayerBuilder::new()
        .file_name("./test.pftrace")
        .build();

    tracing_subscriber::registry().with(layer).init();

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
        tokio::spawn(async { run().await }).await.unwrap();
    });
}

async fn run() {
    let mut handles = vec![];
    for i in 0..10000 {
        handles.push(tokio::task::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_micros(i * 10)).await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
}

```

The trace output would be created in your current directory `./trace.pftrace`. Then you
can use [Perfetto-UI](https://ui.perfetto.dev/) to visualize the trace.


# Symbolize
```bash
cd tracing-perfetto \ 
cargo run --features symbolize --package tracing-perfetto --bin perfetto_symbolize \
    -- --bin-path ./target/debug/examples/runtime-tracing  --perfetto-trace-log ./test.pftrace
```
