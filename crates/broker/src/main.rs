use anyhow::{Context, Result};
use rumqttd::{Broker, Config as BrokerConfig, Notification};
use shared::SharedConfig;
use std::thread;
use tracing_subscriber;

struct BrokerApp {
    config: BrokerConfig,
}

impl BrokerApp {
    fn new() -> Result<Self> {
        let config = SharedConfig::broker().context("Failed to load broker configuration")?;
        Ok(Self { config })
    }

    fn run(self) -> Result<()> {
        let mut broker = Broker::new(self.config);
        let (mut link_tx, mut link_rx) = broker
            .link("singlenode")
            .context("Failed to create broker link")?;

        thread::spawn(move || {
            if let Err(e) = broker.start() {
                eprintln!("Failed to start broker: {:?}", e);
            }
        });

        link_tx
            .subscribe("#")
            .context("Failed to subscribe to topic")?;

        let mut count = 0;
        loop {
            let notification = match link_rx.recv().context("Failed to receive notification")? {
                Some(v) => v,
                None => continue,
            };

            match notification {
                Notification::Forward(forward) => {
                    count += 1;
                    println!(
                        "Topic = {:?}, Count = {}, Payload = {} bytes",
                        forward.publish.topic,
                        count,
                        forward.publish.payload.len()
                    );
                }
                v => {
                    println!("{:?}", v);
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let builder = tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize subscriber: {:?}", e))?;

    let app = BrokerApp::new()?;
    app.run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broker_app_new() {
        let result = BrokerApp::new();
        assert!(
            result.is_ok(),
            "Failed to create BrokerApp: {:?}",
            result.err()
        );
    }
}
