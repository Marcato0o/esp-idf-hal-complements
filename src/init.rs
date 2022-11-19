use esp_idf_sys::*;
use log::info;
use crate::can::{Can, IdLen, Mode};
use crate::can::Filter::AcceptAll;

pub fn init() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    link_patches(); //expand maxsize to 20000 and enable buildscript.
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Logger initialised.");

}

pub fn can_init(tx: u8, rx: u8)  {
    let _setup = Can {
        tx,
        rx,
        id_length: IdLen::Standard,
        bit_timing: 1000000,
        mode: Mode::Normal,
        filter: AcceptAll
    }.install();
    Can::start();

}


