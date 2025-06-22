#![cfg(all(
    tokio_unstable,
    feature = "runtime-tracing",
    target_os = "linux",
    target_arch = "x86_64"
))]

use rt_trace::{backend::perfetto::PerfettoReporter, config::Config};
use std::{io::Write, sync::Arc};

/// A handle to the flight recorder.
#[derive(Debug, Clone)]
pub struct Handle {
    pub(crate) flight_recorder: Arc<PerfettoFlightRecorder>,
}

impl Handle {
    /// Create a new flight recorder handle.
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self {
            flight_recorder: Arc::new(PerfettoFlightRecorder::new()),
        }
    }

    /// Initialize the flight recorder.
    pub fn initialize(&self) {
        self.flight_recorder.initialize();
    }

    /// Start the flight recorder.
    pub fn start(&self) {
        self.flight_recorder.start();
    }

    /// Stop the flight recorder.
    pub fn stop(&self) {
        self.flight_recorder.stop();
    }

    /// Flush the current trace to the specified writer.
    pub fn flush_trace<W: Write>(&self, writer: &mut W) {
        self.flight_recorder.flush_trace(writer);
    }
}

/// A trait that represents flight recorder behavior.
pub(crate) trait FlightRecorder {
    /// Initialize flight recorder
    fn initialize(&self);

    /// Start flight recorder
    fn start(&self);

    /// Stop flight recorder
    fn stop(&self);

    /// Flush current buffer content to the specific storage
    fn flush_trace<W: Write>(&self, writer: &mut W);
}

// perfetto impl

/// Flight recorder implementation powered by perfetto tracing library.
#[derive(Debug)]
pub(crate) struct PerfettoFlightRecorder {
    config: Config,
}

impl PerfettoFlightRecorder {
    /// Create a new instance of `PerfettoFlightRecorder`.
    pub(crate) fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl FlightRecorder for PerfettoFlightRecorder {
    fn initialize(&self) {
        let config = self.config.clone();
        let consumer = PerfettoReporter::new();
        rt_trace::initialize(config, consumer);
    }
    /// Start flight recorder
    fn start(&self) {
        rt_trace::start();
    }

    /// Stop flight recorder
    fn stop(&self) {
        rt_trace::stop();
    }

    /// Flush current buffer to the specific
    fn flush_trace<W: Write>(&self, writer: &mut W) {
        rt_trace::flush(writer);
    }
}

// fastrace impl (TODO)
