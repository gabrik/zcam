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
#![feature(async_closure)]
use clap::{App, Arg, Values};
use futures::prelude::*;
use zenoh::net::*;
use opencv::{
    highgui,
    prelude::*,
};

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let args = App::new("zenoh-net video display example")
    .arg(Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
        .possible_values(&["peer", "client"]).default_value("peer"))
    .arg(Arg::from_usage("-p, --path=[path] 'The zenoh path on which the video will be published.")
        .default_value("/demo/zcam"))
    .arg(Arg::from_usage("-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'"))
        .get_matches();

    let path = args.value_of("path").unwrap();

    let mut config = config::empty();
    config.push((
        config::ZN_MODE_KEY,
        args.value_of("mode").unwrap().as_bytes().to_vec(),
    ));
    for peer in args
        .values_of("peer")
        .or_else(|| Some(Values::default()))
        .unwrap()
    {
        config.push((config::ZN_PEER_KEY, peer.as_bytes().to_vec()));
    }

    println!("Openning session...");
    let session = open(config).await.unwrap();
    let sub_info = SubInfo {
        reliability: Reliability::Reliable,
        mode: SubMode::Push,
        period: None
    };
    let mut sub = session.declare_subscriber(&path.into(), &sub_info).await.unwrap();

    let window = "video";
    highgui::named_window(window, 1).unwrap();

    sub.stream().for_each(async move |sample| {
        let decoded = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_iter(sample.payload.to_vec()), 
            opencv::imgcodecs::IMREAD_COLOR).unwrap();

        if decoded.size().unwrap().width > 0 {
            // let mut enlarged = Mat::default().unwrap();
            // opencv::imgproc::resize(&decoded, &mut enlarged, opencv::core::Size::new(800, 600), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();
            highgui::imshow(window, &decoded).unwrap();
        }

        highgui::wait_key(10).unwrap();
    }).await;

    sub.undeclare().await.unwrap();
    session.close().await.unwrap();
}
