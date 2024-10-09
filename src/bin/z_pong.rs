use core::panic;

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
use zenoh::config::Config;
use zenoh::prelude::sync::*;
use zenoh::publication::CongestionControl;


fn main() {
    // // initiate logging
    // zenoh_util::try_init_log_from_env();

    let args: Vec<_> = std::env::args().collect();
    let config = match args.len() {
        1 => Config::default(),
        2 => Config::from_file(args[1].to_owned()).expect("Failed to load the config file."),
        _ => panic!("Invalid arguments. Use z_pong [config_file].")
    };

    let session = zenoh::open(config).res().unwrap().into_arc();

    // The key expression to read the data from
    let key_expr_ping = keyexpr::new("test/ping").unwrap();

    // The key expression to echo the data back
    let key_expr_pong = keyexpr::new("test/pong").unwrap();

    let publisher = session
        .declare_publisher(key_expr_pong)
        .congestion_control(CongestionControl::Block)
        .res()
        .unwrap();

    let _sub = session
        .declare_subscriber(key_expr_ping)
        .callback(move |sample| publisher.put(sample.value).res().unwrap())
        .res()
        .unwrap();
    std::thread::park();
}
