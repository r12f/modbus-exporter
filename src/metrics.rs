use dashmap::DashMap;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

/// Type of metric — determines exporter semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Gauge,
    Counter,
}

/// A single metric value with metadata.
#[derive(Debug, Clone)]
pub struct MetricValue {
    pub value: f64,
    pub metric_type: MetricType,
    pub labels: BTreeMap<String, String>,
    pub description: String,
    pub unit: String,
    pub updated_at: Instant,
}

/// Thread-safe store aggregating per-collector metric caches.
///
/// Collectors call [`publish`] to atomically replace their cache snapshot.
/// Exporters call [`all_metrics`] for a read-only flat list — they never
/// trigger Modbus calls.
#[derive(Debug, Clone)]
pub struct MetricStore {
    inner: Arc<DashMap<String, Vec<MetricValue>>>,
}

impl MetricStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    /// Atomically replace the cache for `collector_name`.
    ///
    /// `global_labels` and `collector_labels` are merged into each metric
    /// following the precedence order: global → collector → metric-level.
    pub fn publish(
        &self,
        collector_name: &str,
        metrics: Vec<MetricValue>,
        global_labels: &BTreeMap<String, String>,
        collector_labels: &BTreeMap<String, String>,
    ) {
        let merged: Vec<MetricValue> = metrics
            .into_iter()
            .map(|mut m| {
                let mut final_labels = global_labels.clone();
                for (k, v) in collector_labels {
                    final_labels.insert(k.clone(), v.clone());
                }
                for (k, v) in &m.labels {
                    final_labels.insert(k.clone(), v.clone());
                }
                final_labels.insert("collector".to_string(), collector_name.to_string());
                if !m.unit.is_empty() {
                    final_labels.insert("unit".to_string(), m.unit.clone());
                }
                m.labels = final_labels;
                m
            })
            .collect();

        self.inner.insert(collector_name.to_string(), merged);
    }

    /// Return a flat snapshot of all metrics across all collectors.
    pub fn all_metrics(&self) -> Vec<MetricValue> {
        let mut out = Vec::new();
        for entry in self.inner.iter() {
            out.extend(entry.value().clone());
        }
        out
    }

    /// Return metrics for a single collector.
    pub fn metrics_for(&self, collector_name: &str) -> Vec<MetricValue> {
        self.inner
            .get(collector_name)
            .map(|e| e.value().clone())
            .unwrap_or_default()
    }

    /// Number of collectors currently in the store.
    pub fn collector_count(&self) -> usize {
        self.inner.len()
    }
}

impl Default for MetricStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
