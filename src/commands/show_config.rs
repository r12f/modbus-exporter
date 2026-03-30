use anyhow::Result;
use std::path::Path;

use crate::config::{self, OutputFormat};

/// Resolved config for serialization — shows the final merged state.
#[derive(serde::Serialize)]
struct ResolvedConfig {
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    global_labels: std::collections::HashMap<String, String>,
    collectors: Vec<ResolvedCollector>,
}

#[derive(serde::Serialize)]
struct ResolvedCollector {
    name: String,
    protocol: config::Protocol,
    #[serde(skip_serializing_if = "Option::is_none")]
    slave_id: Option<u8>,
    #[serde(with = "humantime_serde")]
    polling_interval: std::time::Duration,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    init_writes: Vec<config::WriteStep>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pre_poll: Vec<config::WriteStep>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    labels: std::collections::HashMap<String, String>,
    metrics: Vec<config::MetricConfig>,
}

impl From<config::CollectorConfig> for ResolvedCollector {
    fn from(c: config::CollectorConfig) -> Self {
        Self {
            name: c.name,
            protocol: c.protocol,
            slave_id: c.slave_id,
            polling_interval: c.polling_interval,
            init_writes: c.init_writes,
            pre_poll: c.pre_poll,
            labels: c.labels,
            metrics: c.metrics,
        }
    }
}

pub fn show_config_command(
    config_path: Option<&Path>,
    collector_filter: Option<&str>,
    metric_filter: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let path = config::find_config_file(config_path)?;
    let cfg = config::Config::load(&path)?;

    let mut collectors = cfg.collectors;

    // Apply collector filter (substring match)
    if let Some(pattern) = collector_filter {
        collectors.retain(|c| c.name.contains(pattern));
    }

    // Apply metric filter (substring match)
    if let Some(pattern) = metric_filter {
        for c in &mut collectors {
            c.metrics.retain(|m| m.name.contains(pattern));
        }
        collectors.retain(|c| !c.metrics.is_empty());
    }

    let resolved = ResolvedConfig {
        global_labels: cfg.global_labels,
        collectors: collectors
            .into_iter()
            .map(ResolvedCollector::from)
            .collect(),
    };

    match format {
        OutputFormat::Yaml => {
            let output = serde_yaml::to_string(&resolved)?;
            print!("{}", output);
        }
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(&resolved)?;
            println!("{}", output);
        }
    }

    Ok(())
}
