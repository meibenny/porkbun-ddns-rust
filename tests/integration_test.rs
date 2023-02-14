use ddns_rust::{Config, ConfigDomainsStruct, update_dns};

#[test]
fn test_update_dns() {
    let mut amazon = mockito::Server::new();
    let mut porkbun = mockito::Server::new();
    let mut discord = mockito::Server::new();

    let amazon_mock = amazon.mock("GET", "/")
        .with_status(200)
        .with_body("10.0.1.1")
        .create();

    let domain_config = ConfigDomainsStruct {
        domain: String::from("domain.tld"),
        dns_entry_type: String::from("A"),
        subdomain: String::from("subdomain")
    };

    let config = Config {
        secretkey: String::from("secretkey"),
        apikey: String::from("apikey"),
        discord_webhook_url: String::from(discord.url()),
        check_ip_providers: Vec::from([amazon.url()]),
        domains: Vec::from([domain_config]),
        porkbun_base_url: String::from(porkbun.url())
    };

    let retrieve_by_name_mock = porkbun.mock(
        "POST",
        &*format!(
            "/retrieveByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "SUCCESS", "records": [{"id": "12345", "name": "subdomain.domain.tld", "type": "A", "content": "10.0.1.2", "ttl": "600", "prio": "0", "notes": ""}]}"#)
    .create();

    let update_dns_mock = porkbun.mock(
        "POST",
        &*format!(
            "/editByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "SUCCESS"}"#)
    .create();

    let mock_discord = discord.mock("POST", "/").with_status(200).create();


    let actual = update_dns(&config).unwrap();
    assert_eq!("Updated DNS Entry to 10.0.1.1", actual);
    amazon_mock.assert();
    retrieve_by_name_mock.assert();
    update_dns_mock.assert();
    mock_discord.assert();
}

#[test]
fn test_ip_already_updated() {
    let mut amazon = mockito::Server::new();
    let mut porkbun = mockito::Server::new();
    let mut discord = mockito::Server::new();

    let amazon_mock = amazon.mock("GET", "/")
        .with_status(200)
        .with_body("10.0.1.1")
        .create();

    let domain_config = ConfigDomainsStruct {
        domain: String::from("domain.tld"),
        dns_entry_type: String::from("A"),
        subdomain: String::from("subdomain")
    };

    let config = Config {
        secretkey: String::from("secretkey"),
        apikey: String::from("apikey"),
        discord_webhook_url: String::from(discord.url()),
        check_ip_providers: Vec::from([amazon.url()]),
        domains: Vec::from([domain_config]),
        porkbun_base_url: String::from(porkbun.url())
    };

    let retrieve_by_name_mock = porkbun.mock(
        "POST",
        &*format!(
            "/retrieveByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "SUCCESS", "records": [{"id": "12345", "name": "subdomain.domain.tld", "type": "A", "content": "10.0.1.1", "ttl": "600", "prio": "0", "notes": ""}]}"#)
    .create();

    let mock_discord = discord.mock("POST", "/").with_status(200).create();

    let actual = update_dns(&config).unwrap();
    assert_eq!(
        "Current IP: 10.0.1.1, identical to current DNS Entry, 10.0.1.1. Not updating.",
        actual
    );
    amazon_mock.assert();
    retrieve_by_name_mock.assert();
    mock_discord.assert();
}

#[test]
fn test_error_updating_dns() {
    let mut amazon = mockito::Server::new();
    let mut porkbun = mockito::Server::new();
    let mut discord = mockito::Server::new();

    let amazon_mock = amazon.mock("GET", "/")
        .with_status(200)
        .with_body("10.0.1.1")
        .create();

    let domain_config = ConfigDomainsStruct {
        domain: String::from("domain.tld"),
        dns_entry_type: String::from("A"),
        subdomain: String::from("subdomain")
    };

    let config = Config {
        secretkey: String::from("secretkey"),
        apikey: String::from("apikey"),
        discord_webhook_url: String::from(discord.url()),
        check_ip_providers: Vec::from([amazon.url()]),
        domains: Vec::from([domain_config]),
        porkbun_base_url: String::from(porkbun.url())
    };

    let retrieve_by_name_mock = porkbun.mock(
        "POST",
        &*format!(
            "/retrieveByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "SUCCESS", "records": [{"id": "12345", "name": "subdomain.domain.tld", "type": "A", "content": "10.0.1.2", "ttl": "600", "prio": "0", "notes": ""}]}"#)
    .create();

    let update_dns_mock = porkbun.mock(
        "POST",
        &*format!(
            "/editByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "error"}"#)
    .create();

    let mock_discord = discord.mock("POST", "/").with_status(200).create();

    let actual = update_dns(&config).unwrap();
    assert_eq!("Could not update DNS Entry. 10.0.1.2 -> 10.0.1.1", actual);
    amazon_mock.assert();
    retrieve_by_name_mock.assert();
    update_dns_mock.assert();
    mock_discord.assert();
}

#[test]
#[should_panic]
fn test_error_getting_dns() {
    let mut amazon = mockito::Server::new();
    let mut porkbun = mockito::Server::new();

    let amazon_mock = amazon.mock("GET", "/")
        .with_status(200)
        .with_body("10.0.1.1")
        .create();

    let domain_config = ConfigDomainsStruct {
        domain: String::from("domain.tld"),
        dns_entry_type: String::from("A"),
        subdomain: String::from("subdomain")
    };

    let config = Config {
        secretkey: String::from("secretkey"),
        apikey: String::from("apikey"),
        discord_webhook_url: String::from("discordurl"),
        check_ip_providers: Vec::from([amazon.url()]),
        domains: Vec::from([domain_config]),
        porkbun_base_url: String::from(porkbun.url())
    };

    let retrieve_by_name_mock = porkbun.mock(
        "POST",
        &*format!(
            "/retrieveByNameType/{}/{}/{}",
            config.domains[0].domain,
            config.domains[0].dns_entry_type,
            config.domains[0].subdomain
        )
    )
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"status": "error"}"#)
    .create();

    let actual = update_dns(&config).unwrap();
    assert_eq!("Could not update DNS Entry. 10.0.1.2 -> 10.0.1.1", actual);
    amazon_mock.assert();
    retrieve_by_name_mock.assert();
}