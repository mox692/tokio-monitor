use tracing_perfetto::external::tokio::TokioPerfettoLayerHandle;

/// A trait that represents flight recorder behavior.
pub trait FlightRecorder {
    /// Start flight recorder
    fn start(&mut self);

    /// Stop flight recorder
    fn stop(&mut self);

    /// Flush current buffer content to the specific storage
    fn flush_trace(&mut self);
}

// perfetto impl

/// Flight recorder implementation powered by perfetto tracing library.
#[derive(Debug)]
pub struct PerfettoFlightRecorder {
    pub(crate) inner: TokioPerfettoLayerHandle,
}

impl FlightRecorder for PerfettoFlightRecorder {
    /// Start flight recorder
    fn start(&mut self) {
        self.inner.start()
    }

    /// Stop flight recorder
    fn stop(&mut self) {
        self.inner.stop()
    }

    /// Flush current buffer to the specific
    fn flush_trace(&mut self) {}
}

// fastrace impl (TODO)
