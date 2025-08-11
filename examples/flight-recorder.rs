use std::{fs::File, io::Result, time::Duration};
use tokio::{runtime::Handle, task, time};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Get the flight recorder handle from the current runtime handle
    let recorder = Handle::current().flight_recorder();
    // Initialize the flight recorder
    recorder.initialize();
    // Start the flight recorder
    recorder.start();

    // Spawn tasks
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            task::spawn(async move {
                time::sleep(Duration::from_micros((i * 100) as u64)).await;
                println!("Task {} completed", i);
            })
        })
        .collect();

    // Await all tasks
    for t in tasks {
        let _ = t.await;
    }

    let mut file = File::create("./test.pftrace")?;

    // This will write the trace to the file
    recorder.flush_trace(&mut file);

    Ok(())
}
