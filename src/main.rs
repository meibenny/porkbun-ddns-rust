use anyhow::{Context, Result};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{info};
use ddns_rust::{Config, update_dns};


#[derive(Parser)]
#[command(
    version,
    about = "Dnyamic DNS with Porkbun API",
    long_about
)]
struct Cli {
    /// The path to the config file
    #[arg(index = 1, required = true)]
    config_file: std::path::PathBuf,
}

fn main() -> Result<()> {
    
    let env = Env::default()
        .default_filter_or("info")
        .default_write_style_or("never");
    let mut builder = Builder::from_env(env);
    builder.init();

    let args = Cli::parse();
    let config_path = &args.config_file;
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("could not read config '{}'", config_path.display()))?;
    let parsed_config: Config = serde_json::from_str(&config_contents).unwrap();
    let update_results = update_dns(&parsed_config).unwrap();
    for result in update_results.iter() {
        info!("{:?}", result.trim_end_matches("\n"));
    }
    Ok(())
}
