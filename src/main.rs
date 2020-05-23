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

        println!("Status: {}", res.status());

        let body = res.text().await?;

        println!("Body:\n\n{}", body);
    }

    Ok(())
}
