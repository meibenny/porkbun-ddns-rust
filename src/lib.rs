use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigDomainsStruct {
    pub domain: String,
    pub dns_entry_type: String,
    pub subdomain: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub secretkey: String,
    pub apikey: String,
    pub discord_webhook_url: String,
    pub check_ip_providers: Vec<String>,
    pub domains: Vec<ConfigDomainsStruct>,
    pub porkbun_base_url: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunUpdateDNSStruct {
    secretapikey: String,
    apikey: String,
    content: String,
    ttl: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSRequest {
    secretapikey: String,
    apikey: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSRecord {
    id: String,
    name: String,
    #[serde(rename="type")]
    type_: String,
    content: String,
    ttl: String,
    prio: String,
    notes: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSResponse {
    status: String,
    records: Vec<PorkbunQueryDNSRecord>
}

#[derive(Serialize, Deserialize)]
struct PorkbunUpdateDNSResponse {
    status: String,
    message: Option<String>
}

#[derive(Serialize, Deserialize)]
struct DiscordMessageRequest {
    content: String
}

fn get_current_ip(config: &Config) -> Result<String, reqwest::Error> {
    let ref url = config.check_ip_providers[0];
    let resp = reqwest::blocking::get(url)?;
    let ip = resp.text()?;
    // let ip = "192.168.0.140".to_string();
    Ok(ip)
}

fn get_current_dns_entry(domain_config: &ConfigDomainsStruct, config: &Config) -> Result<String, reqwest::Error> {
    let ref secretkey = config.secretkey;
    let ref apikey = config.apikey;
    let porkbun_payload = PorkbunQueryDNSRequest {
        secretapikey: String::from(secretkey),
        apikey: String::from(apikey)
    };
    let url = &format!(
        "{}/retrieveByNameType/{}/{}/{}",
        &config.porkbun_base_url, domain_config.domain, domain_config.dns_entry_type, domain_config.subdomain
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url)
        .json(&porkbun_payload)
        .send()?;
    //println!("{}", &resp.text()?);
    let parsed_resp = resp.json::<PorkbunQueryDNSResponse>()?;
    Ok(parsed_resp.records[0].content.to_string())
}

fn update_dns_entry(
    current_ip: &String,
    current_dns_entry: &String,
    domain_config: &ConfigDomainsStruct,
    config: &Config
) -> Result<String, reqwest::Error> {
    if current_ip.eq(current_dns_entry) {
       return Ok(
           format!(
               "Current IP: {}, identical to current DNS Entry, {}. Not updating.",
               current_ip, current_dns_entry
           ).to_string()
       )
    }
    let ref secretkey = config.secretkey;
    let ref apikey = config.apikey;
    let url = &format!(
        "{}/editByNameType/{}/{}/{}",
        &config.porkbun_base_url, domain_config.domain, domain_config.dns_entry_type, domain_config.subdomain
    );
    let porkbun_payload = PorkbunUpdateDNSStruct {
        secretapikey: String::from(secretkey),
        apikey: String::from(apikey),
        content: String::from(current_ip),
        ttl: String::from("600")
    };
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url)
        .json(&porkbun_payload)
        .send()?;
    let parsed_resp = resp.json::<PorkbunUpdateDNSResponse>()?;
    if (parsed_resp.status).eq("SUCCESS") {
        return Ok(format!("Updated DNS Entry to {}", current_ip).to_string())
    } else {
        return Ok(format!("Could not update DNS Entry. {} -> {}", current_dns_entry, current_ip).to_string())
    }
}

fn update_discord(config: &Config, message: &String) -> Result<String, reqwest::Error> {
    let ref url = config.discord_webhook_url;
    let discord_payload = DiscordMessageRequest {
        content: String::from(message)
    };
    let client = reqwest::blocking::Client::new();
    client.post(url)
        .json(&discord_payload)
        .send()?;
    Ok("Ok".to_string())
}

pub fn update_dns(config: &Config) -> Result<Vec<String>> {
    let current_ip_result = get_current_ip(&config);
    let current_ip = match current_ip_result {
       Ok(ip) => ip,
       Err(error) => panic!("Could not retrieve current IP: {:?}", error),
    };
    let mut update_results = Vec::new();
    let domains = &config.domains;
    let domain_iter = domains.iter();
    for domain_config in domain_iter {
        let domain = &domain_config.domain;
        let current_entry = get_current_dns_entry(&domain_config, &config).unwrap();
        let update_result = update_dns_entry(&current_ip, &current_entry, &domain_config, &config).unwrap();
        let domain_string = format!("{0}.{1}: ", domain_config.subdomain, domain);
        let update_message = domain_string.to_owned() + &update_result;
        update_discord(&config, &update_message).unwrap();
        update_results.push(update_message);
    }
    Ok(update_results)
}