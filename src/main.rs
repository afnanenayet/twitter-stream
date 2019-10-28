mod config;
mod sentiment;
mod server;

use canteen::utils;
use canteen::*;
use lazy_static::lazy_static;
use std::{str::FromStr, thread};

fn main() -> Result<(), anyhow::Error> {
    let mut http_server = Canteen::new();
    http_server.bind(("127.0.0.1", 8080));

    // set the default route handler to show a 404 message
    http_server.set_default(utils::err_404);

    let cfg = config::TwitterConfig {
        keywords: vec!["help", "me"]
            .into_iter()
            .map(|x| String::from_str(x).unwrap())
            .collect(),
        ..config::TwitterConfig::default()
    };

    let cfg_wrapper = config::Config {
        config: Box::new(cfg),
    }
    .verify()
    .expect("The provided configuration values were invalid");
    let mut worker = server::Server::new(cfg_wrapper);

    // respond to requests to / by serving the graph
    http_server.add_route("/", &[Method::Get], |_| {
        let mut res = Response::new();
        res.set_status(200);
        res.set_content_type("text/plain");
        res
    });

    // start the worker that collects data on a separate thread so we can run the webserver
    // concurrently
    let handler = thread::spawn(move || {
        worker.run().unwrap();
    });
    http_server.run();
    handler.join().unwrap();
    Ok(())
}
