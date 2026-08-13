/// The common interface for all RC protocols.
#[allow(unused)]
pub trait RxProtocol {
    //fn update(&mut self) -> Result<Option<Self::Frame>, Error>;
    //fn update(&mut self, tick_count_delta: u32);

    fn is_data_available(&self) -> bool;
    fn on_data_received_from_isr(&mut self, data: u8) -> bool;
    fn read_byte(&mut self) -> u8;

    fn channel_pwm(&self, channel_index: u8) -> u16;
}
