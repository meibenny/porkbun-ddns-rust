use clap::Parser;
use anyhow::{Context, Result};

#[derive(Parser)]
struct Cli {
    /// The path to the config file
    config_file: std::path::PathBuf,
}

// fn get_current_ip() -> reqwest::Result<()> {
fn get_current_ip() -> Result<String, reqwest::Error> {
// fn get_current_ip() -> std::string::String {
    let resp = reqwest::blocking::get("https://checkip.amazonaws.com/")?;
    let ip = resp.text()?;
    Ok(ip)
}

fn update_dns_entry() {
    println!("{}", "updating dns entry");
}

fn main() -> Result<()> {
    println!("Hello, world!");
    let args = Cli::parse();
    let config_path = &args.config_file;
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("could not read config '{}'", config_path.display()))?;
    for line in config_contents.lines() {
        println!("{}", line);
    }
    let parsed_config = json::parse(&config_contents).unwrap();
    let secretkey = &parsed_config["secretkey"];
    println!("{}", parsed_config);
    println!("{}", secretkey);
    let current_ip_result = get_current_ip();
    let current_ip = match current_ip_result {
       Ok(ip) => ip,
       Err(error) => panic!("Could not retrieve current IP: {:?}", error),
    };
    println!("{}", current_ip);
    Ok(())
}
