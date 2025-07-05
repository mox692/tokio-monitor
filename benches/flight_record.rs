use std::fs::File;
use std::time::Instant;

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};
use tokio::runtime::{Builder, Handle};
use tokio_util::task::TaskTracker;

const NUM_SPAWN: usize = 1000;

/// Benchmark Overview:
/// * Compare runtime tracing enabled vs. disabled.
/// * Compare backtrace enabled vs. disabled.
/// * Compare periodic flush patterns vs. no flush.
///
/// Variants:
/// 1. Spawn many small tasks with random yields.
/// 2. Spawn a few tasks.
/// 3. Real-world application comparison (e.g., using lodeo).
fn spawn_tasks(group: &mut BenchmarkGroup<WallTime>, trace_enabled: bool, periodic_flush: bool) {
    // Build a multi-threaded Tokio runtime with all features enabled.
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    // Create a descriptive name for this benchmark configuration.
    let bench_name = format!(
        "spawn_{}_tasks - trace: {}, flush: {}",
        NUM_SPAWN, trace_enabled, periodic_flush
    );

    group.bench_function(&bench_name, |bencher| {
        bencher.iter_custom(|iterations| {
            runtime.block_on(async {
                // Obtain the flight recorder from the current runtime handle.
                let recorder = Handle::current().flight_recorder();
                let mut trace_file =
                    File::create("test.pftrace").expect("Unable to create trace file");

                recorder.initialize();
                if trace_enabled {
                    recorder.start();
                }

                let tracker = TaskTracker::new();
                let start_time = Instant::now();

                for _ in 0..iterations {
                    for _ in 0..NUM_SPAWN {
                        tracker.spawn(async {});
                    }
                    tracker.close();
                    tracker.wait().await;

                    if periodic_flush {
                        recorder.flush_trace(&mut trace_file);
                    }
                }

                start_time.elapsed()
            })
        })
    });
}

/// Defines and runs benchmarks for different combinations of tracing and flushing.
fn perfetto_benchmarks(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flight_record");

    for &trace_enabled in &[true, false] {
        for &periodic_flush in &[true, false] {
            spawn_tasks(&mut group, trace_enabled, periodic_flush);
        }
    }

    group.finish();
}

criterion_group!(benches, perfetto_benchmarks);
criterion_main!(benches);
