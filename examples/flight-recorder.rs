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

    let flight_recorder = tokio::runtime::Handle::current().flihgt_recorder();

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
