extern crate select;
extern crate tempfile;
extern crate zip;

use select::document::Document;
use select::predicate::Name;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pypi_root = String::from("https://pypi.org/simple/");
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

    for plugin in pulp_plugins.iter() {
        let res = reqwest::blocking::get(&format!("{}{}", pypi_root, (*plugin).to_string()))?;
        assert!(res.status().is_success());

        let body = res.text()?;

        let document = Document::from_read(body.as_bytes()).unwrap();
        let link = document
            .find(Name("a"))
            .last()
            .unwrap()
            .attr("href")
            .unwrap();

        let version = link.split('-').nth(1).unwrap();

        let mut tmpfile = tempfile::tempfile().unwrap();
        let mut response = reqwest::blocking::get(link)?;
        assert!(response.status().is_success());
        response.copy_to(&mut tmpfile)?;
        let mut zip = zip::ZipArchive::new(tmpfile).unwrap();
        let file_name = &format!(
            "{}-{}.dist-info/METADATA",
            (*plugin).to_string().replace("-", "_"),
            version
        );
        let mut metadata = zip.by_name(file_name).unwrap();
        let mut contents = String::new();
        metadata.read_to_string(&mut contents)?;
        contents
            .lines()
            .filter(|l| l.contains("Requires-Dist: pulpcore"))
            .for_each(|x| {
                println!(
                    "{}-{} {}",
                    plugin,
                    version,
                    x.replace("Requires-Dist", "requires")
                )
            });
    }

    Ok(())
}
