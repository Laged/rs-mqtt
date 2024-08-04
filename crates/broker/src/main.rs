use rumqttd::{Broker, Config as BrokerConfig, Notification};
use shared::SharedConfig;
use std::error::Error;
use std::thread;
use tracing_subscriber;

struct BrokerApp {
    config: BrokerConfig,
}

impl BrokerApp {
    fn new() -> Result<Self, Box<dyn Error>> {
        let config = SharedConfig::broker()?;
        Ok(Self { config })
    }

    fn run(self) -> Result<(), Box<dyn Error>> {
        let mut broker = Broker::new(self.config);
        let (mut link_tx, mut link_rx) = broker.link("singlenode")?;

        thread::spawn(move || {
            broker.start().unwrap();
        });

        link_tx.subscribe("#")?;

        let mut count = 0;
        loop {
            let notification = match link_rx.recv()? {
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
                    println!("{v:?}");
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let builder = tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .expect("initialized subscriber successfully");

    if let Ok(app) = BrokerApp::new() {
        app.run()?;
    } else {
        eprintln!("Failed to load broker configuration");
        // Try to recover or exit
    }

    Ok(())
}
