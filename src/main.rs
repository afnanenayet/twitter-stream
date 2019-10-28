mod config;
mod sentiment;
mod server;

use canteen::utils;
use canteen::*;
use serde_yaml;
use std::{fs::File, io::BufReader, thread};
use structopt::StructOpt;

fn main() -> Result<(), anyhow::Error> {
    // First, get the config from a YAML file
    let opts = config::CliOpts::from_args();
    let file = File::open(opts.config_file)?;
    let reader = BufReader::new(file);
    let config: config::TwitterConfig = serde_yaml::from_reader(reader)?;

    let mut http_server = Canteen::new();
    http_server.bind(("127.0.0.1", 8080));

    // set the default route handler to show a 404 message
    http_server.set_default(utils::err_404);

    let cfg_wrapper = config::Config {
        config: Box::new(config),
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
