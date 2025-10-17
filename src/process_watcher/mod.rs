use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sysinfo::ProcessesToUpdate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub name: String,
    pub pid: i32,
    pub is_running: bool,
}

pub async fn process_watcher(
    expected_processes: HashSet<String>,
    chan: tokio::sync::mpsc::Sender<Process>,
) {
    use sysinfo::System;
    use tokio::time::{Duration, sleep};

    let mut system = System::new_all();
    let mut seen_this_cycle = HashSet::new();
    let mut is_still_alive: HashSet<String> = HashSet::new();

    loop {
        system.refresh_processes(ProcessesToUpdate::All, true);
        let processes = system.processes();
        for p in processes.values() {
            let name = p.name().to_str().unwrap();
            if expected_processes.contains(name) {
                let this_name = name.to_owned();
                is_still_alive.insert(this_name);
                if !seen_this_cycle.contains(name) {
                    let name = name.to_owned();
                    seen_this_cycle.insert(name);
                    println!("Expected process detected: {}", p.name().to_str().unwrap());
                }
            }
        }

        let seen_this_cycle_clone = seen_this_cycle.clone();
        for p in seen_this_cycle_clone {
            if !processes
                .values()
                .any(|proc| proc.name().to_str().unwrap() == p)
                && is_still_alive.contains(&p)
            {
                println!("Process exited: {}", p);
                seen_this_cycle.remove(&p);
                is_still_alive.remove(&p);
            }
        }

        sleep(Duration::from_secs(2)).await; // Check every 2 seconds
    }
}
