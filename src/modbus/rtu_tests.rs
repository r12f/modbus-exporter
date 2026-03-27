use super::RtuClient;
use crate::modbus::ModbusClient;

#[test]
fn test_rtu_client_new_not_connected() {
    let builder = tokio_serial::new("/dev/null", 9600);
    let client = RtuClient::new(builder, 1);
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_rtu_client_read_without_connect_fails() {
    let builder = tokio_serial::new("/dev/null", 9600);
    let mut client = RtuClient::new(builder, 1);
    let result = client.read_holding_registers(0, 1).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not connected"));
}
