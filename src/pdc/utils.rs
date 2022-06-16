use super::PulpPlugin;
use reqwest::Client;
use semver::{Version, VersionReq};
use serde_json::Value;
use std::error::Error;

pub async fn get_pypi_data(client: &Client, package_name: &str) -> Result<Value, Box<dyn Error>> {
    let address = format!("https://pypi.org/pypi/{}/json", package_name);
    let result: Value = client.get(&address).send().await?.json().await?;
    Ok(result)
}

pub fn print_compatible_plugins(pulpcore_version: &str, plugins: &mut Vec<PulpPlugin>) {
    let has_compatibility = plugins.iter().any(|p| p.compatible_with(pulpcore_version));

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

pub fn check_semver(requirement: &str, version: &str) -> bool {
    let default_req = VersionReq::parse("<3.0.1").unwrap();
    let r = VersionReq::parse(requirement).unwrap_or(default_req);
    let v = Version::parse(version).unwrap();
    r.matches(&v)
}
