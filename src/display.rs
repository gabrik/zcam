//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
use clap::{App, Arg};
use opencv::{highgui, prelude::*};
use zenoh::prelude::*;

fn main() {
    // initiate logging
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).wait().unwrap();

    let mut subscriber = session.subscribe(&key_expr).wait().unwrap();

    let window = &format!("[{}] Press 'q' to quit.", &key_expr);
    highgui::named_window(window, 1).unwrap();

    while let Ok(sample) = subscriber.receiver().recv() {
        let decoded = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_iter(sample.value.payload.to_vec()),
            opencv::imgcodecs::IMREAD_COLOR,
        )
        .unwrap();

        if decoded.size().unwrap().width > 0 {
            // let mut enlarged = Mat::default().unwrap();
            // opencv::imgproc::resize(&decoded, &mut enlarged, opencv::core::Size::new(800, 600), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();
            highgui::imshow(window, &decoded).unwrap();
        }

        if highgui::wait_key(10).unwrap() == 113 {
            // 'q'
            break;
        }
    }
    subscriber.close().wait().unwrap();
    session.close().wait().unwrap();
}

fn parse_args() -> (zenoh::config::Config, String) {
    let args = App::new("zenoh-net video display example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .arg(
            Arg::from_usage("-k, --key=[KEYEXPR]        'The key expression to publish onto.'")
                .default_value("/demo/example/zenoh-rs-pub"),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .get_matches();

    let mut config = zenoh::config::Config::default();

    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    match args.value_of("mode").map(|m| m.parse()) {
        Some(Ok(mode)) => {
            config.set_mode(Some(mode)).unwrap();
        }
        Some(Err(())) => panic!("Invalid mode"),
        None => {}
    };
    if let Some(values) = args.values_of("peer") {
        config.peers.extend(values.map(|v| v.parse().unwrap()))
    }

    let key_expr = args.value_of("key").unwrap().to_string();

    (config, key_expr)
}
