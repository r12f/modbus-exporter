use anyhow::{Context, Result};
use tokio_modbus::client::{rtu, Context as ModbusContext, Reader};
use tokio_modbus::Slave;
use tokio_serial::SerialPortBuilder;

use super::ModbusClient;

/// Modbus RTU (serial) client.
pub struct RtuClient {
    builder: SerialPortBuilder,
    slave_id: u8,
    context: Option<ModbusContext>,
}

impl RtuClient {
    /// Create a new RTU client (does not connect yet).
    ///
    /// `builder` is a [`tokio_serial::SerialPortBuilder`] configured with the
    /// desired device path, baud rate, data bits, stop bits, and parity.
    pub fn new(builder: SerialPortBuilder, slave_id: u8) -> Self {
        Self {
            builder,
            slave_id,
            context: None,
        }
    }
}

impl ModbusClient for RtuClient {
    async fn connect(&mut self) -> Result<()> {
        let port = tokio_serial::SerialStream::open(&self.builder)
            .context("failed to open serial port")?;
        let ctx = rtu::attach_slave(port, Slave(self.slave_id));
        self.context = Some(ctx);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.context.is_some()
    }

    async fn read_holding_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        let ctx = self.context.as_mut().context("not connected")?;
        let data = ctx
            .read_holding_registers(addr, count)
            .await?
            .context("empty response")?;
        Ok(data)
    }

    async fn read_input_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        let ctx = self.context.as_mut().context("not connected")?;
        let data = ctx
            .read_input_registers(addr, count)
            .await?
            .context("empty response")?;
        Ok(data)
    }

    async fn read_coils(&mut self, addr: u16, count: u16) -> Result<Vec<bool>> {
        let ctx = self.context.as_mut().context("not connected")?;
        let data = ctx
            .read_coils(addr, count)
            .await?
            .context("empty response")?;
        Ok(data)
    }

    async fn read_discrete_inputs(&mut self, addr: u16, count: u16) -> Result<Vec<bool>> {
        let ctx = self.context.as_mut().context("not connected")?;
        let data = ctx
            .read_discrete_inputs(addr, count)
            .await?
            .context("empty response")?;
        Ok(data)
    }
}

#[cfg(test)]
#[path = "rtu_tests.rs"]
mod rtu_tests;
