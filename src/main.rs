#[macro_use]
use std::io::{self, Write};
use std::cmp::Ordering;
use std::thread;
use std::time;

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use sysinfo::{DiskExt, NetworkExt, ProcessExt, ProcessorExt, System, SystemExt};

fn main() {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    io::stdout().write_all(b"{ \"version\": 1 }[").unwrap();
    let handle = thread::spawn(move || loop {
        let mut status_lines: Vec<StatusLine> = vec![];
        status_lines.push(os_info(&system));
        status_lines.push(cpu_usage(&mut system));
        status_lines.push(memory_usage(&mut system));
        status_lines.append(&mut storage_info(&mut system));
        status_lines.push(network_usage(&mut system));
        status_lines.push(time());
        println!("{},", serde_json::to_string(&status_lines).unwrap());

        let waiting_time = time::Duration::from_secs(2);
        spin_sleep::sleep(waiting_time);
    });

    handle.join().unwrap();
}

fn os_info(sys: &sysinfo::System) -> StatusLine {
    match sys.get_long_os_version() {
        Some(os) => StatusLine {
            full_text: os,
            color: Color::RED.to_string(),
            min_width: None,
            align: None,
        },
        None => StatusLine {
            full_text: "error".to_string(),
            color: Color::RED.to_string(),
            min_width: None,
            align: None,
        },
    }
}

fn cpu_usage(sys: &mut sysinfo::System) -> StatusLine {
    sys.refresh_cpu();
    sys.refresh_cpu();
    let load = sys.get_global_processor_info().get_cpu_usage();
    StatusLine {
        full_text: format!(" : {:>5.1} %", load),
        color: Color::GREEN.to_string(),
        min_width: None,
        align: None,
    }
}

fn memory_usage(sys: &mut sysinfo::System) -> StatusLine {
    sys.refresh_memory();
    let usage = sys.get_used_memory();
    let total = sys.get_total_memory();

    StatusLine {
        full_text: format!(
            " : {:.1}G / {:.1}G",
            usage as f64 / 1000000.0,
            total as f64 / 1000000.0
        ),
        color: Color::YELLOW.to_string(),
        min_width: None,
        align: None,
    }
}

fn storage_info(sys: &mut sysinfo::System) -> Vec<StatusLine> {
    sys.refresh_disks_list();
    let disks = sys.get_disks();
    let mut disk_infos = vec![];

    for disk in disks {
        if disk.get_name().to_str().unwrap() != "/dev/sda2" {
            continue;
        }
        disk_infos.push(StatusLine {
            full_text: format!(
                " : {:.1}G / {:.1}G",
                (disk.get_total_space() - disk.get_available_space()) as f64 / 1000000000.0,
                disk.get_total_space() as f64 / 1000000000.0
            ),
            color: Color::BLUE.to_string(),
            min_width: None,
            align: None,
        });
    }
    disk_infos
}

fn network_usage(sys: &mut sysinfo::System) -> StatusLine {
    sys.refresh_networks();
    let networks = sys.get_networks();
    let mut rx = 0;
    let mut tx = 0;
    for (_, data) in networks {
        rx = rx + data.get_received();
        tx = tx + data.get_transmitted();
    }

    StatusLine {
        full_text: format!(
            " : {:.1}M |  : {:.1}M",
            (rx as f64) / 1000000.,
            (tx as f64) / 1000000.
        ),
        color: Color::MAGENTA.to_string(),
        min_width: None,
        align: None,
    }
}

fn time() -> StatusLine {
    let now = chrono::Local::now();

    StatusLine {
        full_text: format!(
            "{:02}/{:02}/{} {:02}:{:02} {} ",
            now.day(),
            now.month(),
            now.year(),
            now.hour12().1,
            now.minute(),
            match now.hour12().0 {
                true => "PM",
                false => "AM",
            },
        ),
        color: Color::CYAN.to_string(),
        min_width: None,
        align: Some(Align::Right),
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct StatusLine {
    full_text: String,
    color: String,
    min_width: Option<u16>,
    align: Option<Align>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Align {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
}

struct Color;

impl Color {
    const RED: &'static str = "#ea6962";
    const GREEN: &'static str = "#a9b665";
    const YELLOW: &'static str = "#d8a657";
    const BLUE: &'static str = "#7daea3";
    const MAGENTA: &'static str = "#d3869b";
    const CYAN: &'static str = "#89b482";
}
