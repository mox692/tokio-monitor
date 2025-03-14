// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use std::time::Duration;

use fastrace::collector::Config;
use fastrace::collector::ConsoleReporter;
use fastrace::collector::TestReporter;
use fastrace::local::LocalCollector;
use fastrace::prelude::*;
use fastrace::util::tree::tree_str_from_span_records;
use serial_test::serial;
use tokio::runtime::Builder;

fn four_spans() {
    {
        // wide
        for i in 0..2 {
            let _span = LocalSpan::enter_with_local_parent(format!("iter-span-{i}"))
                .with_property(|| ("tmp_property", "tmp_value"));
        }
    }

    {
        #[trace(name = "rec-span")]
        fn rec(mut i: u32) {
            i -= 1;

            if i > 0 {
                rec(i);
            }
        }

        // deep
        rec(2);
    }
}

#[test]
#[serial]
fn single_thread_single_span() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let root = Span::root("root", SpanContext::random());
        let _g = root.set_local_parent();

        four_spans();
    };

    fastrace::flush();

    let expected_graph = r#"
root []
    iter-span-0 [("tmp_property", "tmp_value")]
    iter-span-1 [("tmp_property", "tmp_value")]
    rec-span []
        rec-span []
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );
}

#[test]
#[serial]
fn single_thread_multiple_spans() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let root1 = Span::root("root1", SpanContext::new(TraceId(12), SpanId::default()));
        let root2 = Span::root("root2", SpanContext::new(TraceId(13), SpanId::default()));
        let root3 = Span::root("root3", SpanContext::new(TraceId(14), SpanId::default()));

        let local_collector = LocalCollector::start();

        four_spans();

        let local_spans = local_collector.collect();

        root1.push_child_spans(local_spans.clone());
        root2.push_child_spans(local_spans.clone());
        root3.push_child_spans(local_spans);
    }

    fastrace::flush();

    let expected_graph1 = r#"
root1 []
    iter-span-0 [("tmp_property", "tmp_value")]
    iter-span-1 [("tmp_property", "tmp_value")]
    rec-span []
        rec-span []
"#;
    let expected_graph2 = r#"
root2 []
    iter-span-0 [("tmp_property", "tmp_value")]
    iter-span-1 [("tmp_property", "tmp_value")]
    rec-span []
        rec-span []
"#;
    let expected_graph3 = r#"
root3 []
    iter-span-0 [("tmp_property", "tmp_value")]
    iter-span-1 [("tmp_property", "tmp_value")]
    rec-span []
        rec-span []
"#;
    assert_eq!(
        tree_str_from_span_records(
            collected_spans
                .lock()
                .iter()
                .filter(|s| s.trace_id == TraceId(12))
                .cloned()
                .collect()
        ),
        expected_graph1
    );
    assert_eq!(
        tree_str_from_span_records(
            collected_spans
                .lock()
                .iter()
                .filter(|s| s.trace_id == TraceId(13))
                .cloned()
                .collect()
        ),
        expected_graph2
    );
    assert_eq!(
        tree_str_from_span_records(
            collected_spans
                .lock()
                .iter()
                .filter(|s| s.trace_id == TraceId(14))
                .cloned()
                .collect()
        ),
        expected_graph3
    );
}

#[test]
#[serial]
fn multiple_spans_without_local_spans() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let root1 = Span::root("root1", SpanContext::new(TraceId(12), SpanId::default()));
        let root2 = Span::root("root2", SpanContext::new(TraceId(13), SpanId::default()));
        let mut root3 = Span::root("root3", SpanContext::new(TraceId(14), SpanId::default()));

        let local_collector = LocalCollector::start();

        let local_spans = local_collector.collect();
        root1.push_child_spans(local_spans.clone());
        root2.push_child_spans(local_spans.clone());
        root3.push_child_spans(local_spans);

        root3.cancel();
    }

    fastrace::flush();

    assert_eq!(
        collected_spans
            .lock()
            .iter()
            .filter(|s| s.trace_id == TraceId(12))
            .count(),
        1
    );
    assert_eq!(
        collected_spans
            .lock()
            .iter()
            .filter(|s| s.trace_id == TraceId(13))
            .count(),
        1
    );
    assert_eq!(
        collected_spans
            .lock()
            .iter()
            .filter(|s| s.trace_id == TraceId(14))
            .count(),
        0
    );
}

#[test]
#[serial]
fn multiple_local_parent() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let root = Span::root("root", SpanContext::random());
        let _g = root.set_local_parent();
        let _g = LocalSpan::enter_with_local_parent("span1");
        let span2 = Span::enter_with_local_parent("span2");
        {
            let _g = span2.set_local_parent();
            let _g = LocalSpan::enter_with_local_parent("span3");
        }
        let _g = LocalSpan::enter_with_local_parent("span4");
    }

    fastrace::flush();

    let expected_graph = r#"
root []
    span1 []
        span2 []
            span3 []
        span4 []
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );
}

#[test]
#[serial]
fn early_local_collect() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let local_collector = LocalCollector::start();
        let _g1 = LocalSpan::enter_with_local_parent("span1");
        let _g2 = LocalSpan::enter_with_local_parent("span2");
        drop(_g2);
        let local_spans = local_collector.collect();

        let root = Span::root("root", SpanContext::random());
        root.push_child_spans(local_spans);
    }

    fastrace::flush();

    let expected_graph = r#"
root []
    span1 []
        span2 []
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );
}

#[test]
#[serial]
fn max_spans_per_trace() {
    #[trace(short_name = true)]
    fn recursive(n: usize) {
        if n > 1 {
            recursive(n - 1);
        }
    }

    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default().max_spans_per_trace(Some(5)));

    {
        let root = Span::root("root", SpanContext::random());

        {
            let _g = root.set_local_parent();
            recursive(3);
        }
        {
            let _g = root.set_local_parent();
            recursive(3);
        }
        {
            let _g = root.set_local_parent();
            recursive(3);
        }
        {
            let _g = root.set_local_parent();
            recursive(3);
        }
    }

    fastrace::flush();

    let expected_graph = r#"
root []
    recursive []
        recursive []
            recursive []
    recursive []
        recursive []
            recursive []
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );
}

#[test]
#[serial]
fn test_elapsed() {
    fastrace::set_reporter(ConsoleReporter, Config::default());

    {
        let root = Span::root("root", SpanContext::random());

        std::thread::sleep(Duration::from_millis(50));

        assert!(root.elapsed().unwrap() >= Duration::from_millis(50));
    }

    fastrace::flush();
}

#[test]
#[serial]
fn test_add_property() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());

    {
        let root = Span::root("root", SpanContext::random());
        let _g = root.set_local_parent();
        LocalSpan::add_property(|| ("k1", "v1"));
        LocalSpan::add_properties(|| [("k2", "v2")]);
        let _span = LocalSpan::enter_with_local_parent("span");
        LocalSpan::add_property(|| ("k3", "v3"));
        LocalSpan::add_properties(|| [("k4", "v4"), ("k5", "v5")]);
    }

    fastrace::flush();

    let expected_graph = r#"
root [("k1", "v1"), ("k2", "v2")]
    span [("k3", "v3"), ("k4", "v4"), ("k5", "v5")]
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );
}

#[test]
#[serial]
fn test_not_sampled() {
    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());
    {
        let root = Span::root("root", SpanContext::random().sampled(true));
        let _g = root.set_local_parent();
        let _span = LocalSpan::enter_with_local_parent("span");
    }
    fastrace::flush();
    let expected_graph = r#"
root []
    span []
"#;
    assert_eq!(
        tree_str_from_span_records(collected_spans.lock().clone()),
        expected_graph
    );

    let (reporter, collected_spans) = TestReporter::new();
    fastrace::set_reporter(reporter, Config::default());
    {
        let root = Span::root("root", SpanContext::random().sampled(false));
        let _g = root.set_local_parent();
        let _span = LocalSpan::enter_with_local_parent("span");
    }
    fastrace::flush();
    assert!(collected_spans.lock().is_empty());
}
