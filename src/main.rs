use clap::{Parser, ValueEnum};
use dev_parser::Device;
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    process::Command,
    sync::Mutex,
};

pub mod dev_parser;

#[derive(PartialEq)]
enum CapacityKind {
    NinetyPercent,
    EightyPercent,
    SeventyPercent,
    FiftyPercent,
    BelowFiftyPercent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    PathPrepend,
    DropPacket,
    None,
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
    /// The capacity of the link. Defaults to 70% of capacity
    #[arg(short, long)]
    threshold_capacity_percent: Option<u64>,
    /// The mode to use when the threshold is reached.
    #[arg(short, long, value_enum)]
    defense_mode: Option<Mode>,
}

async fn restart_quagga(file: Arc<Mutex<File>>) {
    let time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let mut bgp_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open("/etc/quagga/bgpd.conf")
        .await
        .expect("Failed to open bgpd.conf");
    bgp_file
        .write_all(
            r#"!
                        hostname Router
                        password zebra
                        enable password zebra
                        log stdout
                        !
                        bgp config-type cisco
                        !
                        router bgp 65004
                         no synchronization
                         bgp router-id 10.1.4.1
                         network 10.1.3.0 mask 255.255.255.0
                         network 10.1.200.0 mask 255.255.255.0
                         neighbor 10.1.3.2 remote-as 65003
                         neighbor 10.1.3.2 route-map prepend out
                         no auto-summary
                        !
                        route-map prepend permit 10
                         set as-path prepend 65004
                        !
                        line vty
                        !
                        end"#
                .as_bytes(),
        )
        .await
        .expect("Failed to write to bgpd.conf");
    write_to_events(
        Arc::clone(&file),
        format!("#About to restart quagga at {}\n", time_now).as_bytes(),
    )
    .await;
    let output = Command::new("systemctl")
        .arg("restart")
        .arg("quagga")
        .output()
        .await
        .unwrap();
    println!("executed path prepend defense {}", output.status);
    let time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    write_to_events(
        Arc::clone(&file),
        format!("#Restarted quagga at {}\n", time_now).as_bytes(),
    )
    .await;
}

async fn deploy_ip_tables_block(file: Arc<Mutex<File>>) {
    let output = Command::new("iptables")
        .arg("-A")
        .arg("INPUT")
        .arg("-p")
        .arg("udp")
        .arg("--dport")
        .arg("53")
        .arg("-m")
        .arg("string")
        .arg("--from")
        .arg("28")
        .arg("--algo")
        .arg("bm")
        .arg("--hex-string")
        .arg("|06|victim|03|com|02|uk")
        .arg("-j")
        .arg("DROP")
        .output()
        .await
        .unwrap();
    println!("executed drop packet defense {}", output.status);
    let time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    write_to_events(
        file,
        format!("#Dropped packets at {}\n", time_now).as_bytes(),
    )
    .await;
}

async fn write_to_events(events_handle: Arc<Mutex<File>>, buf: &[u8]) {
    let mut events = events_handle.lock().await;
    events.write_all(buf).await.unwrap();
    drop(events);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "linux") != true {
        panic!("This program only works on Linux");
    }

    let args = Cli::parse();

    let uid = nix::unistd::getuid();
    if uid != nix::unistd::Uid::from_raw(0) {
        panic!("This program must be run as root");
    }

    let working_dir = args
        .output_directory
        .unwrap_or(std::env::current_dir().unwrap());

    let interface_name = args.interface;
    let capacity: f64 = args.capacity.unwrap_or(50) as f64 * 10_u64.pow(6) as f64;
    let threshold_capacity_percent: f64 = args.threshold_capacity_percent.unwrap_or(70) as f64;

    println!("Working directory: {:?}", working_dir);
    println!("Listening on interface: {}", interface_name);

    let mut dump_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(working_dir.join("dump.tsv"))
        .await?;

    dump_file
        .write(b"#tsv\ttime\ttxpkts\ttxbytes\trxpkts\trxbytes\n")
        .await?;

    let events_handle = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(working_dir.join("events.tsv"))
            .await?,
    ));

    write_to_events(Arc::clone(&events_handle), b"#tsv\ttime\tevent\n").await;

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
    let mut count = 0;
    let mut defense_deployed = false;

    loop {
        let time_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let devices = dev_parser::get();
        for device in devices {
            if device.interface == interface_name {
                let transmit_bytes = device.transmit_bytes - old_stats.transmit_bytes;
                let receive_bytes = device.receive_bytes - old_stats.receive_bytes;

                dump_file
                    .write(
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
                    .await?;
                let transmit_capacity = ((transmit_bytes as f64 * 8_f64) / capacity) * 100_f64;
                let receive_capacity = ((receive_bytes as f64 * 8_f64) / capacity) * 100_f64;

                if transmit_capacity >= threshold_capacity_percent
                    || receive_capacity >= threshold_capacity_percent
                {
                    count += 1;
                } else {
                    count = 0;
                }

                if transmit_capacity >= 90.0 || receive_capacity >= 90.0 {
                    if capacity_kind != CapacityKind::NinetyPercent {
                        println!(">= 90% capacity {}", time_now);
                        write_to_events(
                            Arc::clone(&events_handle),
                            format!("{}\t>=90%\n", time_now).as_bytes(),
                        )
                        .await;
                        capacity_kind = CapacityKind::NinetyPercent;
                    }
                } else if transmit_capacity >= 80.0 || receive_capacity >= 80.0 {
                    if capacity_kind != CapacityKind::EightyPercent {
                        println!(">= 80% capacity {}", time_now);
                        write_to_events(
                            Arc::clone(&events_handle),
                            format!("{}\t>=80%\n", time_now).as_bytes(),
                        )
                        .await;
                        capacity_kind = CapacityKind::EightyPercent;
                    }
                } else if transmit_capacity >= 70.0 || receive_capacity >= 70.0 {
                    if capacity_kind != CapacityKind::SeventyPercent {
                        println!(">= 70% capacity {}", time_now);
                        write_to_events(
                            Arc::clone(&events_handle),
                            format!("{}\t>=70%\n", time_now).as_bytes(),
                        )
                        .await;
                        capacity_kind = CapacityKind::SeventyPercent;
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
                        write_to_events(
                            Arc::clone(&events_handle),
                            format!("{}\t>=50%\n", time_now).as_bytes(),
                        )
                        .await;
                        capacity_kind = CapacityKind::FiftyPercent;
                    }
                } else if capacity_kind != CapacityKind::BelowFiftyPercent {
                    capacity_kind = CapacityKind::BelowFiftyPercent;
                    println!("< 50% capacity {}", time_now);
                    write_to_events(
                        Arc::clone(&events_handle),
                        format!("{}\t<50%\n", time_now).as_bytes(),
                    )
                    .await;
                }
                old_stats = device;
            }
        }
        if count >= 3 {
            if defense_deployed == false {
                let cloned_handle = Arc::clone(&events_handle);
                match args.defense_mode.unwrap_or(Mode::None) {
                    Mode::DropPacket => {
                        tokio::spawn(async move {
                            deploy_ip_tables_block(cloned_handle).await;
                        });
                    }
                    Mode::PathPrepend => {
                        tokio::spawn(
                            async move { restart_quagga(Arc::clone(&cloned_handle)).await },
                        );
                    }
                    Mode::None => {}
                }

                defense_deployed = true;
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
