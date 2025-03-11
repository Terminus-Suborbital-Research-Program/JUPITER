mod telemetry;

pub trait Task {
    type Context;

    /// Run the task, either to completion, or divergin
    fn task(&mut self, context: &mut Self::Context);
}

pub mod tasks {
    use super::*;

    pub use telemetry::TelemetryLogger;
}