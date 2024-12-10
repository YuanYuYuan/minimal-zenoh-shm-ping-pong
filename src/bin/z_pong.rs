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
use zenoh::{
    Wait,
    config::Config,
    qos::CongestionControl,
    key_expr::KeyExpr,
};


fn main() {
    let args: Vec<_> = std::env::args().collect();
    let config = match args.len() {
        1 => Config::default(),
        2 => Config::from_file(args[1].to_owned()).expect("Failed to load the config file."),
        _ => panic!("Invalid arguments. Use z_pong [config_file].")
    };

    let session = zenoh::open(config).wait().unwrap();

    // The key expression to read the data from
    let key_expr_ping = KeyExpr::new("test/ping").unwrap();

    // The key expression to echo the data back
    let key_expr_pong = KeyExpr::new("test/pong").unwrap();

    let publisher = session
        .declare_publisher(key_expr_pong)
        .congestion_control(CongestionControl::Block)
        .wait()
        .unwrap();

    let _sub = session
        .declare_subscriber(key_expr_ping)
        .callback(move |sample| publisher.put(sample.payload().clone()).wait().unwrap())
        .wait()
        .unwrap();
    std::thread::park();
}
