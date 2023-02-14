use anyhow::{Context, Result};
use clap::Parser;
use env_logger::{Builder, Env, Target};
use log::{info};
use std::fs::OpenOptions;
use ddns_rust::{Config, update_dns};


#[derive(Parser)]
struct Cli {
    /// The path to the config file
    config_file: std::path::PathBuf,
}

fn main() -> Result<()> {
    let logfile = Box::new(
        OpenOptions::new().create(true)
            .append(true).open("ddns_rust.log")
            .expect("Couldn't create log file")
    );
    let env = Env::default()
        .default_filter_or("info")
        .default_write_style_or("always");
    let mut builder = Builder::from_env(env);
    builder.target(Target::Pipe(logfile));
    builder.init();

    let args = Cli::parse();
    let config_path = &args.config_file;
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("could not read config '{}'", config_path.display()))?;
    let parsed_config: Config = serde_json::from_str(&config_contents).unwrap();
    let update_result = update_dns(&parsed_config).unwrap();
    info!("{:?}", update_result);
    Ok(())
}
