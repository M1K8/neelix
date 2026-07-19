use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::background::SPLIT_CHAR;
use crate::nostd_types::{CPU, EventType, FOOTER, HEADER, RAM};
use crate::types::HidEvent;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct PCState {}

pub struct PCStatMsg {
    pub cpu: u16,
    pub ram: u16,
}

impl HidEvent for PCStatMsg {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for b in self.cpu.to_le_bytes() {
            bytes.push(b);
        }
        bytes.push(SPLIT_CHAR as u8);
        for b in self.ram.to_le_bytes() {
            bytes.push(b);
        }
        bytes.push(SPLIT_CHAR as u8);

        bytes.resize(32, 0);

        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        let mut header = Vec::new();
        header.extend_from_slice(&HEADER);
        header.extend_from_slice(&[EventType::PCUpdate as u8]);
        v.push(header);

        v.push(self.to_bytes());
        let mut footer = Vec::new();
        footer.extend_from_slice(&FOOTER);
        v.push(footer);
        v
    }

    fn event_type(&self) -> crate::nostd_types::EventType {
        EventType::PCUpdate
    }
}
impl PCState {
    pub fn init() -> Self {
        PCState {}
    }
    pub async fn poll_pc_stats(
        &mut self,
        sys: Arc<Mutex<System>>,
        resp: mpsc::Sender<Arc<dyn HidEvent>>,
        shutting_down: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let mut system = sys.lock().await;

        system.refresh_all();
        tokio::time::sleep(Duration::from_millis(250)).await;
        let max_ram: u64 = system.total_memory();
        drop(system);

        loop {
            if shutting_down.load(Ordering::Relaxed) {
                return Ok(());
            }
            system = sys.lock().await;
            system.refresh_cpu_usage();
            system.refresh_memory();
            let cpu = system.global_cpu_usage();
            let ram = system.used_memory();
            let curr_cpu_pixel_height = (cpu / 100.0) * (CPU.y2 - CPU.y) as f32;
            let cpu = curr_cpu_pixel_height.round() as u16;

            let curr_ram_pixel_height = (ram as f32 / max_ram as f32) * (RAM.y2 - RAM.y) as f32;
            let ram = curr_ram_pixel_height.round() as u16;
            match resp.send(Arc::new(PCStatMsg { cpu, ram })).await {
                Ok(_) => {}
                Err(e) => return Err(e.to_string()),
            };

            drop(system);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
