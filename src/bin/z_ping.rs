//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::time::Instant;
use zenoh::config::Config;
use zenoh::prelude::sync::*;
use zenoh::publication::CongestionControl;
use zenoh::shm::SharedMemoryManager;

const N_SAMPLES: usize = 100;
const WARMUP_SECS: f64 = 1.0;
const SHM_SIZE: usize = 32 * 1024 * 1024;  // Set at most 32 MiB by default

fn main() {
    // // initiate logging
    // zenoh_util::try_init_log_from_env();

    let args: Vec<_> = std::env::args().collect();
    let (size, config) = match args.len() {
        2 => (args[1].parse::<usize>().unwrap(), Config::default()),
        3 => (args[1].parse::<usize>().unwrap(), Config::from_file(args[2].to_owned()).expect("Failed to load the config file.")),
        _ => panic!("Invalid arguments. Use z_ping <payload_size(bytes)> [config_file].")
    };

    let session = zenoh::open(config).res().unwrap();

    let id = session.zid();
    let mut shm = SharedMemoryManager::make(id.to_string(), SHM_SIZE).unwrap();
    let mut buf = shm.alloc(size).unwrap();
    let bs = unsafe { buf.as_mut_slice() };
    for (i, b) in bs.iter_mut().enumerate() {
        *b = (i % 10) as u8;
    }

    // The key expression to publish data on
    let key_expr_ping = keyexpr::new("test/ping").unwrap();

    // The key expression to wait the response back
    let key_expr_pong = keyexpr::new("test/pong").unwrap();

    let sub = session.declare_subscriber(key_expr_pong).res().unwrap();
    let publisher = session
        .declare_publisher(key_expr_ping)
        .congestion_control(CongestionControl::Block)
        .res()
        .unwrap();

    let mut samples = Vec::with_capacity(N_SAMPLES);

    // -- warmup --
    let warmup = std::time::Duration::from_secs_f64(WARMUP_SECS);
    println!("Warming up for {warmup:?}...");
    let now = Instant::now();
    while now.elapsed() < warmup {
        publisher.put(buf.clone()).res().unwrap();

        let _ = sub.recv();
    }

    for _ in 0..N_SAMPLES {
        let write_time = Instant::now();
        publisher.put(buf.clone()).res().unwrap();

        let _ = sub.recv();
        let ts = write_time.elapsed().as_micros();
        samples.push(ts);
    }

    for (i, rtt) in samples.iter().enumerate().take(N_SAMPLES) {
        println!(
            "{} bytes: seq={} rtt={:?}µs lat={:?}µs",
            size,
            i,
            rtt,
            rtt / 2
        );
    }
}
