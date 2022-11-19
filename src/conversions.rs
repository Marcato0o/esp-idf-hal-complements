use esp_idf_sys::twai_message_t;
use log::info;
use messages::CANMessage;

/// Converts a pedals position in percentage into an erpm value.
pub trait IntoErpm {
    fn into_erpm(self) -> i32;
}

impl IntoErpm for f32 {
    fn into_erpm(self) -> i32 {
        let erpm ;
        if self < 20.0 {
            erpm = 0;
        } else { erpm = (self * 2.0) as i32 }

        erpm as i32
    }
}

pub trait IntoPercentage {
    fn into_percentage(self, min: u32, max: u32, error: u32) -> Result<f32, ()>;
}

impl IntoPercentage for u16 {
    fn into_percentage(self, min: u32, max: u32, error: u32) -> Result<f32, ()> {
        let min_f = min as f32;
        let max_f = max as f32;
        let voltage_f = self as f32;
        let p = ((voltage_f - min_f) * (100 as f32)) / (max_f - min_f);

        if p < - (error as f32) {
            Err( () )
        } else if p > (100 + error) as f32 {
            Err( () )
        } else {
            Ok( p.clamp(0.0, 100.0) )
        }
    }
}

pub trait IntoCANMessage {
    fn into_can_message(self) -> CANMessage;
}

impl IntoCANMessage for twai_message_t {
    /// Converts a received message in a CANMessage enum variant.
    fn into_can_message(self) -> CANMessage {
        // Use messages crate to match the received message.
        let vec = self.data.to_vec();
        let slice = &vec[0 as usize..self.data_length_code as usize]; // We need this because `new` function expect a certain length, depending on the message, but rx_message.data has always length 8.
        let received_message = CANMessage::new(self.identifier, slice).expect("Cannot read message.");
        let id = received_message.get_id();
        info!("Received message info: id: {} length: {}", id, CANMessage::get_contents_length(id));
        received_message
    }
}