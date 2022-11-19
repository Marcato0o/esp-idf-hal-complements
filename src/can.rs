use esp_idf_sys::*;
use esp_idf_sys::c_types::c_int;
use esp_idf_sys::twai_general_config_t;
use log::info;
use messages::CANMessage;
pub enum IdLen {
    /// 11 bit id.
    Standard,
    /// 29 bit id.
    Extended,
}

pub enum Mode {
    /// Normal operating mode.
    Normal,
    /// No ACK mode(used for self testing).
    LoopBack,
    /// Tx disabled.
    ListenOnly,
}

pub enum Filter {
    AcceptAll,
    RejectAll,
    Range(u8, u8),
}

pub struct Can {
    /// GPIO pin used for CAN tx.
    pub tx: u8,
    /// GPIO pin used for CAN rx.
    pub rx: u8,
    /// Standard(11 bit id) or Extended(29 bit id).
    pub id_length: IdLen,
    /// Bit timing in bit/s (ex: 1Mbit/s -> 1000000).
    pub bit_timing: u32,
    /// LoopBack needed if testing.
    pub mode: Mode,
    /// Accepted id's (Range not supported).
    pub filter: Filter,
}

impl Can {
    /// Install the Can peripheral on the board with the current configuration.
    ///
    /// returns: none
    ///
    /// # Errors
    ///
    /// Could be wrapped around [`esp!`] or [`esp_nofail!`] macros to return a [`Result`].
    ///
    pub fn install(&self) {
        let general_config = twai_general_config_t {
            mode: match self.mode {
                Mode::Normal => twai_mode_t_TWAI_MODE_NORMAL,
                Mode::LoopBack => twai_mode_t_TWAI_MODE_NO_ACK,
                Mode::ListenOnly => twai_mode_t_TWAI_MODE_LISTEN_ONLY,
            },
            tx_io: self.tx as gpio_num_t,
            rx_io: self.rx as gpio_num_t,
            clkout_io: -1,
            bus_off_io: -1,
            tx_queue_len: 5,
            rx_queue_len: 5,
            alerts_enabled: TWAI_ALERT_NONE,
            clkout_divider: 0,
            intr_flags: ESP_INTR_FLAG_LEVEL1 as c_int,
        };
        let timing_config = twai_timing_config_t {
            brp: match self.bit_timing {
                1000000 => 4,
                500000 => 8,
                250000 => 16,
                1250000 => 32,
                _ => {
                    println!("Incorrect bit timing, setting it at 500 kbit/s.");
                    8
                }
            },                      // BaudRate PreScaler(brp): number that divides the TWAI controller clock(80MHz). Used to set the frequency of a time quanta. Calculated with the following formula:
            tseg_1: 15,             // Number of time quanta before the bit sample point.
            tseg_2: 4,              // Number of time quanta after the bit sample point.
            sjw: 3,                 // The Synchronization Jump Width is used to determine the maximum number of time quanta a single bit time can be lengthened/shortened for synchronization purposes. sjw can range from 1 to 4. (Always set at 3).
            triple_sampling: false, // Enable or disable triple sampling.
        };

        let filter_config = twai_filter_config_t {
            acceptance_code: 0,
            acceptance_mask: 0xFFFFFFFF,
            single_filter: true,
        };

        /*let filter_config = twai_filter_config_t {
            acceptance_code: match self.filter {
                Filter::AcceptAll => 0,
                Filter::RejectAll => 0,
                Filter::Range(_, _) => 0,
            },
            acceptance_mask: match self.filter {
                Filter::AcceptAll => match self.id_length  {
                    IdLen::Standard => 0x7FF,
                    IdLen::Extended => 0x1FFFFFF
                },
                Filter::RejectAll => 0,
                Filter::Range(_, _) => match self.id_length  {
                    IdLen::Standard => 0x7FF,
                    IdLen::Extended => 0x1FFFFFF,
                },
            },
            single_filter: true,
        };*/

        let id_len;

        match self.id_length {
            IdLen::Standard => id_len = String::from("Standard"),
            IdLen::Extended => id_len = String::from("Extended")
        }
        info!("CAN driver installed.\ntx: {}\nrx: {}\nid length: {}", self.tx, self.rx, id_len);

        unsafe {
            twai_driver_install(Box::into_raw(Box::new(general_config)),Box::into_raw(Box::new(timing_config)), Box::into_raw(Box::new(filter_config)));
        }
    }

    /// Create a new message instance to be sent.
    ///
    /// # Arguments
    ///
    /// * `can_id`: The CAN ID of the message.
    /// * `length`: The length of the message(from 1 to 8).
    /// * `data`: The contents of the message in an array.
    ///
    /// returns: [`twai_message_t`]
    ///
    /// # Errors
    ///
    /// Could be wrapped around [`esp!`] or [`esp_nofail!`] macros to return a [`Result`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo: examples
    /// ```
    pub fn create_message(content: CANMessage) -> twai_message_t {

        let mut buf = [0 as u8; 8];
        content.fill_can_buffer(&mut buf);
        let id = content.get_id();
        let length = CANMessage::get_contents_length(id);
        content.fill_can_buffer(&mut buf);

        let mut message_flags = twai_message_t__bindgen_ty_1__bindgen_ty_1::default();
        message_flags.set_extd(0);

        let tx_message = twai_message_t {
            __bindgen_anon_1: twai_message_t__bindgen_ty_1 {
                __bindgen_anon_1: message_flags,
            },
            identifier: id,
            data_length_code: length as u8,
            data: buf,
        };
        tx_message
    }

    /// Starts CAN peripheral with the setup provided by `install`. Panics if the returned value isn't ESP_OK.
    pub fn start() {
        unsafe {
            esp_nofail!(twai_start());
        }
        info!("CAN driver started.");
    }

    /// Transmit a CAN message using the parameters provided.
    pub fn transmit(tx_message: twai_message_t) {
        unsafe {
            esp_nofail!(twai_transmit(Box::into_raw(Box::new(tx_message)), 1));
        }
    }


    /// Receive a CAN message from the queue.
    pub fn receive(rx_message: twai_message_t) {
        unsafe {
            esp_nofail!(twai_receive(Box::into_raw(Box::new(rx_message)),100000));
        }
    }
    /// Non-blocking receive (see `receive`).
    pub fn lock_free_receive(rx_message: twai_message_t) {
        if CanInfo::get_status_info().msgs_to_rx > 0 {
            Can::receive(rx_message);
        }
    }

    /// Print tx CAN state if something went wrong.
    pub fn tx_errors_info() {
        let info = CanInfo::get_status_info();

        if info.state != 1 || info.tx_error_counter != 0 || info.tx_failed_count != 0 || info.msgs_to_tx != 0 {
            info!("State: {} {} {} {}", info.state, info.tx_error_counter, info.tx_failed_count, info.msgs_to_tx);
        }
    }

    /// Print tx CAN state if something went wrong.
    pub fn rx_errors_info() {
        let info = CanInfo::get_status_info();

        if info.state != 1 || info.rx_error_counter != 0 || info.rx_overrun_count != 0 || info.msgs_to_rx > 100 || info.rx_missed_count != 0 {
            info!("State: {} {} {} {} {}", info.state, info.rx_error_counter, info.rx_overrun_count, info.msgs_to_rx, info.rx_missed_count);
        }
    }
}

pub struct CanInfo {
    /// Current state of CAN controller (Stopped/Running/Bus-Off/Recovery).
    pub state: twai_state_t,
    /// Number of messages queued for transmission or awaiting transmission completion.
    pub msgs_to_tx: u32,
    /// Number of messages in RX queue waiting to be read.
    pub msgs_to_rx: u32,
    /// Current value of Transmit Error Counter.
    pub tx_error_counter: u32,
    /// Current value of Receive Error Counter.
    pub rx_error_counter: u32,
    /// Number of messages that failed transmissions.
    pub tx_failed_count: u32,
    /// Number of messages that were lost due to a full RX queue (or errata workaround if enabled).
    pub rx_missed_count: u32,
    /// Number of messages that were lost due to a RX FIFO overrun.
    pub rx_overrun_count: u32,
    /// Number of instances arbitration was lost.
    pub arb_lost_count: u32,
    /// Number of instances a bus error has occurred.
    pub bus_error_count: u32,
}

impl CanInfo {
    pub fn default() -> Self {
        let info = CanInfo {
            state: 0,
            msgs_to_tx: 0,
            msgs_to_rx: 0,
            tx_error_counter: 0,
            rx_error_counter: 0,
            tx_failed_count: 0,
            rx_missed_count: 0,
            rx_overrun_count: 0,
            arb_lost_count: 0,
            bus_error_count: 0,
        };
        info
    }

    pub fn get_status_info() -> CanInfo {
        let mut twai_info = twai_status_info_t::default();
        unsafe {
            esp_nofail!(twai_get_status_info(&mut twai_info));
        }
        let info = CanInfo {
            state: twai_info.state,
            msgs_to_tx: twai_info.msgs_to_tx,
            msgs_to_rx: twai_info.msgs_to_rx,
            tx_error_counter: twai_info.tx_error_counter,
            rx_error_counter: twai_info.rx_error_counter,
            tx_failed_count: twai_info.tx_failed_count,
            rx_missed_count: twai_info.rx_missed_count,
            rx_overrun_count: twai_info.rx_overrun_count,
            arb_lost_count: twai_info.arb_lost_count,
            bus_error_count: twai_info.bus_error_count
        };
        info
    }
}



