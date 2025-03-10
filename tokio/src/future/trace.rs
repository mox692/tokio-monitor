use std::future::Future;

#[allow(dead_code)]
pub(crate) trait InstrumentedFuture: Future {
    fn id(&self) -> Option<tracing::Id>;
}

impl<F: Future> InstrumentedFuture for tracing::instrument::Instrumented<F> {
    fn id(&self) -> Option<tracing::Id> {
        self.span().id()
    }
}
