mod config;
mod sentiment;
mod server;

use serde_yaml;
use std::{fs::File, io::BufReader};
use structopt::StructOpt;

fn main() -> Result<(), anyhow::Error> {
    // First, get the config from a YAML file
    let opts = config::CliOpts::from_args();
    let file = File::open(opts.config_file)?;
    let reader = BufReader::new(file);
    let config: config::TwitterConfig = serde_yaml::from_reader(reader)?;

    let cfg_wrapper = config::Config {
        config: Box::new(config),
    }
    .verify()
    .expect("The provided configuration values were invalid");
    let mut worker = server::Server::new(cfg_wrapper);

    // start the worker that collects data on a separate thread so we can run the webserver
    // concurrently
    worker.run().unwrap();
    Ok(())
}
