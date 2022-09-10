use std::{
    fs::OpenOptions,
    io::Write,
    thread,
    time::{Duration, SystemTime},
};

use dev_parser::Device;

pub mod dev_parser;

const BOUND_IP_ADDR: &'static str = "10.1.3.3:0";
const CAPACITY: u64 = 100 * 10_u64.pow(6);

#[derive(PartialEq)]
enum CapacityKind {
    NinetyPercent,
    EightyPercent,
    FiftyPercent,
    BelowFiftyPercent,
}

fn main() {
    if cfg!(target_os = "linux") != true {
        panic!("This program only works on Linux");
    }

    let mut dump_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("dump.tsv")
        .unwrap();

    dump_file
        .write_all(b"#tsv\ttime\ttxpkts\ttxbytes\trxpkts\trxbytes\n")
        .expect("Failed to write header");

    let mut events_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("#tsv\ttime\tevent")
        .unwrap();

    events_file
        .write_all(b"timestamp\tcapacity\tkind")
        .expect("The events header failed to write");

    // Usage of external nix crate
    let interfaces = nix::ifaddrs::getifaddrs().unwrap();
    let mut interface_name = String::new();
    for interface in interfaces {
        match interface.address {
            Some(address) => {
                if address.to_string() == BOUND_IP_ADDR {
                    interface_name = interface.interface_name;
                }
            }
            None => {}
        }
    }

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
                dump_file
                    .write_all(
                        format!(
                            "{}\t{}\t{}\t{}\t{}\n",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            device.transmit_packets - old_stats.transmit_packets,
                            transmit_bytes,
                            device.receive_packets - old_stats.receive_packets,
                            receive_bytes
                        )
                        .as_bytes(),
                    )
                    .expect("Failed to write data");
                let transmit_capacity =
                    ((transmit_bytes as f64 * 8_f64) / (CAPACITY as f64)) * 100_f64;
                let receive_capacity =
                    ((receive_bytes as f64 * 8_f64) / (CAPACITY as f64)) * 100_f64;

                if transmit_capacity >= 90.0 || receive_capacity >= 90.0 {
                    if capacity_kind != CapacityKind::NinetyPercent {
                        println!(
                            ">= 90% capacity {}",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64()
                        );
                        events_file
                            .write_all(
                                format!(
                                    "{}\t>= 90%\t",
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs_f64(),
                                )
                                .as_bytes(),
                            )
                            .expect("Failed to write data");
                        capacity_kind = CapacityKind::NinetyPercent;
                    }
                } else if transmit_capacity >= 80.0 || receive_capacity >= 80.0 {
                    if capacity_kind != CapacityKind::EightyPercent {
                        println!(
                            ">= 80% capacity {}",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64()
                        );
                        events_file
                            .write_all(
                                format!(
                                    "{}\t>= 80%\t",
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs_f64(),
                                )
                                .as_bytes(),
                            )
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
                            .write_all(
                                format!(
                                    "{}\t>= 50%\t",
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs_f64(),
                                )
                                .as_bytes(),
                            )
                            .expect("Failed to write data");
                        capacity_kind = CapacityKind::FiftyPercent;
                    }
                } else if capacity_kind != CapacityKind::BelowFiftyPercent {
                    capacity_kind = CapacityKind::BelowFiftyPercent;
                    println!(
                        "< 50% capacity {}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64()
                    );
                    events_file
                        .write_all(
                            format!(
                                "{}\t< 50%\t",
                                SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs_f64(),
                            )
                            .as_bytes(),
                        )
                        .expect("Failed to write data");
                }
                old_stats = device;
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
