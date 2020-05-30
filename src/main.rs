use futures::future::try_join_all;
use reqwest::Client;
use semver::Version;
use semver::VersionReq;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pulp_plugins = [
        "galaxy-ng",
        "pulp-ansible",
        "pulp-certguard",
        "pulp-container",
        "pulp-cookbook",
        "pulp-deb",
        "pulp-file",
        "pulp-gem",
        "pulp-maven",
        "pulp-npm",
        "pulp-python",
        "pulp-rpm",
    ];

    let client = Client::builder().build()?;
    let mut data_plugins = Vec::new();

    for plugin in pulp_plugins.iter() {
        let data = get_pypi_data(&client, plugin);
        data_plugins.push(data);
    }

    let results = try_join_all(data_plugins).await?;

    let pulpcore_json: Value = get_pypi_data(&client, "pulpcore").await?;
    let pulpcore_version = pulpcore_json["info"]["version"].as_str().unwrap();

    println!("Lastest pulpcore version: {}", pulpcore_version);
    print_compatible_plugins(pulpcore_version, results);

    Ok(())
}

async fn get_pypi_data(client: &Client, plugin: &str) -> Result<Value, Box<dyn Error>> {
    let pypi_root = String::from("https://pypi.org/pypi/");
    let address = format!("{}{}/json", pypi_root, plugin);
    let result = client.get(&address).send().await?.json().await?;
    Ok(result)
}

fn print_compatible_plugins(pulpcore_version: &str, plugins: Vec<Value>) {
    for pypi_json in plugins {
        let name = pypi_json["info"]["name"].as_str().unwrap();
        let plugin_version = pypi_json["info"]["version"].as_str().unwrap();
        let requires_dist = pypi_json["info"]["requires_dist"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .filter(|l| l.contains("pulpcore"))
            .last()
            .unwrap();
        let pulpcore_requirement = requires_dist
            .as_str()
            .split('(')
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .map(|i| i.replace("~=", "~"))
            .unwrap();

        println!(
            "{}-{} requirement: {} || pulpcore-{} matches the requirement: {}",
            name,
            plugin_version,
            requires_dist,
            pulpcore_version,
            check_semver(&pulpcore_requirement.as_str(), &pulpcore_version),
        );
    }
}

fn check_semver(requirement: &str, version: &str) -> bool {
    let default_req = VersionReq::parse("<3.0.1").unwrap();
    let r = VersionReq::parse(requirement).unwrap_or(default_req);
    let v = Version::parse(version).unwrap();
    r.matches(&v)
}
