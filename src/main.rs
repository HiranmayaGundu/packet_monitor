use std::{
    fs::OpenOptions,
    io::Write,
    thread,
    time::{Duration, SystemTime},
};

use dev_parser::Device;

pub mod dev_parser;

const BOUND_IP_ADDR: &'static str = "10.1.3.3:0";
const CAPACITY: u64 = 100 * 10_u64.pow(6) / 8;

fn main() {
    // println!("Hello, world!");
    if cfg!(target_os = "linux") != true {
        panic!("This program only works on Linux");
    }
    // let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    // let result = unsafe { getifaddrs(addrs.as_mut_ptr()) };
    // println!("Result: {}", result);
    // println!("{:?}", addrs);
    // let mut ifaddrs = unsafe { addrs.assume_init() };
    // while ifaddrs != std::ptr::null_mut() {
    //     let ifa_name = unsafe { ffi::CStr::from_ptr((*ifaddrs).ifa_name) }
    //         .to_string_lossy()
    //         .to_string();
    //     let ifa_addr = unsafe { (*ifaddrs).ifa_addr };
    //     let result = unsafe {
    //         let family = (*ifa_addr).sa_family;
    //         let ipv4 = libc::AF_INET.abs() as u16;
    //         let ipv6 = libc::AF_INET6.abs() as u16;
    //         match family {
    //             ipv4 => {
    //                 let addr = (*ifa_addr).sa_data;
    //                 let addr = SocketAddr::from((addr, 0));
    //                 println!("{}: {}", ifa_name, addr);
    //             }
    //             ipv6 => {
    //                 let addr = (*ifa_addr).sa_data;
    //                 let addr = SocketAddr::from((addr, 0));
    //                 println!("{}: {}", ifa_name, addr);
    //             }
    //             _ => {}
    //         }
    //     };
    //     ifaddrs = unsafe { (*ifaddrs).ifa_next };
    //     println!("Interface: {}", ifa_name);
    // }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("dump.tsv")
        .unwrap();

    file.write_all(b"#tsv\ttime\ttxpkts\ttxbytes\trxpkts\trxbytes\n")
        .expect("Failed to write header");

    let interfaces = nix::ifaddrs::getifaddrs().unwrap();
    let mut interface_name = String::new();
    for interface in interfaces {
        match interface.address {
            Some(address) => {
                // println!("interface {} address {}", interface.interface_name, address);
                if address.to_string() == BOUND_IP_ADDR {
                    interface_name = interface.interface_name;
                }
                // if address.to_string() == "127.0.0.1:0" {
                //     interface_name = interface.interface_name;
                // }
            }
            None => {
                // println!(
                //     "interface {} with unsupported address family",
                //     interface.interface_name
                // );
            }
        }
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

    loop {
        let devices = dev_parser::get();
        for device in devices {
            if device.interface == interface_name {
                // println!("{}: {}", device.interface, device.receive_bytes);

                let transmit_bytes = device.transmit_bytes - old_stats.transmit_bytes;
                let receive_bytes = device.receive_bytes - old_stats.receive_bytes;
                file.write_all(
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
                let transmit_capacity = receive_bytes / CAPACITY * 100;
                let receive_capacity = transmit_bytes / CAPACITY * 100;

                if transmit_capacity >= 90 || receive_capacity >= 90 {
                    println!(
                        ">= 90% capacity {}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64()
                    );
                } else if transmit_capacity >= 80 || receive_capacity >= 80 {
                    println!(
                        ">= 80% capacity {}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64()
                    );
                } else if transmit_capacity >= 50 || receive_capacity >= 50 {
                    println!(
                        ">= 50% capacity {}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64()
                    );
                }
            }
        }
        thread::sleep(Duration::from_secs(1));
    }

    // let contents =
    //     fs::read_to_string(PROC_NET_DEV).expect("Failed to read the file at /proc/net/dev");

    // println!("With text:\n{contents}");
}
