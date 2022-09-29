use clap::Parser;
use dev_parser::Device;
use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

pub mod dev_parser;

const BOUND_IP_ADDR: &'static str = "10.1.3.3:0";

#[derive(PartialEq)]
enum CapacityKind {
    NinetyPercent,
    EightyPercent,
    FiftyPercent,
    BelowFiftyPercent,
}

/// Application that monitors packets by reading /proc/net/dev
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The interface to monitor
    interface: String,
    /// The directory to write the log file to
    #[arg(short, long, value_name = "DIRECTORY")]
    output_directory: Option<PathBuf>,
    /// The capacity of the link. Defaults to 50mbps
    #[arg(short, long)]
    capacity: Option<u64>,
}

fn main() {
    if cfg!(target_os = "linux") != true {
        panic!("This program only works on Linux");
    }

    let args = Cli::parse();

    let working_dir = args
        .output_directory
        .unwrap_or(std::env::current_dir().unwrap());

    let interface_name = args.interface;
    let capacity: f64 = args.capacity.unwrap_or(50) as f64 * 10_u64.pow(6) as f64;

    println!("Working directory: {:?}", working_dir);
    println!("Listening on interface: {}", interface_name);

    let mut dump_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(working_dir.join("dump.tsv"))
        .unwrap();

    dump_file
        .write_all(b"#tsv\ttime\ttxpkts\ttxbytes\trxpkts\trxbytes\n")
        .expect("Failed to write header");

    let mut events_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(working_dir.join("events.tsv"))
        .unwrap();

    events_file
        .write_all(b"#tsv\ttime\tevent\n")
        .expect("The events header failed to write");

    if interface_name.is_empty() {
        panic!("Could not find interface with IP address {}", BOUND_IP_ADDR);
    }

    let mut old_stats = Device::new();
    let devices = dev_parser::get();
    for device in devices {
        if device.interface == interface_name {
            old_stats = device;
        }
    }

    if old_stats.interface.is_empty() {
        panic!("Could not find interface {}", interface_name);
    }

    let mut capacity_kind = CapacityKind::BelowFiftyPercent;
    loop {
        let devices = dev_parser::get();
        for device in devices {
            if device.interface == interface_name {
                let transmit_bytes = device.transmit_bytes - old_stats.transmit_bytes;
                let receive_bytes = device.receive_bytes - old_stats.receive_bytes;
                let time_now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                dump_file
                    .write_all(
                        format!(
                            "{}\t{}\t{}\t{}\t{}\n",
                            time_now,
                            device.transmit_packets - old_stats.transmit_packets,
                            transmit_bytes,
                            device.receive_packets - old_stats.receive_packets,
                            receive_bytes
                        )
                        .as_bytes(),
                    )
                    .expect("Failed to write data");
                let transmit_capacity = ((transmit_bytes as f64 * 8_f64) / capacity) * 100_f64;
                let receive_capacity = ((receive_bytes as f64 * 8_f64) / capacity) * 100_f64;

                if transmit_capacity >= 90.0 || receive_capacity >= 90.0 {
                    if capacity_kind != CapacityKind::NinetyPercent {
                        println!(">= 90% capacity {}", time_now);
                        events_file
                            .write_all(format!("{}\t>=90%\n", time_now).as_bytes())
                            .expect("Failed to write data");
                        capacity_kind = CapacityKind::NinetyPercent;
                    }
                } else if transmit_capacity >= 80.0 || receive_capacity >= 80.0 {
                    if capacity_kind != CapacityKind::EightyPercent {
                        println!(">= 80% capacity {}", time_now);
                        events_file
                            .write_all(format!("{}\t>=80%\n", time_now).as_bytes())
                            .expect("Failed to write data");
                        capacity_kind = CapacityKind::EightyPercent;
                    }
                } else if transmit_capacity >= 50.0 || receive_capacity >= 50.0 {
                    if capacity_kind != CapacityKind::FiftyPercent {
                        println!(
                            ">= 50% capacity {}",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64()
                        );
                        events_file
                            .write_all(format!("{}\t>=50%\n", time_now).as_bytes())
                            .expect("Failed to write data");
                        capacity_kind = CapacityKind::FiftyPercent;
                    }
                } else if capacity_kind != CapacityKind::BelowFiftyPercent {
                    capacity_kind = CapacityKind::BelowFiftyPercent;
                    println!("< 50% capacity {}", time_now);
                    events_file
                        .write_all(format!("{}\t<50%\n", time_now).as_bytes())
                        .expect("Failed to write data");
                }
                old_stats = device;
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
