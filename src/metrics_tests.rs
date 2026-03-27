use super::*;
use std::collections::BTreeMap;
use std::time::Instant;

fn make_metric(name: &str, value: f64, metric_type: MetricType) -> MetricValue {
    MetricValue {
        value,
        metric_type,
        labels: BTreeMap::new(),
        description: name.to_string(),
        unit: String::new(),
        updated_at: Instant::now(),
    }
}

#[test]
fn test_publish_and_read() {
    let store = MetricStore::new();
    let global = BTreeMap::new();
    let coll_labels = BTreeMap::new();

    store.publish(
        "collector1",
        vec![make_metric("temp", 22.5, MetricType::Gauge)],
        &global,
        &coll_labels,
    );

    let all = store.all_metrics();
    assert_eq!(all.len(), 1);
    assert!((all[0].value - 22.5).abs() < f64::EPSILON);
    assert_eq!(all[0].labels.get("collector").unwrap(), "collector1");
}

#[test]
fn test_cache_overwrite() {
    let store = MetricStore::new();
    let g = BTreeMap::new();
    let c = BTreeMap::new();

    store.publish("c1", vec![make_metric("a", 1.0, MetricType::Gauge)], &g, &c);
    store.publish("c1", vec![make_metric("b", 2.0, MetricType::Gauge)], &g, &c);

    let all = store.all_metrics();
    assert_eq!(all.len(), 1);
    assert!((all[0].value - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_label_merging_precedence() {
    let store = MetricStore::new();

    let mut global = BTreeMap::new();
    global.insert("env".to_string(), "prod".to_string());
    global.insert("region".to_string(), "us-east".to_string());

    let mut coll_labels = BTreeMap::new();
    coll_labels.insert("region".to_string(), "eu-west".to_string());

    let mut metric = make_metric("voltage", 230.0, MetricType::Gauge);
    metric
        .labels
        .insert("region".to_string(), "ap-south".to_string());
    metric.unit = "V".to_string();

    store.publish("plc1", vec![metric], &global, &coll_labels);

    let all = store.all_metrics();
    assert_eq!(all.len(), 1);
    let labels = &all[0].labels;
    assert_eq!(labels.get("region").unwrap(), "ap-south");
    assert_eq!(labels.get("env").unwrap(), "prod");
    assert_eq!(labels.get("collector").unwrap(), "plc1");
    assert_eq!(labels.get("unit").unwrap(), "V");
}

#[test]
fn test_gauge_vs_counter() {
    let store = MetricStore::new();
    let g = BTreeMap::new();
    let c = BTreeMap::new();

    store.publish(
        "c1",
        vec![
            make_metric("temperature", 25.0, MetricType::Gauge),
            make_metric("total_energy", 1000.0, MetricType::Counter),
        ],
        &g,
        &c,
    );

    let all = store.all_metrics();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].metric_type, MetricType::Gauge);
    assert_eq!(all[1].metric_type, MetricType::Counter);
}

#[test]
fn test_multiple_collectors() {
    let store = MetricStore::new();
    let g = BTreeMap::new();
    let c = BTreeMap::new();

    store.publish("c1", vec![make_metric("a", 1.0, MetricType::Gauge)], &g, &c);
    store.publish("c2", vec![make_metric("b", 2.0, MetricType::Gauge)], &g, &c);

    assert_eq!(store.collector_count(), 2);
    assert_eq!(store.all_metrics().len(), 2);
    assert_eq!(store.metrics_for("c1").len(), 1);
    assert_eq!(store.metrics_for("c2").len(), 1);
    assert!(store.metrics_for("c3").is_empty());
}

#[test]
fn test_concurrent_reads() {
    let store = MetricStore::new();
    let g = BTreeMap::new();
    let c = BTreeMap::new();
    store.publish(
        "c1",
        vec![make_metric("x", 42.0, MetricType::Gauge)],
        &g,
        &c,
    );

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let s = store.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let metrics = s.all_metrics();
                    assert!(!metrics.is_empty());
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_unit_label_omitted_when_empty() {
    let store = MetricStore::new();
    let g = BTreeMap::new();
    let c = BTreeMap::new();

    store.publish(
        "c1",
        vec![make_metric("temp", 1.0, MetricType::Gauge)],
        &g,
        &c,
    );
    let labels = &store.all_metrics()[0].labels;
    assert!(!labels.contains_key("unit"));
}
