/*pub const UNKNOWN_STATE: u8 = 0;
pub const RUNNING_STATE: u8 = 1;
pub const ERROR_STATE: u8 = 2;
pub const RECOVERING_STATE: u8 = 3;
pub const WAITING_STATE: u8 = 4;
pub const STATUS_CRITICAL_COUNTER_VALUE: u32 = 128;

struct BoardStatus {
    powertrain: u8,
    dashboard: u8,
    telemetry: u8,
}

pub struct ErrorCounters {
    dashboard_ec: u32,
    powertrain_ec: u32,
    telemetry_ec: u32,
}

impl ErrorCounters {
    fn increase_counters(mut self, status: BoardStatus) {
        if self.telemetry_ec < STATUS_CRITICAL_COUNTER_VALUE {
            if status.telemetry == ERROR_STATE {
                self.telemetry_ec += 1;
            }
        }
        if self.powertrain_ec < STATUS_CRITICAL_COUNTER_VALUE {
            if status.dashboard == ERROR_STATE {
                self.dashboard_ec += 1;
            }
        }
        if self.powertrain_ec < STATUS_CRITICAL_COUNTER_VALUE {
            if status.powertrain == ERROR_STATE {
                self.dashboard_ec += 1;
            }
        }
    }
}*/
