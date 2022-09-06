use std::{thread, time::Duration};

pub mod dev_parser;

const BOUND_IP_ADDR: &'static str = "10.1.3.3:0";

fn main() {
    println!("Hello, world!");
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

    let interfaces = nix::ifaddrs::getifaddrs().unwrap();
    let mut interface_name = String::new();
    for interface in interfaces {
        match interface.address {
            Some(address) => {
                println!("interface {} address {}", interface.interface_name, address);
                if address.to_string() == BOUND_IP_ADDR {
                    interface_name = interface.interface_name;
                }
                // if address.to_string() == "127.0.0.1:0" {
                //     interface_name = interface.interface_name;
                // }
            }
            None => {
                println!(
                    "interface {} with unsupported address family",
                    interface.interface_name
                );
            }
        }
    }

    println!("Interface name: {}", interface_name);
    loop {
        let devices = dev_parser::get();
        for device in devices {
            if device.interface == interface_name {
                println!("{}: {}", device.interface, device.receive_bytes);
            }
            // println!("{:?}", device);
        }
        thread::sleep(Duration::from_secs(1));
        println!("\n\n\n 1 sec up!! \n\n")
    }

    // let contents =
    //     fs::read_to_string(PROC_NET_DEV).expect("Failed to read the file at /proc/net/dev");

    // println!("With text:\n{contents}");
}
