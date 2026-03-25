pub mod cpu;
pub mod mem;
pub mod disk;

use std::time::Instant;

pub struct Snapshot {
    pub timestamp: Instant,
    pub values: Vec<f64>,
    pub label: String,
}

pub trait Collector {
    fn collect(&mut self) -> Snapshot;
    fn name(&self) -> &str;
}