use std::thread;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::net::UdpSocket;
use std::io::ErrorKind;
use std::time::Duration;
use signal_hook::{consts::SIGINT};

const INITIAL_SEQUENCE_NUMBER: u32 = 1;

fn main() {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&term)).unwrap();
    thread::spawn(move || -> ! {
        loop {
            if term.load(Ordering::Relaxed) {
                println!("Exit");
                process::exit(1);
            }
            thread::sleep(Duration::from_millis(300));
        }
    });

    let socket = UdpSocket::bind("0.0.0.0:1235").expect("Could not bind to address");
    let mut counter = 0;
    let mut buf = [0u8; 2048];
    let mut expected_seq_num = INITIAL_SEQUENCE_NUMBER;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _src)) => {
                let seq_num = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                if seq_num != expected_seq_num {
                    counter += seq_num - expected_seq_num;
                    println!("Received {}, expected {}. Error counter {}", seq_num, expected_seq_num, counter);
                }
                expected_seq_num = seq_num + 1;
                println!("Received {} bytes with seq_num {}", size, seq_num);
                if seq_num > 180 {
                    break
                }
            },
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    println!("WouldBlock on read")
                } else {
                    println!("Error: {:?}", e);
                }
            }
        }
    }
    println!("Missing packets: {}", counter)
}