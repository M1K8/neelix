use crate::background::qgf_art;
use crate::nostd_types::FOOTER;
use crate::nostd_types::HEADER;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::nostd_types::SPLIT_CHAR;
use serde::{Deserialize, Serialize};
use sysinfo::ProcessesToUpdate;
use sysinfo::System;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, sleep};

use crate::nostd_types::EventType;
use crate::types::HidEvent;
#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub name: String,
    pub pid: u16,
    pub is_running: bool,
    pub metadata: Option<HashMap<String, String>>,
    /// QGF-encoded process icon (from `icons/<stem>.ico`) built via the
    /// qmk-qgf crate; not transmitted yet.
    #[serde(skip)]
    pub icon_qgf: Option<Vec<u8>>,
}

impl HidEvent for Process {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.name.as_bytes());
        bytes.extend_from_slice(&[SPLIT_CHAR]);
        bytes.extend_from_slice(&self.pid.to_le_bytes());
        bytes.extend_from_slice(&[SPLIT_CHAR]);
        bytes.extend_from_slice(&(self.is_running as u8).to_le_bytes());
        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        let mut header_chunk = Vec::new();
        header_chunk.extend_from_slice(&HEADER); // Header
        header_chunk.extend_from_slice(&[EventType::ProcessStateUpdate as u8]); // Header
        v.push(header_chunk);
        let mut chunk = self.to_bytes();
        if chunk.len() < 32 {
            chunk.resize(32, 0);
        }
        v.push(chunk);
        let mut footer_chunk = Vec::new();
        footer_chunk.extend_from_slice(&FOOTER);
        v.push(footer_chunk);
        v
    }

    fn event_type(&self) -> crate::nostd_types::EventType {
        EventType::ProcessStateUpdate
    }
}

pub struct ProcessWatcher {
    active_processes: Arc<Mutex<HashSet<String>>>,
}

impl ProcessWatcher {
    pub fn new() -> Self {
        ProcessWatcher {
            active_processes: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    pub async fn is_active(&self, process: &str) -> bool {
        return self.active_processes.lock().await.contains(process);
    }
    pub fn watch(
        &self,
        sys: Arc<Mutex<System>>,
        expected_processes: Vec<String>,
        chan: mpsc::Sender<Arc<dyn HidEvent>>,
        shutting_down: Arc<AtomicBool>,
    ) {
        let mut seen_this_cycle = HashSet::new();
        let mut is_still_alive: HashSet<String> = HashSet::new();
        let active_processes = self.active_processes.clone();
        tokio::spawn(async move {
            loop {
                if shutting_down.load(Ordering::Relaxed) {
                    break;
                }

                let mut system = sys.lock().await;
                system.refresh_processes(ProcessesToUpdate::All, true);
                let processes = system.processes();
                for p in processes.values() {
                    // Skip processes with non-UTF-8 names rather than panicking
                    let Some(name) = p.name().to_str() else {
                        continue;
                    };
                    let this_name = name.to_owned();
                    if expected_processes.contains(&this_name) {
                        is_still_alive.insert(this_name.clone());
                        if !seen_this_cycle.contains(&this_name) {
                            let index = expected_processes
                                .iter()
                                .position(|n| *n == this_name)
                                .unwrap();
                            let mut lo_prio = false;
                            for proc in &seen_this_cycle {
                                if expected_processes.iter().position(|n| n == proc)
                                    < expected_processes.iter().position(|n| *n == this_name)
                                {
                                    lo_prio = true;
                                    break;
                                }
                            }
                            active_processes.lock().await.insert(this_name.clone());
                            if !lo_prio {
                                seen_this_cycle.insert(this_name.clone());

                                let icon_qgf = qgf_art::process_icon_qgf(&this_name);
                                if chan
                                    .send(Arc::new(Process {
                                        name: this_name,
                                        pid: index as u16,
                                        is_running: true,
                                        metadata: None,
                                        icon_qgf,
                                    }))
                                    .await
                                    .is_err()
                                {
                                    // Receiver gone — we're shutting down
                                    return;
                                }
                            }
                        }
                    }
                }

                let seen_this_cycle_clone = seen_this_cycle.clone();
                for p in seen_this_cycle_clone {
                    if !processes
                        .values()
                        .any(|proc| proc.name().to_str() == Some(p.as_str()))
                        && is_still_alive.contains(&p)
                    {
                        let index = expected_processes.iter().position(|n| n == &p).unwrap();
                        seen_this_cycle.remove(&p);
                        is_still_alive.remove(&p);
                        let pc = p.clone();
                        if chan
                            .send(Arc::new(Process {
                                name: p,
                                pid: index as u16,
                                is_running: false,
                                metadata: None,
                                icon_qgf: None,
                            }))
                            .await
                            .is_err()
                        {
                            // Receiver gone — we're shutting down
                            return;
                        }

                        active_processes.lock().await.remove(&pc);
                        tokio::time::sleep(Duration::from_millis(250)).await; // leave some time
                    }
                }
                drop(system);
                if shutting_down.load(Ordering::Relaxed) {
                    break;
                }
                sleep(Duration::from_secs(12)).await;
            }
        });
    }
}
