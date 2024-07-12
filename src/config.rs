use std::collections::HashSet;

use serde::Deserialize;

use crate::Result;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub rerun_ip: [u8; 4],
    pub port: Option<u16>,
    pub years: HashSet<u32>,
    pub output_data: bool,
    pub logs: Logs,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Logs {
    pub en_passant_mates: bool,
    pub double_disambiguation_checkmates: bool,
    pub double_disambiguation_capture_checkmates: bool,
}

impl Config {
    pub fn from_file() -> Result<Self> {
        let s = std::fs::read_to_string("config.toml")?;
        Ok(toml::from_str(&s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config: Config = toml::from_str(
            r#"
rerun_ip = [127, 0, 0, 1]
years = [2013, 2014, 2015]
output_data = true

[logs]
en_passant_mates = true
double_disambiguation_checkmates = true
double_disambiguation_capture_checkmates = true
 "#,
        )
        .unwrap();

        assert_eq!(config.rerun_ip, [127, 0, 0, 1]);
        assert_eq!(config.port, None);
        assert_eq!(config.years.len(), 3);
        assert_eq!(config.years, [2013, 2014, 2015].into());
    }
}
