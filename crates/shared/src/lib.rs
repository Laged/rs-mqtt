use anyhow::{Context, Result};
use config::{Config, File, FileFormat};
use rumqttc::{MqttOptions, QoS};
use rust_embed::RustEmbed;
use serde::de::DeserializeOwned;

#[derive(RustEmbed)]
#[folder = "config/"]
struct Asset;

const CONFIG_FILE: &str = "mqtt.toml";

pub struct SharedConfig;

pub struct ClientConfig {
    pub mqtt_options: MqttOptions,
    pub subscribe_topic: String,
    pub publish_topic: String,
    pub qos: QoS,
    pub publish_count: usize,
    pub publish_interval_ms: u64,
}

impl SharedConfig {
    fn new() -> Result<Config> {
        let config_file =
            Asset::get(CONFIG_FILE).context("Configuration file not found in embedded assets")?;
        let config_str = std::str::from_utf8(config_file.data.as_ref())
            .context("Configuration file is not valid UTF-8")?;

        let settings = Config::builder()
            .add_source(File::from_str(config_str, FileFormat::Toml))
            .build()
            .context("Failed to build config")?;

        Ok(settings)
    }

    fn get<T: DeserializeOwned + Clone>(config: &Config, key: &str) -> Result<T> {
        config
            .get::<T>(key)
            .with_context(|| format!("Failed to get {}", key))
    }

    pub fn broker() -> Result<rumqttd::Config> {
        let config = SharedConfig::new()?;
        config
            .try_deserialize()
            .context("Failed to deserialize broker config")
    }

    pub fn client() -> Result<ClientConfig> {
        let config = SharedConfig::new()?;

        let client_id: String = SharedConfig::get(&config, "client.client_id")?;
        let listen: String = SharedConfig::get(&config, "v4.1.listen")?;
        let mut parts = listen.split(':');
        let host = parts
            .next()
            .context("Failed to parse host from listen address")?
            .to_string();
        let port: u16 = parts
            .next()
            .context("Failed to parse port from listen address")?
            .parse()
            .context("Failed to parse port")?;
        let keep_alive_seconds: u64 =
            SharedConfig::get::<i64>(&config, "client.keep_alive_seconds")? as u64;

        let mqtt_options = {
            let mut options = MqttOptions::new(client_id, host, port);
            options.set_keep_alive(std::time::Duration::from_secs(keep_alive_seconds));
            options
        };

        let subscribe_topic = SharedConfig::get(&config, "client.subscribe_topic")
            .unwrap_or_else(|_| "hello/rumqtt".to_string());
        let publish_topic = SharedConfig::get(&config, "client.publish_topic")
            .unwrap_or_else(|_| "hello/rumqtt".to_string());
        let qos = match SharedConfig::get::<i64>(&config, "client.qos").unwrap_or(1) {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        };
        let publish_count: usize =
            SharedConfig::get::<i64>(&config, "client.publish_count").unwrap_or(10) as usize;
        let publish_interval_ms: u64 =
            SharedConfig::get::<i64>(&config, "client.publish_interval_ms").unwrap_or(100) as u64;

        Ok(ClientConfig {
            mqtt_options,
            subscribe_topic,
            publish_topic,
            qos,
            publish_count,
            publish_interval_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_load_config() {
        let config = SharedConfig::new();
        assert!(
            config.is_ok(),
            "Failed to load configuration: {:?}",
            config.err()
        );
    }

    #[test]
    fn test_broker_config() {
        let broker_config = SharedConfig::broker();
        assert!(
            broker_config.is_ok(),
            "Failed to load broker configuration: {:?}",
            broker_config.err()
        );
    }

    #[test]
    fn test_client_config() {
        let client_config = SharedConfig::client();
        assert!(
            client_config.is_ok(),
            "Failed to load client configuration: {:?}",
            client_config.err()
        );

        let client_config = client_config.unwrap();
        assert_eq!(
            client_config.mqtt_options.broker_address(),
            ("0.0.0.0".to_string(), 1883)
        );
        assert_eq!(client_config.subscribe_topic, "hello/rumqtt");
        assert_eq!(client_config.publish_topic, "hello/rumqtt");
        assert_eq!(client_config.qos, QoS::AtLeastOnce);
        assert_eq!(client_config.publish_count, 10);
        assert_eq!(client_config.publish_interval_ms, 100);
        assert_eq!(
            client_config.mqtt_options.keep_alive(),
            Duration::from_secs(5)
        );
    }
}
