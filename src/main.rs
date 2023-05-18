#[macro_use]
extern crate log;

#[macro_use]
extern crate ureq;

use arch_mirrors::country::Kind::{Germany, Serbia, Sweden, Turkey};
use log::{debug, info, trace, warn};
use reqwest::{Client, Response};
use std::collections::HashMap;
use std::error::Error;
use std::fs::copy;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task;
use tokio::task::spawn_blocking;
use std::io::Read;


#[tokio::main]
async fn main() {
    env_logger::init();
    let status = match arch_mirrors::get_status().await {
        Ok(status) => status,
        Err(error) => {
            eprintln!("error: {}", error);
            panic!("Couldn't fetch mirrors")
        }
    };

    let mut res: HashMap<String, u128> = HashMap::new();
    for url in status.urls {
        if !url.url.scheme().contains("rsync") {
            if url.country.kind == Sweden {
                let _url = url.url.clone();
                let speed = spawn_blocking(move || {
                    println!("{}community/os/x86_64/community.db", _url);
                    let start = Instant::now();

                    let resp = ureq::get(format!("{}community/os/x86_64/community.db", _url).as_str()).call().unwrap();

                    let len: usize = resp.header("Content-Length").unwrap().parse().unwrap_or(0);

                    let mut bytes: Vec<u8> = Vec::with_capacity(len);
                    resp.into_reader()
                        .take(10_000_000)
                        .read_to_end(&mut bytes).unwrap();

                    // let x = ureq::get(format!("{}community/os/x86_64/community.db", _url).as_str())
                    //     .call()
                    //     .unwrap()
                    //     .into_string()
                    //     .unwrap_or("".to_string());
                    let millis = start.elapsed().as_millis();
                    println!("{} / {}", len, millis);
                    (len as u128 / millis * 1000  / 1024 / 1024) as u128
                })
                .await
                .unwrap();

                //println!("{} - {}", url.url.to_string(), speed);
                res.insert(url.url.to_string(), speed);
                println!(
                    "## {} - {} Mbit/s",
                    url.url.host_str().unwrap_or("None"),
                    speed as u128
                );
            }
        };
    }
    let mut hash_vec: Vec<(&String, &u128)> = res.iter().collect();
    hash_vec.sort_by(|a, b| b.1.cmp(a.1));
    println!(
        r#"##
## Arch Linux repository mirrorlist
## Created by arch_mirrors
## Generated on {}
##
"#,
        chrono::Utc::now().date_naive()
    );
    for i in hash_vec {
        println!("# {}", i.1);
        println!("Server = {}$repo/os/$arch", i.0)
    }
}
