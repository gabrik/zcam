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
use opencv::{core, prelude::*, videoio};
use zenoh::prelude::*;

fn main() {
    // initiate logging
    env_logger::init();

    let (config, key_expr, resolution, delay) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).wait().unwrap();

    let expr_id = session.declare_expr(&key_expr).wait().unwrap();
    session.declare_publication(expr_id).wait().unwrap();

    #[cfg(feature = "opencv-32")]
    let mut cam = videoio::VideoCapture::new_default(0).unwrap(); // 0 is the default camera
    #[cfg(not(feature = "opencv-32"))]
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut encode_options = opencv::types::VectorOfi32::new();
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(90);

    loop {
        let mut frame = core::Mat::default();
        cam.read(&mut frame).unwrap();

        let mut reduced = Mat::default();
        opencv::imgproc::resize(
            &frame,
            &mut reduced,
            opencv::core::Size::new(resolution[0], resolution[1]),
            0.0,
            0.0,
            opencv::imgproc::INTER_LINEAR,
        )
        .unwrap();

        let mut buf = opencv::types::VectorOfu8::new();
        opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options).unwrap();

        let data: Vec<u8> = buf.into();

        session.put(expr_id, data).wait().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }
}

fn parse_args() -> (zenoh::config::Config, String, Vec<i32>, u64) {
    let args = App::new("zenoh-net videocapture example")
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
        .arg(
            Arg::from_usage(
                "-r, --resolution=[RESOLUTION] 'The resolution of the published video.",
            )
            .default_value("640x360"),
        )
        .arg(
            Arg::from_usage("-d, --delay=[DELAY] 'The delay between each frame in milliseconds.")
                .default_value("40"),
        )
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
    let resolution = args
        .value_of("resolution")
        .unwrap()
        .split('x')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    let delay = args.value_of("delay").unwrap().parse::<u64>().unwrap();

    (config, key_expr, resolution, delay)
}
