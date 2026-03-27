use super::ModbusClient;
use anyhow::Result;

/// A mock Modbus client for testing the trait interface.
struct MockClient {
    connected: bool,
    holding_regs: Vec<u16>,
    input_regs: Vec<u16>,
    coils: Vec<bool>,
    discrete_inputs: Vec<bool>,
}

impl MockClient {
    fn new() -> Self {
        Self {
            connected: false,
            holding_regs: vec![100, 200, 300],
            input_regs: vec![10, 20, 30],
            coils: vec![true, false, true],
            discrete_inputs: vec![false, true, false],
        }
    }
}

impl ModbusClient for MockClient {
    async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn read_holding_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        let start = addr as usize;
        let end = start + count as usize;
        Ok(self.holding_regs[start..end].to_vec())
    }

    async fn read_input_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        let start = addr as usize;
        let end = start + count as usize;
        Ok(self.input_regs[start..end].to_vec())
    }

    async fn read_coils(&mut self, addr: u16, count: u16) -> Result<Vec<bool>> {
        let start = addr as usize;
        let end = start + count as usize;
        Ok(self.coils[start..end].to_vec())
    }

    async fn read_discrete_inputs(&mut self, addr: u16, count: u16) -> Result<Vec<bool>> {
        let start = addr as usize;
        let end = start + count as usize;
        Ok(self.discrete_inputs[start..end].to_vec())
    }
}

#[tokio::test]
async fn test_mock_connect() {
    let mut client = MockClient::new();
    assert!(!client.is_connected());
    client.connect().await.unwrap();
    assert!(client.is_connected());
}

#[tokio::test]
async fn test_mock_read_holding_registers() {
    let mut client = MockClient::new();
    client.connect().await.unwrap();
    let regs = client.read_holding_registers(0, 2).await.unwrap();
    assert_eq!(regs, vec![100, 200]);
}

#[tokio::test]
async fn test_mock_read_input_registers() {
    let mut client = MockClient::new();
    client.connect().await.unwrap();
    let regs = client.read_input_registers(1, 2).await.unwrap();
    assert_eq!(regs, vec![20, 30]);
}

#[tokio::test]
async fn test_mock_read_coils() {
    let mut client = MockClient::new();
    client.connect().await.unwrap();
    let coils = client.read_coils(0, 3).await.unwrap();
    assert_eq!(coils, vec![true, false, true]);
}

#[tokio::test]
async fn test_mock_read_discrete_inputs() {
    let mut client = MockClient::new();
    client.connect().await.unwrap();
    let inputs = client.read_discrete_inputs(0, 2).await.unwrap();
    assert_eq!(inputs, vec![false, true]);
}
