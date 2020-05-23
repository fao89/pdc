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

        Document::from_read(body.as_bytes())
            .unwrap()
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .for_each(|x| println!("{}", x));
    }

    Ok(())
}
