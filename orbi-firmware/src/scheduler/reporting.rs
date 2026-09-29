use esp_println::println;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionState {
    Moving,
    Idle,
    Parked,
}

#[derive(Debug, Clone, Copy)]
pub struct ReportingPolicy {
    pub moving_interval_ms: u32,
    pub idle_interval_ms: u32,
    pub parked_interval_ms: u32,

    pub calibration_interval_ms: u32,

    pub moving_threshold_kmh: f64,
    pub parked_threshold_kmh: f64,
}

impl ReportingPolicy {
    pub const fn default() -> Self {
        Self {
            // Temporary GPS-only defaults.
            moving_interval_ms: 2_000,
            idle_interval_ms: 10_000,
            parked_interval_ms: 30_000,

            /*
             * Guided fuel calibration needs frequent physical KUM
             * observations even while the vehicle is stationary.
             *
             * Calibration mode therefore uses a fixed reporting cadence
             * rather than the normal motion-based adaptive interval.
             */
            calibration_interval_ms: 5_000,

            moving_threshold_kmh: 3.0,
            parked_threshold_kmh: 0.5,
        }
    }

    pub fn classify_speed_knots(&self, speed_knots: f64) -> MotionState {
        let speed_kmh = knots_to_kmh(speed_knots);

        if speed_kmh >= self.moving_threshold_kmh {
            MotionState::Moving
        } else if speed_kmh >= self.parked_threshold_kmh {
            MotionState::Idle
        } else {
            MotionState::Parked
        }
    }

    pub fn interval_for(&self, state: MotionState) -> u32 {
        match state {
            MotionState::Moving => self.moving_interval_ms,
            MotionState::Idle => self.idle_interval_ms,
            MotionState::Parked => self.parked_interval_ms,
        }
    }

    pub fn effective_interval_for(&self, state: MotionState, calibration_mode: bool) -> u32 {
        /*
         * Guided fuel calibration overrides the normal motion-based
         * reporting cadence.
         *
         * This does not change the physical motion classification.
         * A parked vehicle remains Parked; we simply report more
         * frequently while an ACTIVE calibration session exists.
         */
        if calibration_mode {
            self.calibration_interval_ms
        } else {
            self.interval_for(state)
        }
    }

    pub fn next_interval_from_speed(&self, speed_knots: f64) -> u32 {
        let speed_kmh = knots_to_kmh(speed_knots);
        let state = self.classify_speed_knots(speed_knots);
        let interval_ms = self.interval_for(state);

        println!("========================");
        println!("ORBI REPORTING SCHEDULER");
        println!("========================");
        println!("GNSS speed: {} knots", speed_knots);
        println!("Converted speed: {} km/h", speed_kmh);
        println!("Motion state: {:?}", state);
        println!("Next reporting interval: {} seconds", interval_ms / 1000);

        interval_ms
    }
}

pub fn knots_to_kmh(speed_knots: f64) -> f64 {
    speed_knots * 1.852
}
