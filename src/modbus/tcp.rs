use std::net::SocketAddr;

use anyhow::{Context, Result};
use tokio_modbus::client::{tcp, Context as ModbusContext, Reader};
use tokio_modbus::Slave;

use super::ModbusClient;

/// Modbus TCP client.
pub struct TcpClient {
    endpoint: SocketAddr,
    slave_id: u8,
    context: Option<ModbusContext>,
}

impl TcpClient {
    /// Create a new TCP client (does not connect yet).
    pub fn new(endpoint: SocketAddr, slave_id: u8) -> Self {
        Self {
            endpoint,
            slave_id,
            context: None,
        }
    }
}

impl ModbusClient for TcpClient {
    async fn connect(&mut self) -> Result<()> {
        let ctx = tcp::connect_slave(self.endpoint, Slave(self.slave_id))
            .await
            .with_context(|| format!("failed to connect to {}", self.endpoint))?;
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
#[path = "tcp_tests.rs"]
mod tcp_tests;
