use semver::Version;
use semver::VersionReq;
extern crate serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pypi_root = String::from("https://pypi.org/pypi/");
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

    let res = reqwest::blocking::get(&format!("{}pulpcore/json", pypi_root))?;
    assert!(res.status().is_success());

    let pypi_json: serde_json::Value = res.json()?;
    let pulpcore_version = pypi_json["info"]["version"].as_str().unwrap();
    println!("Lastest pulpcore version: {}", pulpcore_version);

    for plugin in pulp_plugins.iter() {
        let res = reqwest::blocking::get(&format!("{}{}/json", pypi_root, (*plugin).to_string()))?;
        assert!(res.status().is_success());

        let pypi_json: serde_json::Value = res.json()?;
        let plugin_version = pypi_json["info"]["version"].as_str().unwrap();
        let pulpcore_requirement = pypi_json["info"]["requires_dist"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .filter(|l| l.contains("pulpcore"))
            .last()
            .unwrap();
        let req = pulpcore_requirement
            .as_str()
            .split('(')
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .unwrap();
        let default = VersionReq::parse(&format!(">{}", pulpcore_version)).unwrap();
        let r = VersionReq::parse(req.replace("~=", "~").as_str()).unwrap_or(default);
        let v = Version::parse(pulpcore_version).unwrap();
        println!(
            "{}-{} requirement {} is compatible with pulpcore-{}: {}",
            plugin,
            plugin_version,
            pulpcore_requirement,
            pulpcore_version,
            r.matches(&v)
        );
    }

    Ok(())
}
