#[cfg(all(tokio_unstable, target_os = "linux", target_arch = "x86_64"))]
fn main() {
    // use std::{
    //     fs::File,
    //     sync::atomic::{AtomicUsize, Ordering},
    // };

    // #[inline(never)]
    // async fn foo() {
    //     bar().await
    // }
    // #[inline(never)]
    // async fn bar() {
    //     baz().await
    // }
    // #[inline(never)]
    // async fn baz() {
    //     let mut handles = vec![];
    //     for i in 0..10 {
    //         handles.push(tokio::task::spawn(async move {
    //             // tokio::time::sleep(std::time::Duration::from_micros(i * 10)).await;

    //             use tokio::net::TcpStream;

    //             for _ in 0..1 {
    //                 tokio::spawn(async move {
    //                     match TcpStream::connect("example.com:80").await {
    //                         Ok(mut stream) => {
    //                             use tokio::io::{AsyncReadExt, AsyncWriteExt};

    //                             // println!("タスク {}: example.com に接続しました", i);

    //                             // 例として HTTP GET リクエストを送ってみる
    //                             if let Err(e) = stream
    //                                 .write_all(b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n")
    //                                 .await
    //                             {
    //                                 eprintln!("タスク {}: 書き込みエラー: {}", i, e);
    //                                 return;
    //                             }

    //                             // 簡単にレスポンスを受信して表示
    //                             let mut buf = vec![0u8; 1024];
    //                             match stream.read(&mut buf).await {
    //                                 Ok(n) => {
    //                                     // println!(
    //                                     //     "タスク {}: レスポンス受信 ({} バイト):\n{}",
    //                                     //     i,
    //                                     //     n,
    //                                     //     String::from_utf8_lossy(&buf[..n])
    //                                     // );
    //                                 }
    //                                 Err(e) => {
    //                                     eprintln!("タスク {}: 読み込みエラー: {}", i, e);
    //                                 }
    //                             }
    //                         }
    //                         Err(e) => {
    //                             eprintln!("タスク {}: 接続に失敗しました: {}", i, e);
    //                         }
    //                     }
    //                 });
    //             }
    //         }));
    //     }

    //     for handle in handles {
    //         let _ = handle.await;
    //     }
    // }

    // let mut file = File::create("./test.pftrace").unwrap();
    // let mut recorder = PerfettoFlightRecorder::new();
    // recorder.initialize();
    // recorder.start();

    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .thread_name_fn(|| {
    //         static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
    //         let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
    //         format!("tokio-runtime-worker-{}", id)
    //     })
    //     .build()
    //     .unwrap();

    // rt.block_on(async {
    //     tokio::spawn(async { foo().await }).await.unwrap();
    // });

    // // Dropping is required to flush all spans.
    // drop(rt);

    // recorder.flush_trace(&mut file);
}

#[cfg(not(all(tokio_unstable, target_os = "linux", target_arch = "x86_64")))]
fn main() {}
