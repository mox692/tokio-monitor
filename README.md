# tokio-monitor
This is a fork project of [tokio](https://github.com/tokio-rs/tokio) with additional runtime monitoring support.

⚠️ It is still experimental, so I recommend to take caution before using this library in production.  
You can check the diffs between this fork and upstream tokio in this [link](https://github.com/mox692/tokio-monitor/compare/upstream...master).


# How to use
Just replace the dependency:
```diff
[dependencies]
- tokio = "1"
+ tokio-monitor = "0.1"
```

Here is a simple example:

```rust
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = TokioPerfettoLayerBuilder::new()
        .file_name("./trace.pftrace")
        .build();

    tracing_subscriber::registry().with(layer).init();

    // your logic
}
```

The trace output would be created in your current directory `./trace.pftrace`.

# Symbolize
```bash
cd tracing-perfetto \ 
cargo run --features symbolize --package tracing-perfetto --bin perfetto_symbolize \
    -- --bin-path ./target/debug/examples/runtime-tracing  --perfetto-trace-log ./test.pftrace
```


# Motivation

