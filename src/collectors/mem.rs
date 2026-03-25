use std::fs;
use std::collections::HashMap;
use super::{Collector,Snapshot};
use std::time::Instant;

pub struct MemCollector;

impl MemCollector {
    pub fn new() -> Self { MemCollector }

    fn read_meminfo() -> std::io::Result<HashMap<String,u64>> {
        let contents = fs::read_to_string("/proc/meminfo")?;
        let mut map = HashMap::new();
        for line in contents.lines() {
            let mut parts = line.split_whitespace();
            let key = match parts.next() {
                Some(k) => k.trim_end_matches(':').to_string(),
                None => continue,
            };
            let value: u64 = match parts.next() {
                Some(v) => v.parse().unwrap_or(0),
                None => continue,
            };
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl Collector for MemCollector {
    fn name(&self) -> &str {
        "mem"
    }

    fn collect(&mut self) -> Snapshot {
        let map = match Self::read_meminfo() {
            Ok(m) => m,
            Err(_) => return Snapshot {
                timestamp: Instant::now(),
                values: Vec::new(),
                label: "mem".to_string(),
            },
        };

        let get = |key: &str| -> f64 {
            map.get(key).copied().unwrap_or(0) as f64
        };

        let total     = get("MemTotal");
        let available = get("MemAvailable");
        let swap_total = get("SwapTotal");
        let swap_free  = get("SwapFree");

        let used      = total - available;
        let swap_used = swap_total - swap_free;


         // convert kB to GB for display
        // values layout: [used_gb, total_gb, swap_used_gb, swap_total_gb]
        const KB_TO_GB: f64 = 1_048_576.0; // 1024 * 1024

         Snapshot {
            timestamp: Instant::now(),
            values: vec![
                used       / KB_TO_GB,
                total      / KB_TO_GB,
                swap_used  / KB_TO_GB,
                swap_total / KB_TO_GB,
            ],
            label: "mem".to_string(),
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Collector;

    #[test]
    fn collect_returns_four_values() {
        let mut collector = MemCollector::new();
        let snapshot = collector.collect();
        // used, total, swap_used, swap_total
        assert_eq!(snapshot.values.len(), 4);
    }

    #[test]
    fn used_does_not_exceed_total() {
        let mut collector = MemCollector::new();
        let snapshot = collector.collect();
        // used (index 0) should never exceed total (index 1)
        assert!(snapshot.values[0] <= snapshot.values[1]);
    }

    #[test]
    fn total_memory_is_plausible() {
        let mut collector = MemCollector::new();
        let snapshot = collector.collect();
        // total should be > 0.5 GB and < 10TB — sanity check
        assert!(snapshot.values[1] > 0.5);
        assert!(snapshot.values[1] < 10_000.0);
    }

    #[test]
    fn swap_used_does_not_exceed_swap_total() {
        let mut collector = MemCollector::new();
        let snapshot = collector.collect();
        assert!(snapshot.values[2] <= snapshot.values[3]);
    }
}


