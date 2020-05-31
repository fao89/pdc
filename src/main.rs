use futures::future::try_join_all;
use reqwest::Client;
use semver::Version;
use semver::VersionReq;
use serde_json::Value;
use spinners::{Spinner, Spinners};
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

    let sp = Spinner::new(Spinners::Dots9, "Loading ...".into());

    for plugin in pulp_plugins.iter() {
        let data = get_pypi_data(&client, plugin);
        data_plugins.push(data);
    }

    let mut results = try_join_all(data_plugins).await?;

    let pulpcore_json: Value = get_pypi_data(&client, "pulpcore").await?;
    sp.stop();

    let pulpcore_releases = pulpcore_json["releases"].as_object().unwrap().keys();
    for version in pulpcore_releases.rev() {
        if version.contains("3.0.0") {
            // avoiding rc versions
            print_compatible_plugins(&"3.0.0", &mut results);
            break;
        }
        print_compatible_plugins(&version, &mut results);
    }

    Ok(())
}

async fn get_pypi_data(client: &Client, plugin: &str) -> Result<Value, Box<dyn Error>> {
    let pypi_root = String::from("https://pypi.org/pypi/");
    let address = format!("{}{}/json", pypi_root, plugin);
    let result = client.get(&address).send().await?.json().await?;
    Ok(result)
}

fn print_compatible_plugins(pulpcore_version: &str, plugins: &mut Vec<Value>) {
    let mut to_remove = Vec::new();
    let mut index = 0;
    let mut pulpcore_printed = false;
    for pypi_json in plugins.iter() {
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

        if check_semver(&pulpcore_requirement.as_str(), &pulpcore_version) {
            if !pulpcore_printed {
                println!("\nCompatible with pulpcore-{}", pulpcore_version);
            }
            pulpcore_printed = true;
            println!(
                " -> {}-{} requirement: {}",
                name, plugin_version, requires_dist,
            );
            to_remove.push(index);
        }
        index += 1;
    }
    for n in to_remove.iter().rev() {
        plugins.remove(*n);
    }
}

fn check_semver(requirement: &str, version: &str) -> bool {
    let default_req = VersionReq::parse("<3.0.1").unwrap();
    let r = VersionReq::parse(requirement).unwrap_or(default_req);
    let v = Version::parse(version).unwrap();
    r.matches(&v)
}
