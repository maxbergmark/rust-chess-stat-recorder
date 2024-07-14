use std::{collections::HashSet, time::Duration};

use serde::Deserialize;
use serde_with::serde_as;

use crate::Result;

#[serde_with::serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub rerun_ip: [u8; 4],
    pub port: Option<u16>,
    pub years: HashSet<u32>,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(alias = "update_interval_seconds")]
    pub update_interval: Duration,
    pub output: Output,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Output {
    pub rare_moves: bool,
    pub data: bool,
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
update_interval_seconds = 5

[logs]
en_passant_mates = true
double_disambiguation_checkmates = true
double_disambiguation_capture_checkmates = true

[output]
rare_moves = true
data = false
"#,
        )
        .unwrap();

        assert_eq!(config.rerun_ip, [127, 0, 0, 1]);
        assert_eq!(config.port, None);
        assert_eq!(config.years.len(), 3);
        assert_eq!(config.years, [2013, 2014, 2015].into());
        assert_eq!(config.update_interval, Duration::from_secs(5));
    }
}
