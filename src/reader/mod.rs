pub mod i2c;
pub mod i3c;
pub mod modbus;
pub mod spi;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::MetricConfig;

/// Unified interface for reading metrics from any bus protocol.
///
/// Each reader is configured with a set of metrics via [`set_metrics`](Self::set_metrics),
/// then [`read`](Self::read) returns all configured metric values in one call.
///
/// This trait requires `Send` but intentionally does **not** require `Sync`.
/// The underlying transport (e.g. `tokio_modbus::client::Context`) is `!Sync`,
/// so each reader is owned by a single task (`run_collector`) and accessed via
/// `&mut self` — no shared-reference concurrency is needed.
#[async_trait]
pub trait MetricReader: Send {
    /// Configure which metrics this reader should collect.
    fn set_metrics(&mut self, metrics: Vec<MetricConfig>);

    /// Establish the underlying connection/transport.
    async fn connect(&mut self) -> Result<()>;

    /// Close the underlying connection/transport.
    async fn disconnect(&mut self) -> Result<()>;

    /// Returns `true` when connected.
    fn is_connected(&self) -> bool;

    /// Read all configured metrics. Returns name → result mapping.
    async fn read(&mut self) -> HashMap<String, Result<f64>>;
}
