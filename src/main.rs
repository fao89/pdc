use futures::future::try_join_all;
use reqwest::Client;
use semver::{Version, VersionReq};
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
    let spinner = Spinner::new(Spinners::Dots9, "Loading ...".into());

    let plugin_data_futures: Vec<_> = pulp_plugins
        .iter()
        .map(|plugin| get_pypi_data(&client, plugin))
        .collect();

    let mut plugins = try_join_all(plugin_data_futures)
        .await?
        .iter()
        .map(|m| PulpPlugin::from_metadata(m.clone()))
        .collect();

    let pulpcore_json = get_pypi_data(&client, "pulpcore").await?;
    spinner.stop();

    let mut pulpcore_releases = pulpcore_json["releases"]
        .as_object()
        .unwrap()
        .keys()
        .map(|x| x.to_string())
        .filter(|x| Version::parse(x).is_ok())
        .map(|x| Version::parse(&x).unwrap())
        .collect::<Vec<Version>>();

    pulpcore_releases.sort();

    for version in pulpcore_releases.iter().rev() {
        print_compatible_plugins(&version.to_string().trim(), &mut plugins);
    }

    Ok(())
}

#[derive(Debug)]
struct PulpPlugin {
    metadata: Value,
}

impl PulpPlugin {
    fn from_metadata(metadata: Value) -> Self {
        PulpPlugin { metadata }
    }

    fn compatible_with(&self, pulpcore_version: &str) -> bool {
        let requires = self.requires();
        let clean_requires = requires
            .as_str()
            .split('(')
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .map(|i| i.replace("~=", "~"))
            .map(|i| i.replace(",", " "))
            .unwrap();
        check_semver(&clean_requires, pulpcore_version)
    }

    fn version(&self) -> &str {
        self.metadata["info"]["version"].as_str().unwrap()
    }

    fn name(&self) -> &str {
        self.metadata["info"]["name"].as_str().unwrap()
    }

    fn requires(&self) -> String {
        self.metadata["info"]["requires_dist"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .filter(|l| l.contains("pulpcore"))
            .last()
            .unwrap()
    }
}

async fn get_pypi_data(client: &Client, package_name: &str) -> Result<Value, Box<dyn Error>> {
    let address = format!("https://pypi.org/pypi/{}/json", package_name);
    let result: Value = client.get(&address).send().await?.json().await?;
    Ok(result)
}

fn print_compatible_plugins(pulpcore_version: &str, plugins: &mut Vec<PulpPlugin>) {
    let has_compatibility = plugins.iter().any(|p| p.compatible_with(&pulpcore_version));

    if has_compatibility {
        println!("\nCompatible with pulpcore-{}", pulpcore_version);
    }
    plugins
        .iter()
        .filter(|p| p.compatible_with(pulpcore_version))
        .for_each(|p| {
            println!(
                " -> {}-{} requirement: {}",
                p.name(),
                p.version(),
                p.requires()
            )
        });
    plugins.retain(|p| !p.compatible_with(pulpcore_version))
}

fn check_semver(requirement: &str, version: &str) -> bool {
    let default_req = VersionReq::parse("<3.0.1").unwrap();
    let r = VersionReq::parse(requirement).unwrap_or(default_req);
    let v = Version::parse(version).unwrap();
    r.matches(&v)
}
