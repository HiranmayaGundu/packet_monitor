use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Lines;

const PROC_NET_DEV: &'static str = "/proc/net/dev";

#[derive(Debug)]
pub struct Device {
    pub interface: String,
    pub receive_bytes: u64,
    pub receive_packets: u64,
    pub receive_errs: u64,
    pub receive_drop: u64,
    pub receive_fifo: u64,
    pub receive_frame: u64,
    pub receive_compressed: u64,
    pub receive_multicast: u64,
    pub transmit_bytes: u64,
    pub transmit_packets: u64,
    pub transmit_errs: u64,
    pub transmit_drop: u64,
    pub transmit_fifo: u64,
    pub transmit_colls: u64,
    pub transmit_carrier: u64,
    pub transmit_compressed: u64,
}

pub fn get() -> Vec<Device> {
    let file = File::open(PROC_NET_DEV)
        .expect(format!("Failed to open file at {}", PROC_NET_DEV).as_str());

    let reader = BufReader::new(file);

    return parse(reader.lines());
}

impl Device {
    pub fn new() -> Device {
        Device {
            interface: String::new(),
            receive_bytes: 0,
            receive_compressed: 0,
            receive_drop: 0,
            receive_errs: 0,
            receive_fifo: 0,
            receive_frame: 0,
            receive_multicast: 0,
            receive_packets: 0,
            transmit_bytes: 0,
            transmit_carrier: 0,
            transmit_colls: 0,
            transmit_compressed: 0,
            transmit_drop: 0,
            transmit_errs: 0,
            transmit_fifo: 0,
            transmit_packets: 0,
        }
    }
}

fn parse(mut lines: Lines<BufReader<File>>) -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    lines.next();
    lines.next();
    for line in lines {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let interface = parts.next().unwrap().split(':').next().unwrap().to_string();
        let receive_bytes = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_packets = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_errs = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_drop = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_fifo = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_frame = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_compressed = parts.next().unwrap().parse::<u64>().unwrap();
        let receive_multicast = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_bytes = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_packets = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_errs = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_drop = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_fifo = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_colls = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_carrier = parts.next().unwrap().parse::<u64>().unwrap();
        let transmit_compressed = parts.next().unwrap().parse::<u64>().unwrap();

        let device = Device {
            interface,
            receive_bytes,
            receive_packets,
            receive_errs,
            receive_drop,
            receive_fifo,
            receive_frame,
            receive_compressed,
            receive_multicast,
            transmit_bytes,
            transmit_packets,
            transmit_errs,
            transmit_drop,
            transmit_fifo,
            transmit_colls,
            transmit_carrier,
            transmit_compressed,
        };

        devices.push(device);
    }
    return devices;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let devices = get();
        assert!(devices.len() > 0);
        println!("{:?}", devices);
    }
}
