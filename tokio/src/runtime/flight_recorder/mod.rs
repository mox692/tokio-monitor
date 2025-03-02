use tracing_perfetto::external::tokio::TokioPerfettoLayerHandle;

/// docs
#[derive(Debug)]
pub struct FlightRecorderConfig {}

/// docs
pub trait FlightRecorder {
    /// start flight recorder
    fn start(&mut self);

    /// stop flight recorder
    fn stop(&mut self);

    /// flush current buffer content to the specific storage
    fn flush_trace(&mut self);
}

// perfetto impl

/// docs
#[derive(Debug)]
pub struct PerfettoFlightRecorder {
    pub(crate) inner: TokioPerfettoLayerHandle,
}

impl FlightRecorder for PerfettoFlightRecorder {
    /// start flight recorder
    fn start(&mut self) {
        self.inner.start()
    }

    /// stop flight recorder
    fn stop(&mut self) {
        self.inner.stop()
    }

    /// flush current buffer to the specific
    fn flush_trace(&mut self) {
        // using writer
    }
}

// fastrace impl

#[derive(Debug)]
pub(crate) struct FastraceFlightRecorder {}

impl FlightRecorder for FastraceFlightRecorder {
    /// start flight recorder
    fn start(&mut self) {}

    /// stop flight recorder
    fn stop(&mut self) {}

    /// flush current buffer to the specific
    fn flush_trace(&mut self) {}
}
