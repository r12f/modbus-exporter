//! E2E integration test: SPI via spidev loopback → bus-exporter pull → JSON validation.
//!
//! Uses the shared test harness from `tests/common/mod.rs` for config generation,
//! pull execution, and validation. Requires a real spidev device (e.g. `/dev/spidev0.0`)
//! with MOSI connected to MISO for loopback, or a virtual spidev.
//!
//! **Requirements:** `/dev/spidev*` device accessible (typically requires root).
//! The test is marked `#[ignore]` and skips gracefully when no spidev device is found.

#[allow(dead_code)]
mod common;

use common::{ConnectionParams, TestFixtures, TestMetric};
use std::fs;

// ── SPI-specific test fixtures ────────────────────────────────────────

/// SPI test fixtures using simple register reads.
/// In SPI, register addresses are protocol-defined byte offsets.
fn spi_fixtures() -> TestFixtures {
    TestFixtures {
        metrics: vec![
            // u16 big-endian at address 0x00: raw=1500, scale=0.1 → 150.0
            TestMetric {
                name: "voltage",
                description: "SPI voltage sensor",
                metric_type: "gauge",
                register_type: "",
                address: 0x00,
                data_type: "u16",
                byte_order: "big_endian",
                scale: 0.1,
                offset: 0.0,
                unit: "V",
                raw_registers: vec![1500],
                expected_value: 150.0,
            },
            // u16 big-endian at address 0x02: raw=250, scale=0.1, offset=0 → 25.0
            TestMetric {
                name: "temperature",
                description: "SPI temperature sensor",
                metric_type: "gauge",
                register_type: "",
                address: 0x02,
                data_type: "u16",
                byte_order: "big_endian",
                scale: 0.1,
                offset: 0.0,
                unit: "C",
                raw_registers: vec![250],
                expected_value: 25.0,
            },
        ],
    }
}

// ── SPI device discovery ──────────────────────────────────────────────

/// Find the first available spidev device path (e.g., `/dev/spidev0.0`).
fn find_spidev() -> Option<String> {
    let dev_dir = fs::read_dir("/dev").ok()?;
    let mut spi_devices: Vec<String> = dev_dir
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_str()?.to_string();
            if name.starts_with("spidev") {
                Some(format!("/dev/{}", name))
            } else {
                None
            }
        })
        .collect();
    spi_devices.sort();
    spi_devices.into_iter().next()
}

// ── Test ──────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore] // Requires spidev device (typically root + hardware or virtual spidev)
async fn e2e_spi_pull() {
    // 1. Find an available spidev device
    let spi_device = match find_spidev() {
        Some(dev) => dev,
        None => {
            eprintln!("Skipping SPI e2e test: no /dev/spidev* device found");
            return;
        }
    };

    eprintln!("Using SPI device: {}", spi_device);

    let fixtures = spi_fixtures();

    // 2. Run shared e2e workflow
    //
    // Note: Without real hardware or a loopback connection (MOSI→MISO),
    // the pull will execute but metrics may return errors or zero values.
    // The test validates that:
    //   - Config generation with SPI protocol works correctly
    //   - bus-exporter can parse and attempt SPI communication
    //   - The pull command completes (exit code handling)
    //
    // For full validation with loopback, wire MOSI to MISO on the SPI bus.
    let connection = ConnectionParams::Spi {
        device: spi_device,
        speed_hz: 1_000_000,
        mode: 0,
        bits_per_word: 8,
    };

    // Use the shared workflow — on real hardware with loopback this validates end-to-end.
    // Without hardware, we at least validate config generation and pull execution structure.
    common::run_e2e_workflow("spi_test_device", &connection, &fixtures).await;
}

/// Validate that SPI config generation produces correct YAML.
#[test]
fn spi_config_generation() {
    let tmp = tempfile::tempdir().unwrap();
    let fixtures = spi_fixtures();
    let connection = ConnectionParams::Spi {
        device: "/dev/spidev0.0".to_string(),
        speed_hz: 1_000_000,
        mode: 0,
        bits_per_word: 8,
    };

    let config_path =
        common::generate_config(tmp.path(), "spi_config_test", &connection, &fixtures);
    let config = fs::read_to_string(&config_path).unwrap();

    // Verify SPI protocol fields are present
    assert!(config.contains("type: spi"), "config missing 'type: spi'");
    assert!(
        config.contains("device: \"/dev/spidev0.0\""),
        "config missing device"
    );
    assert!(
        config.contains("speed_hz: 1000000"),
        "config missing speed_hz"
    );
    assert!(config.contains("mode: 0"), "config missing mode");
    assert!(
        config.contains("bits_per_word: 8"),
        "config missing bits_per_word"
    );

    // Verify no slave_id (SPI doesn't use it)
    assert!(
        !config.contains("slave_id"),
        "SPI config should not contain slave_id"
    );
}
