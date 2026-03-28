pub mod mqtt;
pub mod otlp;
pub mod prometheus;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::{ExportersConfig, MetricConfig};

/// Common trait for all metric exporters.
///
/// Each implementation receives metric configs and cached read results,
/// then formats and sends them in its own protocol-specific way.
#[async_trait]
pub trait MetricExporter: Send {
    /// Export cached metric results.
    async fn export(
        &mut self,
        metrics: &[MetricConfig],
        results: &HashMap<String, Result<f64>>,
    ) -> Result<()>;

    /// Graceful shutdown.
    async fn shutdown(&mut self) -> Result<()>;
}

/// Create the appropriate exporter(s) from the top-level exporter config.
///
/// Returns a `Vec` because multiple exporters can be enabled simultaneously.
pub fn create_exporters(config: &ExportersConfig) -> Result<Vec<Box<dyn MetricExporter>>> {
    let mut exporters: Vec<Box<dyn MetricExporter>> = Vec::new();

    if let Some(ref otlp_cfg) = config.otlp {
        if otlp_cfg.enabled {
            exporters.push(Box::new(otlp::OtlpMetricExporter::new(otlp_cfg.clone())?));
        }
    }

    if let Some(ref prom_cfg) = config.prometheus {
        if prom_cfg.enabled {
            exporters.push(Box::new(prometheus::PrometheusMetricExporter::new(
                prom_cfg.clone(),
            )));
        }
    }

    if let Some(ref mqtt_cfg) = config.mqtt {
        if mqtt_cfg.enabled {
            exporters.push(Box::new(mqtt::MqttMetricExporter::new(mqtt_cfg.clone())?));
        }
    }

    Ok(exporters)
}
