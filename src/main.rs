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

    let pypi_json: Value = get_pypi_data(&client, "pulpcore").await?;
    let pulpcore_version = pypi_json["info"]["version"].as_str().unwrap();

    println!("Lastest pulpcore version: {}", pulpcore_version);

    for result in results {
        let pypi_json: Value = result;
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
        let default = VersionReq::parse(&format!(">{}", pulpcore_version)).unwrap();
        let r = VersionReq::parse(pulpcore_requirement.as_str()).unwrap_or(default);
        let v = Version::parse(pulpcore_version).unwrap();
        println!(
            "{}-{} requires pulpcore {} || pulpcore-{} matches the requirement: {}",
            name,
            plugin_version,
            pulpcore_requirement,
            pulpcore_version,
            r.matches(&v)
        );
    }

    Ok(())
}

async fn get_pypi_data(client: &Client, plugin: &str) -> Result<Value, Box<dyn Error>> {
    let pypi_root = String::from("https://pypi.org/pypi/");
    let address = format!("{}{}/json", pypi_root, plugin);
    let result = client.get(&address).send().await?.json().await?;
    Ok(result)
}
