#[cfg(all(tokio_unstable, target_os = "linux", target_arch = "x86_64"))]
fn main() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::runtime::{FlightRecorder, PerfettoFlightRecorder};

    #[inline(never)]
    async fn foo() {
        bar().await
    }
    #[inline(never)]
    async fn bar() {
        baz().await
    }
    #[inline(never)]
    async fn baz() {
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

    let mut recorder = PerfettoFlightRecorder::new("./test.pftrace");
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

    // Dropping is required to flush all spans.
    drop(rt);

    recorder.flush_trace();
}

#[cfg(not(all(tokio_unstable, target_os = "linux", target_arch = "x86_64")))]
fn main() {}
