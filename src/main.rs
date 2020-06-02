use futures::future::try_join_all;
use reqwest::Client;
use semver::{Version, VersionReq};
use serde_json::Value;
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
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

    let spinner = Spinner::new(Spinners::Dots9, "Loading ...".into());

    for plugin in pulp_plugins.iter() {
        let data = get_pypi_data(&client, plugin);
        data_plugins.push(data);
    }

    let mut results = try_join_all(data_plugins).await?;

    let pulpcore_json = get_pypi_data(&client, "pulpcore").await?;
    spinner.stop();

    let pulpcore_releases = pulpcore_json["releases"].split(',');
    for version in pulpcore_releases.rev() {
        if version.contains("3.0.0") {
            // avoiding rc versions
            print_compatible_plugins(&"3.0.0", &mut results);
            break;
        }
        print_compatible_plugins(&version.trim(), &mut results);
    }

    Ok(())
}

async fn get_pypi_data(
    client: &Client,
    package: &str,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let pypi_root = String::from("https://pypi.org/pypi/");
    let address = format!("{}{}/json", pypi_root, package);
    let result: Value = client.get(&address).send().await?.json().await?;
    let mut pypi_data: HashMap<String, String> = HashMap::new();
    pypi_data.insert("name".to_string(), package.to_string());
    pypi_data.insert(
        "version".to_string(),
        result["info"]["version"].as_str().unwrap().to_string(),
    );
    pypi_data.insert(
        "releases".to_string(),
        result["releases"]
            .as_object()
            .unwrap()
            .keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", "),
    );
    if package != "pulpcore" {
        pypi_data.insert(
            "requires".to_string(),
            result["info"]["requires_dist"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i.to_string())
                .filter(|l| l.contains("pulpcore"))
                .last()
                .unwrap(),
        );
        pypi_data.insert(
            "clean_requires".to_string(),
            pypi_data["requires"]
                .as_str()
                .split('(')
                .nth(1)
                .unwrap()
                .split(')')
                .next()
                .map(|i| i.replace("~=", "~"))
                .unwrap(),
        );
    }
    Ok(pypi_data)
}

fn print_compatible_plugins(pulpcore_version: &str, plugins: &mut Vec<HashMap<String, String>>) {
    let has_compatibility = plugins
        .iter()
        .any(|x| check_semver(&x["clean_requires"].as_str(), &pulpcore_version));
    if has_compatibility {
        println!("\nCompatible with pulpcore-{}", pulpcore_version);
    }
    plugins
        .iter()
        .filter(|x| check_semver(&x["clean_requires"].as_str(), &pulpcore_version))
        .for_each(|x| {
            println!(
                " -> {}-{} requirement: {}",
                x["name"], x["version"], x["requires"]
            )
        });
    plugins.retain(|x| !check_semver(&x["clean_requires"].as_str(), &pulpcore_version))
}

fn check_semver(requirement: &str, version: &str) -> bool {
    let default_req = VersionReq::parse("<3.0.1").unwrap();
    let r = VersionReq::parse(requirement).unwrap_or(default_req);
    let v = Version::parse(version).unwrap();
    r.matches(&v)
}
