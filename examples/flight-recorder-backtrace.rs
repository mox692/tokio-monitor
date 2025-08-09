#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), target_os = "macos"))]
fn main() {
    use std::{
        fs::File,
        sync::atomic::{AtomicUsize, Ordering},
    };

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
        let num_iter = 10000;
        let mut handles = vec![];
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        handles.push(tokio::task::spawn(async move {
            for _ in 0..num_iter {
                rx.recv().await.unwrap();
            }
        }));
        for _ in 0..num_iter {
            let tx = tx.clone();
            handles.push(tokio::task::spawn(async move {
                tx.send(()).await.unwrap();
                tokio::task::yield_now().await;
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    let mut file = File::create("./test.pftrace").unwrap();

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
        let flight_recorder = tokio::runtime::Handle::current().flight_recorder();
        flight_recorder.initialize();
        flight_recorder.start();
        tokio::spawn(async { foo().await }).await.unwrap();

        flight_recorder.flush_trace(&mut file);
    });

    // Dropping is required to flush all spans.
    drop(rt);
}

#[cfg(not(any(all(target_os = "linux", target_arch = "x86_64"), target_os = "macos")))]
fn main() {}
