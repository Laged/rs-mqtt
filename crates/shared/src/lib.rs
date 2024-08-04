use config::{Config, ConfigError, File, FileFormat};
pub use rumqttd::Config as BrokerConfig;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "config/"]
struct Asset;

pub struct SharedConfig;

impl SharedConfig {
    fn new(file_name: &str) -> Result<Config, ConfigError> {
        let config_file =
            Asset::get(file_name).expect("Configuration file not found in embedded assets");
        let config_str = std::str::from_utf8(config_file.data.as_ref())
            .expect("Configuration file is not valid UTF-8");

        let settings = Config::builder()
            .add_source(File::from_str(config_str, FileFormat::Toml))
            .build()?;

        Ok(settings)
    }

    pub fn broker() -> Result<BrokerConfig, ConfigError> {
        let config = SharedConfig::new("broker.toml")?;
        config.try_deserialize()
    }

    pub fn client() -> Result<Config, ConfigError> {
        SharedConfig::new("client.toml")
    }
}
