use std::default::default;
use std::ptr;
use esp_idf_sys::*;
use esp_idf_sys::c_types::{c_int, c_void};
use log::info;

/// TODO: All the functions should return Result<Ok(), Err()>, `using esp_idf_sys` constants like `ESP_OK` or `ESP_ERR`, so we can use `esp!` or `esp_nofail!` macros for error handling.

pub const TIMER_FRQ: f64 = 80000000.0;
pub static mut EVENT_QUEUE: Option<QueueHandle_t> = None;
pub const QUEUE_TYPE_BASE: u8 = 0;
pub const ITEM_SIZE: u32 = 0; // we're not posting any actual data, just notifying.
pub const QUEUE_SIZE: u32 = 1;

#[link_section = ".iram0.text"]
pub unsafe extern "C" fn timer_interrupt(_: *mut c_void) -> bool {
    xQueueGiveFromISR(EVENT_QUEUE.unwrap(), ptr::null_mut());
    true
}
#[derive(Clone, Copy)]
pub enum Alarms {
    /// Alarms enabled.
    Enabled,
    /// Alarms not enabled.
    Disabled,
}
#[derive(Clone, Copy)]
pub enum Counter {
    /// Timer starts after `timer_init()`.
    Enabled,
    /// Timer starts when `timer_start()` is called.
    Disabled,
}

#[derive(Clone, Copy)]
pub enum Groups {
    /// Timer group 0.
    Group0,
    /// Timer group 1.
    Group1,
}

#[derive(Clone, Copy)]
pub enum Timers {
    /// Timer group 0.
    Timer0,
    /// Timer group 1.
    Timer1,
}

#[derive(Clone, Copy)]
pub enum CounterDirection {
    /// Timer group 0.
    Up,
    /// Timer group 1.
    Down,
}

#[derive(Clone, Copy)]
pub enum AutoReload {
    /// Alarms reset counter.
    Enabled,
    /// Alarm does not reset counter.
    Disabled,
}
#[derive(Clone, Copy)]
pub enum ClockSource {
    /// 80MHz clock source.
    APB,
    /// External Clock source(not used).
    XTAL,
}

#[derive(Clone, Copy)]
pub struct Timer {
    /// Set alarms in the current used timer.
    pub alarm_en: Alarms,
    /// Select when to start the timer.
    pub counter_en: Counter,
    /// Used to set increasing or decreasing counter.
    pub counter_dir: CounterDirection,
    /// Reset after alarm if enabled.
    pub auto_reload: AutoReload,
    /// Clock divider (value from 2 to 65'536) .
    pub divider: u64,
    /// Clock source
    pub clk_src: ClockSource,
}

impl Timer {
    pub fn init(group: Groups, timer: Timers, config: Self) {
        let config = timer_config_t {
            alarm_en: match config.alarm_en {
                Alarms::Enabled {} => 1,
                Alarms::Disabled {} => 0,
            },
            counter_en: match config.counter_en {
                Counter::Enabled => 1,
                Counter::Disabled => 0,
            },
            intr_type: default(),
            counter_dir: match config.counter_dir {
                CounterDirection::Up => 1,
                CounterDirection::Down => 0,
            },
            auto_reload: match config.auto_reload {
                AutoReload::Enabled => 1,
                AutoReload::Disabled => 0,
            },
            divider: config.divider as u32,
            clk_src: match config.clk_src {
                ClockSource::APB => 0,
                ClockSource::XTAL => 1,
            },
        };
        match group {
            Groups::Group0 => 0,
            Groups::Group1 => 1,
        };
        unsafe {
            timer_init(match group {
                Groups::Group0 => 0,
                Groups::Group1 => 1,
            }, match timer {
                Timers::Timer0 => 0,
                Timers::Timer1 => 1,
            }, Box::into_raw(Box::new(config)));
        }
    }

    pub fn set_alarm_value(group: Groups, timer: Timers, alarm_value_in_secs: f64, config: Self) {
        info!("XTAL clock is not supported, setting clock frequency at 80MHz");
        info!("divider: {}", config.divider);
        let alarm_value = alarm_value_in_secs * TIMER_FRQ / (config.divider as f64);
        info!("alarm value: {}", alarm_value);
        unsafe {
            timer_set_alarm_value(set_group(group), set_timer(timer), alarm_value as u64);
        }
    }

    pub fn set_counter_value(group: Groups, timer: Timers, load_value: u64) {
        unsafe {
            timer_set_counter_value(set_group(group), set_timer(timer), load_value);
        }
    }
    /// Enable interrupts
    ///
    /// Enables interrupts, creates queue where to store what the interrupt function returns and a lot of other great things.
    ///
    /// # Arguments
    ///
    /// * `group`: used to select `TIMER_GROUP1` or `TIMER_GROUP0`
    /// * `timer`: used to select the timer where to enable interrupts.
    ///
    /// returns: Queue where an interrupt posts a bool value when it triggers, this queue has to be an argument of `has triggered(queue: QueueHandle_t)`.
    ///
    /// # Errors
    ///
    /// Todo
    ///
    /// # Examples
    ///
    /// ```
    /// Todo
    /// ```
    pub fn enable_interrupts(_group: Groups, _timer: Timers) {
        const QUEUE_TYPE_BASE: u8 = 0;
        const ITEM_SIZE: u32 = 0; // we're not posting any actual data, just notifying
        const QUEUE_SIZE: u32 = 1;

        unsafe {
            EVENT_QUEUE = Some(xQueueGenericCreate(QUEUE_SIZE, ITEM_SIZE, QUEUE_TYPE_BASE));
            timer_isr_callback_add(0, 0, Some(timer_interrupt), ptr::null_mut(), ESP_INTR_FLAG_IRAM as c_int);
        }
    }

    pub fn start(group: Groups, timer: Timers) {
        unsafe {
            timer_start(set_group(group), set_timer(timer));
        }
    }

    pub fn has_triggered(queue: QueueHandle_t) -> bool {
        let res = unsafe { xQueueReceiveFromISR(queue, ptr::null_mut(), ptr::null_mut()) };

        match res {
            1 => true,
            _ => false,
        }
    }
}

fn set_group(group: Groups) -> timer_group_t {
    match group {
        Groups::Group0 => 0 as timer_group_t,
        Groups::Group1 => 1 as timer_group_t,
    }
}

fn set_timer(timer: Timers) -> timer_idx_t {
    match timer {
        Timers::Timer0 => 0 as timer_idx_t,
        Timers::Timer1 => 1 as timer_idx_t,
    }
}
