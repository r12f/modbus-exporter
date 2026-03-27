mod logging;

use logging::{init_logging, LoggingConfig};

fn main() {
    let config = LoggingConfig::default();
    if let Err(e) = init_logging(&config) {
        eprintln!("failed to initialize logging: {e}");
    }
    println!("otel-modbus-exporter starting...");
}
