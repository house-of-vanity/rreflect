use crossbeam_utils::thread;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use url::{ParseError, Url};

use crate::structs::Mirrorlist;
mod structs;

const SKIP_LINES: &'static [&'static str] = &["Arch Linux repository mirrorlist", "Generated on"];

fn main() -> Result<(), ureq::Error> {
    let mut mirrorlist: Mirrorlist = Mirrorlist {
        country: Default::default(),
    };

    mirrorlist.country.insert("Default".to_string(), vec![]);

    let body: String = ureq::get("https://archlinux.org/mirrorlist/?country=AT&protocol=http&protocol=https&ip_version=4")
        .call()?
        .into_string()?;

    let mut current_country: &str = "";
    'line: for line in body.lines() {
        if line.starts_with("## ") {
            for s in SKIP_LINES.iter() {
                if line.contains(s) {
                    continue 'line;
                }
            }
            match line.get(3..) {
                Some(country) => {
                    mirrorlist.country.insert(country.to_string(), vec![]);
                    current_country = country;
                }
                _ => {
                    continue 'line;
                }
            };
        } else if line.starts_with("#Server = ") {
            match mirrorlist.country.get_mut(current_country) {
                None => {}
                Some(country) => {
                    match line.get(10..) {
                        Some(url) => {
                            country.push(url.to_string());
                        }
                        _ => {
                            continue 'line;
                        }
                    };
                }
            };
        }
    }
    //println!("{:?}", mirrorlist);
    let mut urls: Vec<String> = vec![];

    for country in &mirrorlist.country {
        println!("{:?}!", country.0);
        thread::scope(|s| {
            for url in country.1 {
                s.spawn(move |_| {
                    //println!("{:?}!", url);
                    match Url::parse(url.as_str()) {
                        Ok(p) => {
                            let start = Instant::now();
                            match ureq::get(
                                format!("{}://{}", p.scheme(), p.host_str().unwrap()).as_str(),
                            )
                                .timeout(Duration::from_secs(1))
                                .call()
                            {
                                Ok(body) => {
                                    let duration = start.elapsed();
                                    &urls.push(body.into_string().unwrap());
                                    println!("{} {} takes {:?}", url, p.host_str().unwrap(), duration);
                                }
                                Err(_) => {}
                            };


                        },
                        Err(e) => {
                            unimplemented!()
                        }
                    }
                });
            }
        })
        .unwrap();
    }

    /*
       let server_list: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
       let pool = ThreadPool::new(100);

       for (country, urls) in mirrorlist.country {

           let server_list = Arc::clone(&server_list);

           pool.execute(move || {
               println!("THREAD");
               for server in urls {

                   match Url::parse(server.as_str()) {
                       Ok(p) => {
                           let start = Instant::now();
                           match ureq::get(
                               format!("{}://{}", p.scheme(), p.host_str().unwrap()).as_str(),
                           )
                           .timeout(Duration::from_secs(1))
                           .call()
                           {
                               Ok(_) => {
                                   let mut server_list = server_list.lock().unwrap();
                                   server_list.push(server.clone())
                               },
                               Err(e) => {
                                   println!("{:?}", e)
                               }
                           };
                           let duration = start.elapsed();
                           //println!("{} {} takes {:?}", country, p.host_str().unwrap(), duration)
                       }
                       Err(_) => {}
                   };
               }
           });
           pool.join()
       };

       println!("Server = {:?}", server_list.lock().unwrap());

    */
    Ok(())
}
