use std::time::{Duration, Instant};

use crate::ring_buffer::RingBuffer;
use crate::collectors::{Collector, Snapshot};
use crate::collectors::cpu::CpuCollector;
use crate::collectors::mem::MemCollector;
use crate::collectors::disk::DiskCollector;

const HISTORY_SIZE: usize = 60;

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Summary,
    Cpu,
    Processes,
}

impl Page {
    pub fn next(&self) -> Page {
        match self {
            Page::Summary   => Page::Cpu,
            Page::Cpu       => Page::Processes,
            Page::Processes => Page::Summary,
        }
    }

    pub fn prev(&self) -> Page {
        match self {
            Page::Summary   => Page::Processes,
            Page::Cpu       => Page::Summary,
            Page::Processes => Page::Cpu,
        }
    }
}

pub struct AppState {
    cpu_collector: CpuCollector,
    mem_collector: MemCollector,
    disk_collector: DiskCollector,

    pub cpu_history: Vec<RingBuffer<f64>>,
    pub mem_history: RingBuffer<f64>,
    pub disk_history: Vec<RingBuffer<f64>>,

    pub last_cpu: Snapshot,
    pub last_mem: Snapshot,
    pub last_disk: Snapshot,

    pub page: Page,
    pub paused: bool,

    last_collect: Instant,
    pub collect_interval: Duration,
}

impl AppState {
    pub fn new() -> Self {
        let now = Instant::now();

        let empty_cpu = Snapshot {
            timestamp: now,
            values: Vec::new(),
            label: "cpu".to_string()
        };
        let empty_mem = Snapshot {
            timestamp: now,
            values: Vec::new(),
            label: "mem".to_string()
        };
        let empty_disk = Snapshot {
            timestamp: now,
            values: Vec::new(),
            label: "disk".to_string()
        };

        AppState { 
            cpu_collector: CpuCollector::new(), 
            mem_collector: MemCollector::new(), 
            disk_collector: DiskCollector::new(), 
            
            cpu_history: Vec::new(), 
            mem_history: RingBuffer::new(HISTORY_SIZE), 
            disk_history: Vec::new(), 
            
            last_cpu: empty_cpu, 
            last_mem: empty_mem, 
            last_disk: empty_disk, 
            
            page: Page::Summary, 
            paused: false, 
            
            last_collect: now, 
            collect_interval: Duration::from_secs(1),
        }
    }


    pub fn tick(&mut self) {
        if self.paused { return; }

        if self.last_collect.elapsed() < self.collect_interval { return; }
        self.last_collect = Instant::now();

        //--CPU--
        let cpu_snap = self.cpu_collector.collect();

        if self.cpu_history.is_empty() && !cpu_snap.values.is_empty() {
            for _ in 0..cpu_snap.values.len() {
                self.cpu_history.push(RingBuffer::new(HISTORY_SIZE));
            }
        }

        for (i, &val) in cpu_snap.values.iter().enumerate() {
            if let Some(buf) = self.cpu_history.get_mut(i) { buf.push(val); }
        }
        self.last_cpu = cpu_snap;

        //--Memory--
        let mem_snap = self.mem_collector.collect();
         if let Some(&used) = mem_snap.values.first() { self.mem_history.push(used); }
        self.last_mem = mem_snap;

        //--Disk--
        let disk_snap = self.disk_collector.collect();

        if self.disk_history.is_empty() && !disk_snap.values.is_empty() {
            for _ in 0..disk_snap.values.len() { self.disk_history.push(RingBuffer::new(HISTORY_SIZE)); }
        }

        for (i, &val) in disk_snap.values.iter().enumerate() {
            if let Some(buf) = self.disk_history.get_mut(i) { buf.push(val); }
        }
        self.last_disk = disk_snap;
    }

    pub fn next_page(&mut self) { self.page = self.page.next(); }

    pub fn prev_page(&mut self) { self.page = self.page.prev(); }

    pub fn toggle_pause(&mut self) { self.paused = !self.paused; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_appstate_starts_on_summary_page() {
        let app = AppState::new();
        assert_eq!(app.page, Page::Summary);
    }

    #[test]
    fn new_appstate_starts_unpaused() {
        let app = AppState::new();
        assert!(!app.paused);
    }

    #[test]
    fn toggle_pause_flips_state() {
        let mut app = AppState::new();
        app.toggle_pause();
        assert!(app.paused);
        app.toggle_pause();
        assert!(!app.paused);
    }

    #[test]
    fn page_navigation_cycles_forward() {
        let mut app = AppState::new();
        app.next_page();
        assert_eq!(app.page, Page::Cpu);
        app.next_page();
        assert_eq!(app.page, Page::Processes);
        app.next_page();
        assert_eq!(app.page, Page::Summary); // wraps back
    }

    #[test]
    fn page_navigation_cycles_backward() {
        let mut app = AppState::new();
        app.prev_page();
        assert_eq!(app.page, Page::Processes); // wraps to end
        app.prev_page();
        assert_eq!(app.page, Page::Cpu);
        app.prev_page();
        assert_eq!(app.page, Page::Summary);
    }

    #[test]
    fn tick_populates_cpu_history() {
        let mut app = AppState::new();
        
        // first tick establishes baseline — cpu returns zeros
        app.tick();
        
        // small delay so second tick has a real delta
        std::thread::sleep(std::time::Duration::from_millis(1100));
        app.tick();

        // after two ticks cpu_history should be initialised
        assert!(!app.cpu_history.is_empty());
    }

    #[test]
    fn tick_does_nothing_when_paused() {
        let mut app = AppState::new();
        app.toggle_pause();
        app.tick();

        // paused — cpu_history should still be empty
        assert!(app.cpu_history.is_empty());
    }

    #[test]
    fn tick_respects_collect_interval() {
        let mut app = AppState::new();
        
        // first tick
        app.tick();
        let len_after_first = app.cpu_history.len();

        // immediate second tick — interval hasn't elapsed
        app.tick();

        // cpu_history length shouldn't change — no collection happened
        assert_eq!(app.cpu_history.len(), len_after_first);
    }
}