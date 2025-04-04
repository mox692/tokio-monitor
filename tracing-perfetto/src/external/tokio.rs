use crate::{get_trace_enable, set_trace_enable, PerfettoLayer};
use std::path::PathBuf;
use std::{fs::File, sync::Mutex};
use tracing::{span, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

pub struct TokioPerfettoLayer {
    inner: PerfettoLayer<TokioPerfettoWriter>,
}

impl<S: Subscriber> Layer<S> for TokioPerfettoLayer
where
    S: for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.inner.on_new_span(attrs, id, ctx)
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.inner.on_event(event, ctx)
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.inner.on_record(span, values, ctx)
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.inner.on_close(id, ctx)
    }

    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        self.inner.enabled(metadata, ctx)
    }
}

// TODO: better impl
type TokioPerfettoWriter = Mutex<File>;

pub struct TokioPerfettoLayerBuilder {
    file_name: Option<PathBuf>,
}

impl TokioPerfettoLayerBuilder {
    pub fn new() -> Self {
        Self { file_name: None }
    }

    pub fn file_name<P: Into<PathBuf>>(mut self, file_name: P) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    pub fn build(self) -> TokioPerfettoLayer {
        let inner = PerfettoLayer::new(Mutex::new(
            File::create(
                self.file_name
                    .unwrap_or_else(|| PathBuf::from("./trace.pftrace")),
            )
            .unwrap(),
        ))
        .with_filter_by_marker(|field_name| field_name == "tokio_runtime_event")
        .with_debug_annotations(true);

        TokioPerfettoLayer { inner }
    }
}

#[derive(Debug)]
pub struct TokioPerfettoLayerHandle {}

impl TokioPerfettoLayerHandle {
    pub fn start(&self) {
        use super::super::{INITIALIZE, RUNNING, SUSPENDED};
        use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        match get_trace_enable() {
            INITIALIZE => {
                let layer = TokioPerfettoLayerBuilder::new()
                    .file_name("./test.pftrace")
                    .build();

                tracing_subscriber::registry().with(layer).init();

                set_trace_enable(RUNNING);
            }
            RUNNING => {
                // already enabled
            }
            SUSPENDED => {
                set_trace_enable(super::super::RUNNING);
            }
            _ => panic!("Unexpected State"),
        }
    }

    pub fn stop(&self) {
        set_trace_enable(super::super::SUSPENDED)
    }
}
