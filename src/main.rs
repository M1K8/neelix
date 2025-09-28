use std::{collections::HashSet, fs};

extern crate hidapi;
mod device;
mod process_watcher;
// fn main() {
//     let api = hidapi::HidApi::new().unwrap();
//     let devices = api.device_list().filter( |d| d.vendor_id() == 0xfaf0 && d.product_id() == 0xfaf0 );
//     let mut indx: u8 = 0;
//     for device in devices {
//         let path = device.path();
//         println!("{:#x} {:#x} {}", device.product_id(), device.vendor_id(), path.to_str().unwrap());
//         let device = api.open_path(path).unwrap();
//         if let Err(err) = device.write(&['a' as u8, 'b' as u8, 'c' as u8, 'd' as u8, 'e' as u8, 'f' as u8, 'g' as u8, 'h' as u8]) {
//             eprintln!("Error for device {}: {}", indx,  err);
//                 indx +=1;
//             continue;
//         }

//     }
// }


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config_contents = fs::read_to_string("config.toml")?;
    let config: toml::Value = toml::from_str(&config_contents)?;

    println!("Config: {:?}", config);


    let expected_processes = if let Some(processes) = config.get("recognised_processes") {
        processes.as_array().unwrap().iter().map(|p| p.as_str().unwrap().to_string()).collect()
    } else {
        println!("non");
        HashSet::new()
    };

    let (send, recv) = tokio::sync::mpsc::channel(100);


    if expected_processes.is_empty() {
        eprintln!("No recognised_processes found in config.toml");
    } else {
        if let Err(err) = tokio::spawn(process_watcher::process_watcher(expected_processes, send)).await {
            eprintln!("Error spawning process watcher: {}",  err);
        }
    }

    Ok(())

}

// for known vid / pid pairs, either config known usage page / usage, or try all