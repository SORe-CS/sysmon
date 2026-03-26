use std::fs;
use std::collections::HashMap;
use std::time::Instant;
use super::{Collector, Snapshot};

const SECTOR_SIZE: f64 = 512.00;

pub struct DiskCollector {
    prev_sectors: HashMap<String, (u64, u64)>,
    last_time: Option<Instant>,
}

impl DiskCollector {
    pub fn new() -> Self {
        DiskCollector { 
            prev_sectors: HashMap::new(),
            last_time: None,
        }
    }

    fn read_diskstats() -> std::io::Result<HashMap<String, (u64, u64)>> {
        let contents = fs::read_to_string("/proc/diskstats")?;
        let mut map = HashMap::new();

        for line in contents.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 10 { continue; }

            let name = fields[2];

            if name.starts_with("ram") || name.starts_with("loop") { continue; }

            let sectors_read: u64 = fields[5].parse().unwrap_or(0);
            let sectors_written: u64 = fields[9].parse().unwrap_or(0);

            map.insert(name.to_string(), (sectors_read, sectors_written));
        }
        Ok(map)
    }
}

impl Collector for DiskCollector {
    fn name(&self) -> &str {
        "disk"
    }

    fn collect(&mut self) -> Snapshot {
        let now = Instant::now();
        
        let current = match Self::read_diskstats() {
            Ok(m) => m,
            Err(_) => return Snapshot {
                timestamp: now,
                values: Vec::new(),
                label: "disk".to_string(),
            },
        };

        // first call — establish baseline, return zeros
        if self.prev_sectors.is_empty() {
            self.prev_sectors = current;
            self.last_time = Some(now);
            return Snapshot {
                timestamp: now,
                values: Vec::new(),
                label: "disk".to_string(),
            };
        }

        let elapsed_secs = match self.last_time {
            Some(t) => {
                let d = now.duration_since(t).as_secs_f64();
                if d < 0.001 { 0.001 } else { d }
            },
            None => 1.0,
        };

        let mut values = Vec::new();
        let mut labels = Vec::new();
        let mut devices: Vec<&String> = current.keys().collect();
        devices.sort();

        for device in devices {
            let (cur_read, cur_write) = current[device];
            let (prev_read, prev_write) = self.prev_sectors
                .get(device)
                .copied()
                .unwrap_or((cur_read, cur_write));

            let read_sectors  = cur_read.saturating_sub(prev_read);
            let write_sectors = cur_write.saturating_sub(prev_write);

            let read_bps  = (read_sectors  as f64 * SECTOR_SIZE) / elapsed_secs;
            let write_bps = (write_sectors as f64 * SECTOR_SIZE) / elapsed_secs;

            values.push(read_bps);
            values.push(write_bps);
            labels.push(device.clone());
        }

        self.prev_sectors = current;
        self.last_time = Some(now);

        Snapshot {
            timestamp: now,
            values,
            label:labels.join(","),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Collector;

    #[test]
    fn first_collect_returns_empty_values() {
        let mut collector = DiskCollector::new();
        let snapshot = collector.collect();
        // first call returns empty — no delta yet
        assert!(snapshot.values.is_empty());
    }

    #[test]
    fn second_collect_returns_even_number_of_values() {
        let mut collector = DiskCollector::new();
        collector.collect(); // baseline
        
        // small delay so elapsed time is nonzero
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let snapshot = collector.collect();
        // values come in pairs — read_bps and write_bps per device
        assert_eq!(snapshot.values.len() % 2, 0);
    }

    #[test]
    fn bytes_per_second_are_non_negative() {
        let mut collector = DiskCollector::new();
        collector.collect();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let snapshot = collector.collect();
        assert!(snapshot.values.iter().all(|&v| v >= 0.0));
    }
}