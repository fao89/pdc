mod pdc;
use self::pdc::PulpPlugin;
use self::pdc::{get_pypi_data, print_compatible_plugins};
use futures::future::try_join_all;
use reqwest::Client;
use semver::Version;
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
        .filter_map(|x| Version::parse(x).ok())
        .collect::<Vec<Version>>();

    pulpcore_releases.sort();

    for version in pulpcore_releases.iter().rev() {
        print_compatible_plugins(version.to_string().trim(), &mut plugins);
    }

    Ok(())
}
