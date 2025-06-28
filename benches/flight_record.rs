use std::fs::File;
use std::time::Instant;

/// # 比較
/// * rt-trace をONにしてる時と, offにしてる時の比較
/// * backtraceをONにしてる時と, offにしてる時の比較
/// * flushを定期的にするパターンとi, しないパターン
///
/// # コード
/// * ケース1: 小さいタスクを大量にspawnするケース
///     * ランダムなyield
/// * ケース2: 少量のtaskをspawnするケース
/// * ケース3: 実際のアプリケーションでの比較 (lodeoとか使っていいかなw)
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::measurement::WallTime;
use criterion::BenchmarkGroup;
use criterion::Criterion;
use tokio::runtime::Handle;

const NUM_SPAWN: usize = 1000;

fn spawn_many_tasks(g: &mut BenchmarkGroup<WallTime>, enable: bool, flush: bool) {
    use tokio_util::task::TaskTracker;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let name = format!(
        "spawn_{NUM_SPAWN}_tasks - enable: {}, flush: {}",
        enable, flush
    );

    g.bench_function(name, |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let flight_recorder = Handle::current().flihgt_recorder();
                let mut file = File::create("./test.pftrace").unwrap();
                flight_recorder.initialize();
                if enable {
                    flight_recorder.start();
                }
                let tracker = TaskTracker::new();

                let start = Instant::now();
                for _ in 0..iters {
                    for i in 0..NUM_SPAWN {
                        tracker.spawn(async move {});
                    }
                    tracker.close();
                    tracker.wait().await;
                    if flush {
                        flight_recorder.flush_trace(&mut file);
                    }
                }

                let dur = start.elapsed();
                dur
            })
        })
    });
}

fn perfetto(c: &mut Criterion) {
    let mut bgroup = c.benchmark_group("flight_record");

    for enable in [true, false] {
        for flush in [true, false] {
            spawn_many_tasks(&mut bgroup, enable, flush);
        }
    }

    bgroup.finish();
}

criterion_group!(benches, perfetto);
criterion_main!(benches);
