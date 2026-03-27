# Metrics Store Specification

## Overview

The metric store holds the latest values for all metrics and serves as the shared state between collectors (writers) and exporters (readers).

## Architecture

The metric store is a **read-only aggregation view** of all per-collector caches. Collectors are the sole producers; exporters are pure consumers. No exporter ever triggers a Modbus call.

```
Collectors (producers)          MetricStore           Exporters (consumers)
┌──────────┐                  ┌─────────────┐        ┌──────────────┐
│Collector 1│──publish()─────▶│             │◀──read──│ OTLP         │
│  [cache]  │                 │  Aggregated  │        └──────────────┘
├──────────┤                  │   Snapshots  │        ┌──────────────┐
│Collector 2│──publish()─────▶│             │◀──read──│ Prometheus   │
│  [cache]  │                 └─────────────┘        └──────────────┘
└──────────┘
```

## In-Memory Design

- A single `MetricStore` instance shared via `Arc<MetricStore>`.
- Internally uses `DashMap<String, HashMap<String, MetricValue>>` — outer key is collector name, inner map is that collector's latest cache snapshot.
- `publish(collector_name, cache)` replaces the entire entry for that collector atomically.
- Read methods iterate all collector entries to produce a flat list of metrics.

### MetricValue

```rust
struct MetricValue {
    value: f64,
    metric_type: MetricType, // Gauge or Counter
    labels: BTreeMap<String, String>,
    description: String,
    unit: String,
    updated_at: Instant,
}
```

## Label Merging Order

Labels are merged in this order (later wins on conflict):

1. **Global labels** — from `global_labels` in config
2. **Collector labels** — from `collectors[].labels`
3. **Metric-level labels** — automatically added:
   - `collector`: collector name
   - `unit`: metric unit (if non-empty)

## Gauge vs Counter Semantics

- **Gauge**: represents a point-in-time value. Each poll overwrites the previous value.
- **Counter**: represents a monotonically increasing total. Each poll overwrites with the latest reading from the device. The exporter is responsible for communicating counter semantics (OTLP cumulative temporality, Prometheus counter type).

## Thread Safety

- `DashMap` provides concurrent read/write without a global lock.
- Collectors write via `publish()` — each collector writes only to its own key.
- Exporters are **pure readers** — they call `store.all_metrics()` which iterates all collector snapshots. They never trigger Modbus calls or modify the store.
- No mutex contention between collectors since each writes to distinct keys.
