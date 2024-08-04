use anyhow::{Context, Result};
use rumqttc::Client;
use shared::{ClientConfig, SharedConfig};
use std::thread;
use std::time::Duration;

struct ClientApp {
    config: ClientConfig,
}

impl ClientApp {
    fn new() -> Result<Self> {
        let config = SharedConfig::client().context("Failed to load client configuration")?;
        Ok(Self { config })
    }

    fn run(self) -> Result<()> {
        let mqtt_options = self.config.mqtt_options.clone();
        let (client, mut connection) = Client::new(mqtt_options, 10);

        client
            .subscribe(&self.config.subscribe_topic, self.config.qos)
            .context("Failed to subscribe to topic")?;

        let publish_topic = self.config.publish_topic.clone();
        let qos = self.config.qos;
        let publish_count = self.config.publish_count;
        let publish_interval_ms = self.config.publish_interval_ms;

        thread::spawn(move || {
            for i in 0..publish_count {
                let payload: Vec<u8> = (0..i).map(|_| i as u8).collect();
                client
                    .publish(&publish_topic, qos, false, payload)
                    .expect("Failed to publish message");
                thread::sleep(Duration::from_millis(publish_interval_ms));
            }
        });

        for (i, notification) in connection.iter().enumerate() {
            println!("Notification = {:?} ({:?})", notification, i);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let app = ClientApp::new()?;
    app.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_app_new() {
        let result = ClientApp::new();
        assert!(
            result.is_ok(),
            "Failed to create ClientApp: {:?}",
            result.err()
        );
    }
}
