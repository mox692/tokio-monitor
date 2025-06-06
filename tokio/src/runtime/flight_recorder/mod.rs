use rt_trace::{backend::perfetto::PerfettoReporter, config::Config};

/// A trait that represents flight recorder behavior.
pub trait FlightRecorder {
    /// Initialize flight recorder
    fn initialize(&mut self);

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
    config: Config,
}

impl PerfettoFlightRecorder {
    /// Create a new instance of `PerfettoFlightRecorder`.
    pub fn new(_file_name: &str) -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl FlightRecorder for PerfettoFlightRecorder {
    fn initialize(&mut self) {
        let config = std::mem::take(&mut self.config);
        let consumer = PerfettoReporter::new("./test.pftrace");
        rt_trace::initialize(config, consumer);
    }
    /// Start flight recorder
    fn start(&mut self) {
        rt_trace::start();
    }

    /// Stop flight recorder
    fn stop(&mut self) {
        rt_trace::stop();
    }

    /// Flush current buffer to the specific
    fn flush_trace(&mut self) {
        rt_trace::flush();
    }
}

// fastrace impl (TODO)
