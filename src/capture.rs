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
use clap::{App, Arg, Values};
use zenoh::net::*;
use zenoh::net::ResKey::*;
use opencv::{
    core,
    prelude::*,
    videoio,
};

fn main() {
    // initiate logging
    env_logger::init();

    let (config, path, resolution, delay) = parse_args();

    println!("Openning session...");
    let session = open(config).wait().unwrap();

    let reskey = RId(session.declare_resource(&path.into()).wait().unwrap());
    let _publ = session.declare_publisher(&reskey).wait().unwrap();

    #[cfg(feature = "opencv-32")]
    let mut cam = videoio::VideoCapture::new_default(0).unwrap();  // 0 is the default camera
    #[cfg(not(feature = "opencv-32"))]
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();  // 0 is the default camera
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
        opencv::imgproc::resize(&frame, &mut reduced, opencv::core::Size::new(resolution[0], resolution[1]), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();

        let mut buf = opencv::types::VectorOfu8::new();
        opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options).unwrap();

        session.write(&reskey, buf.to_vec().into()).wait().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }
}

fn parse_args() -> (ConfigProperties, String, Vec<i32>, u64){

    let args = App::new("zenoh-net videocapture example")
        .arg(Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
            .possible_values(&["peer", "client"]).default_value("peer"))
        .arg(Arg::from_usage("-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'"))
        .arg(Arg::from_usage("-p, --path=[PATH] 'The zenoh path on which the video will be published.")
            .default_value("/demo/zcam"))
        .arg(Arg::from_usage("-r, --resolution=[RESOLUTION] 'The resolution of the published video.")
            .default_value("600x400"))
        .arg(Arg::from_usage("-d, --delay=[DELAY] 'The delay between each frame in milliseconds.")
            .default_value("40"))
        .get_matches();

    let mut config = config::empty();
    config.insert(
        config::ZN_MODE_KEY,
        String::from(args.value_of("mode").unwrap())
    );
    for peer in args
        .values_of("peer")
        .or_else(|| Some(Values::default()))
        .unwrap()
    {
        config.insert(config::ZN_PEER_KEY, String::from(peer));
    }

    let path = args.value_of("path").unwrap();
    let resolution = args.value_of("resolution").unwrap()
        .split('x').map(|s| s.parse::<i32>().unwrap()).collect::<Vec<i32>>();
    let delay = args.value_of("delay").unwrap().parse::<u64>().unwrap();

    (config, path.to_string(),resolution, delay)
}