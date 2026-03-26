use std::fs;
use super::{Collector, Snapshot};
use std::time::Instant;

pub struct CpuCollector {
    prev_idle: Vec<u64>,
    prev_total: Vec<u64>,
}

impl CpuCollector {
    pub fn new() -> Self {
        CpuCollector {
            prev_idle: Vec::new(),
            prev_total: Vec::new(),
        }
    }


    fn read_stat() -> std::io::Result<Vec<(u64,u64)>> {
        let contents = fs::read_to_string("/proc/stat")?;
        let mut results = Vec::new();

        for line in contents.lines() {
            if !line.starts_with("cpu") { break; }

            let mut fields = line.split_whitespace();
            fields.next();

            let nums: Vec<u64> = fields
                .take(7)
                .map(|f| f.parse().unwrap_or(0))
                .collect();

            if nums.len() < 5 { continue; }

            let idle = nums[3] + nums[4];
            let total: u64 = nums.iter().sum();

            results.push((idle, total));
        }

        Ok(results)
    }
}

impl Collector for CpuCollector {
    fn name(&self) -> &str {
        "cpu"
    }

    fn collect(&mut self) -> Snapshot {
        let readings = match Self::read_stat() {
            Ok(r) => r,
            Err(_) => return Snapshot {
                timestamp: Instant::now(),
                values: Vec::new(),
                label: "cpu".to_string(),
            },
        };

        if self.prev_idle.is_empty() {
            self.prev_idle = readings.iter().map(|(idle, _)| *idle).collect();
            self.prev_total = readings.iter().map(|(_, total)| *total).collect();

            return Snapshot {
                timestamp: Instant::now(),
                values: vec![0.0; readings.len()],
                label: "cpu".to_string(),
            };
        }
 let values: Vec<f64> = readings
            .iter()
            .enumerate()
            .map(|(i, (idle, total))| {
                let prev_idle = self.prev_idle.get(i).copied().unwrap_or(0);
                let prev_total = self.prev_total.get(i).copied().unwrap_or(0);

                let delta_idle = idle.saturating_sub(prev_idle);
                let delta_total = total.saturating_sub(prev_total);

                if delta_total == 0 {
                    0.0
                } else {
                    // usage = (total_ticks - idle_ticks) / total_ticks
                    let usage = (delta_total - delta_idle) as f64 
                                / delta_total as f64 
                                * 100.0;
                    // clamp to 0-100 — floating point can produce tiny negatives
                    usage.clamp(0.0, 100.0)
                }
            })
            .collect();

         self.prev_idle = readings.iter().map(|(idle, _)| *idle).collect();
        self.prev_total = readings.iter().map(|(_, total)| *total).collect();

        Snapshot {
            timestamp: Instant::now(),
            values,
            label: "cpu".to_string(),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Collector;

    #[test]
    fn first_collect_returns_zeros() {
        let mut collector = CpuCollector::new();
        let snapshot = collector.collect();

        // first call should return all zeros — no delta yet
        assert!(snapshot.values.iter().all(|&v| v == 0.0));
        // but should have entries — one per cpu line
        assert!(!snapshot.values.is_empty());
    }

    #[test]
    fn second_collect_returns_percentages() {
        let mut collector = CpuCollector::new();
        
        // first call establishes baseline
        collector.collect();
        
        // do some work so there's a measurable delta
        let mut x = 0u64;
        for i in 0..1_000_000 {
            x = x.wrapping_add(i);
        }
        
        // second call should return real percentages
        let snapshot = collector.collect();
        
        // all values should be valid percentages
        assert!(snapshot.values.iter().all(|&v| v >= 0.0 && v <= 100.0));
        // label should be correct
        assert_eq!(snapshot.label, "cpu");
        // should have 33 entries — 1 aggregate + 32 cores
        assert_eq!(snapshot.values.len(), 33);
    }
}
