mod config;
mod error;
mod xmpp;
use fpush_push::FpushPush;

use log::{debug, error, info};
use std::sync::Arc;

/// init env_logger
fn setup_logging() {
    env_logger::init();
}

#[tokio::main]
async fn main() {
    setup_logging();

    // get settings name
    let args: Vec<String> = std::env::args().collect();
    let settings_filename = match args.get(1) {
        Some(f) => f,
        None => "./settings.json",
    };
    info!("Loading config file {}", settings_filename);

    let settings = match crate::config::load_config(settings_filename) {
        Ok(s) => {
            debug!("Config loaded");
            s
        }
        Err(e) => {
            panic!("Error loading config file: {}", e);
        }
    };

    let push_impl: Arc<FpushPush> = Arc::new(FpushPush::new(settings.push_modules()).await);

    loop {
        info!(
            "Opening connection to {}",
            settings.component().server_hostname()
        );
        // open component connection
        match crate::xmpp::init_component_connection(&settings).await {
            Err(e) => {
                error!("Could not connect to XMPP Server {}", e);
                info!(
                    "Waiting {} seconds before reconnecting",
                    settings.timeout().xmppconnection_error().as_secs()
                );
                tokio::time::sleep(*settings.timeout().xmppconnection_error()).await;
            }
            Ok(component) => {
                // open new messageLoop
                crate::xmpp::message_loop_main_thread(component, push_impl.clone()).await;
            }
        }
    }
}
