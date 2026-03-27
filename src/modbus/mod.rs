pub mod rtu;
pub mod tcp;

#[cfg(test)]
mod mod_tests;

use anyhow::Result;

/// Common trait for Modbus clients (RTU and TCP).
pub trait ModbusClient: Send {
    /// Read holding registers (FC 03).
    fn read_holding_registers(
        &mut self,
        addr: u16,
        count: u16,
    ) -> impl std::future::Future<Output = Result<Vec<u16>>> + Send;

    /// Read input registers (FC 04).
    fn read_input_registers(
        &mut self,
        addr: u16,
        count: u16,
    ) -> impl std::future::Future<Output = Result<Vec<u16>>> + Send;

    /// Read coils (FC 01).
    fn read_coils(
        &mut self,
        addr: u16,
        count: u16,
    ) -> impl std::future::Future<Output = Result<Vec<bool>>> + Send;

    /// Read discrete inputs (FC 02).
    fn read_discrete_inputs(
        &mut self,
        addr: u16,
        count: u16,
    ) -> impl std::future::Future<Output = Result<Vec<bool>>> + Send;

    /// Establish the connection.
    fn connect(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Check if the client is currently connected.
    fn is_connected(&self) -> bool;
}
