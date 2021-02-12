use super::check_semver;
use serde_json::Value;

#[derive(Debug)]
pub struct PulpPlugin {
    metadata: Value,
}

impl PulpPlugin {
    pub fn from_metadata(metadata: Value) -> Self {
        PulpPlugin { metadata }
    }

    pub fn compatible_with(&self, pulpcore_version: &str) -> bool {
        let requires = self.requires();
        let clean_requires = requires
            .as_str()
            .split('(')
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .map(|i| i.replace("~=", "~").replace(",", " "))
            .unwrap();
        check_semver(&clean_requires, pulpcore_version)
    }

    pub fn version(&self) -> &str {
        self.metadata["info"]["version"].as_str().unwrap()
    }

    pub fn name(&self) -> &str {
        self.metadata["info"]["name"].as_str().unwrap()
    }

    pub fn requires(&self) -> String {
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
