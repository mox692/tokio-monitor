/// docs
#[derive(Debug)]
pub struct FlightRecorderConfig {}

/// docs
#[derive(Debug)]
pub struct FlightRecorderHandle {}

impl FlightRecorderHandle {
    /// start flight recorder
    pub fn start(&mut self) {}

    /// stop flihgt recorder
    pub fn stop(&mut self) {}

    /// flush current buffer to the specific
    pub fn flush_trace(&mut self) {
        // using writer
    }
}
