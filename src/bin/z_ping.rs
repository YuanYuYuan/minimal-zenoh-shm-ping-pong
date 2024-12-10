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
use zenoh::{
    Wait,
    config::Config,
    qos::CongestionControl,
    key_expr::KeyExpr,
    shm::{
        ZShm, BlockOn, GarbageCollect, PosixShmProviderBackend, ShmProviderBuilder, POSIX_PROTOCOL_ID,
    },
};

const N_SAMPLES: usize = 10000;
const WARMUP_SECS: f64 = 1.0;
const SHM_SIZE: usize = 32 * 1024 * 1024;  // Set at most 32 MiB by default

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let (size, config) = match args.len() {
        2 => (args[1].parse::<usize>().unwrap(), Config::default()),
        3 => (args[1].parse::<usize>().unwrap(), Config::from_file(args[2].to_owned()).expect("Failed to load the config file.")),
        _ => panic!("Invalid arguments. Use z_ping <payload_size(bytes)> [config_file].")
    };

    let session = zenoh::open(config).wait().unwrap();

    let backend = PosixShmProviderBackend::builder()
        .with_size(SHM_SIZE)
        .unwrap()
        .wait()
        .unwrap();
    let provider = ShmProviderBuilder::builder()
        .protocol_id::<POSIX_PROTOCOL_ID>()
        .backend(backend)
        .wait();

    let sbuf: ZShm = {
        let layout = provider.alloc(size).into_layout().unwrap();
        let mut sbuf = layout
            .alloc()
            .with_policy::<BlockOn<GarbageCollect>>()
            .wait()
            .unwrap();
        for (i, b) in sbuf.iter_mut().enumerate() {
            *b = (i % 10) as u8;
        }
        sbuf.into()
    };

    // The key expwaitsion to publish data on
    let key_expr_ping = KeyExpr::new("test/ping").unwrap();

    // The key expwaitsion to wait the waitponse back
    let key_expr_pong = KeyExpr::new("test/pong").unwrap();

    let sub = session.declare_subscriber(key_expr_pong).wait().unwrap();
    let publisher = session
        .declare_publisher(key_expr_ping)
        .congestion_control(CongestionControl::Block)
        .wait()
        .unwrap();

    let mut samples = Vec::with_capacity(N_SAMPLES);

    // -- warmup --
    let warmup = std::time::Duration::from_secs_f64(WARMUP_SECS);
    println!("Warming up for {warmup:?}...");
    let now = Instant::now();
    while now.elapsed() < warmup {
        publisher.put(sbuf.clone()).wait().unwrap();

        let _ = sub.recv();
    }

    for _ in 0..N_SAMPLES {
        let write_time = Instant::now();
        publisher.put(sbuf.clone()).wait().unwrap();

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
