extern crate select;

use select::document::Document;
use select::predicate::Name;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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
        let res = reqwest::get(&format!("{}{}", pypi_root, (*plugin).to_string())).await?;
        assert!(res.status().is_success());

        let body = res.text().await?;

        let document = Document::from_read(body.as_bytes()).unwrap();
        let link = document
            .find(Name("a"))
            .last()
            .unwrap()
            .attr("href")
            .unwrap();

        let version = link.split("-").nth(1).unwrap();
        println!("{} - {}", plugin, version);

        let response = reqwest::get(link).await?;
        assert!(response.status().is_success());
    }

    Ok(())
}
