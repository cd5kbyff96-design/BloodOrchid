use crate::errors::BoundaryError;

pub trait MetricsSink: Send + Sync {
    fn increment_counter(&self, name: &'static str);
}

pub trait EventLogger: Send + Sync {
    fn on_accept(&self, message_id: &str, family: &str, name: &str);
    fn on_reject(&self, message_id: &str, error: &BoundaryError);
}

#[derive(Default)]
pub struct NoopMetrics;

impl MetricsSink for NoopMetrics {
    fn increment_counter(&self, _name: &'static str) {}
}

#[derive(Default)]
pub struct NoopLogger;

impl EventLogger for NoopLogger {
    fn on_accept(&self, _message_id: &str, _family: &str, _name: &str) {}
    fn on_reject(&self, _message_id: &str, _error: &BoundaryError) {}
}
