#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReceiverSerial {
    //serial_port,
    //serial_port_watcher:,
    pub packet_is_empty: bool,
    pub received_packet_count: u32,
    pub error_packet_count: i32,
    pub packet_index: usize,
    pub start_time: u32,
}

impl Default for ReceiverSerial {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiverSerial {
    pub fn new() -> Self {
        Self { packet_is_empty: true, received_packet_count: 0, error_packet_count: 0, packet_index: 0, start_time: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_normal::<ReceiverSerial>();
    }
    #[test]
    fn new() {
        let receiver = ReceiverSerial::new();
        assert!(receiver.packet_is_empty);
    }
}
