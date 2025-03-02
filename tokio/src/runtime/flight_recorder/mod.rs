use tracing_perfetto::external::tokio::TokioPerfettoLayerHandle;

/// docs
#[derive(Debug)]
pub struct FlightRecorderConfig {}

/// docs
#[derive(Debug)]
pub struct FlightRecorderHandle {
    pub(crate) inner: TokioPerfettoLayerHandle,
}

impl FlightRecorderHandle {
    /// start flight recorder
    pub fn start(&mut self) {
        self.inner.start()
    }

    /// stop flight recorder
    pub fn stop(&mut self) {
        self.inner.stop()
    }

    /// flush current buffer to the specific
    pub fn flush_trace(&mut self) {
        // using writer
    }
}
