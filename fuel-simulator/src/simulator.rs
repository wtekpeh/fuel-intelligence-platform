use chrono::{DateTime, Duration, Utc};
use rand::Rng;

use crate::{
    config::{AppConfig, SimulationTimeMode},
    models::{FuelReading, SimulationMode},
};

pub struct FuelSimulator {
    device_id: String,
    tank_capacity_litres: f64,
    current_fuel_litres: f64,
    current_time: DateTime<Utc>,
    simulation_time_mode: SimulationTimeMode,
    reading_count: i32,
    latitude: f64,
    longitude: f64,

    theft_reading_number: i32,
    leak_start_reading: i32,
    leak_end_reading: i32,
    refill_reading_number: i32,
}

impl FuelSimulator {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            device_id: config.device_id.clone(),
            tank_capacity_litres: config.tank_capacity_litres,
            current_fuel_litres: config.initial_fuel_litres,
            current_time: Utc::now(),
            simulation_time_mode: config.simulation_time_mode.clone(),
            reading_count: 0,
            latitude: config.latitude,
            longitude: config.longitude,

            theft_reading_number: config.theft_reading_number,
            leak_start_reading: config.leak_start_reading,
            leak_end_reading: config.leak_end_reading,
            refill_reading_number: config.refill_reading_number,
        }
    }

    pub fn next_reading(&mut self) -> FuelReading {
        self.reading_count += 1;

        let mode = self.select_simulation_mode();
        let fuel_change = self.generate_fuel_change(mode);
        let (vibration_level, motion_detected) = self.generate_vibration(mode);
        self.update_location(mode);

        self.current_fuel_litres += fuel_change;

        if self.current_fuel_litres <= 10.0 {
            let refill_amount = generate_refill_amount();

            self.current_fuel_litres += refill_amount;

            self.current_fuel_litres = self.current_fuel_litres.min(self.tank_capacity_litres);
        }

        self.current_fuel_litres = self
            .current_fuel_litres
            .clamp(0.0, self.tank_capacity_litres);

        let reading = FuelReading {
            device_id: self.device_id.clone(),
            timestamp: match self.simulation_time_mode {
                SimulationTimeMode::Realtime => Utc::now(),
                SimulationTimeMode::Historical => self.current_time,
            },
            fuel_level_litres: round_2(self.current_fuel_litres),
            fuel_level_percentage: round_2(
                (self.current_fuel_litres / self.tank_capacity_litres) * 100.0,
            ),
            latitude: self.latitude,
            longitude: self.longitude,
            vibration_level,
            motion_detected,
            simulation_mode: format!("{:?}", mode),
        };

        self.current_time += Duration::minutes(1);

        reading
    }

    fn select_simulation_mode(&self) -> SimulationMode {
        if self.reading_count == self.theft_reading_number {
            SimulationMode::Theft
        } else if self.reading_count >= self.leak_start_reading
            && self.reading_count <= self.leak_end_reading
        {
            SimulationMode::Leak
        } else if self.reading_count == self.refill_reading_number {
            SimulationMode::Refill
        } else {
            SimulationMode::Normal
        }
    }

    fn generate_fuel_change(&self, mode: SimulationMode) -> f64 {
        match mode {
            SimulationMode::Normal => -generate_normal_consumption(),
            SimulationMode::Theft => -generate_theft_drop(),
            SimulationMode::Leak => -generate_leak_loss(),
            SimulationMode::Refill => generate_refill_amount(),
        }
    }

    fn generate_vibration(&self, mode: SimulationMode) -> (f64, bool) {
        let vibration_level = match mode {
            SimulationMode::Normal => rand::thread_rng().gen_range(4.0..12.0),
            SimulationMode::Theft => rand::thread_rng().gen_range(0.0..2.0),
            SimulationMode::Leak => rand::thread_rng().gen_range(3.0..8.0),
            SimulationMode::Refill => rand::thread_rng().gen_range(1.0..5.0),
        };

        let motion_detected = vibration_level >= 3.0;

        (round_2(vibration_level), motion_detected)
    }

    fn update_location(&mut self, mode: SimulationMode) {
        match mode {
            SimulationMode::Normal => {
                self.latitude = round_6(self.latitude + 0.00025);
                self.longitude = round_6(self.longitude + 0.00020);
            }

            SimulationMode::Theft | SimulationMode::Leak | SimulationMode::Refill => {
                // Keep location stable for now during special fuel events.
                // Later, we can make this configurable per scenario.
            }
        }
    }
}

fn generate_normal_consumption() -> f64 {
    rand::thread_rng().gen_range(0.01..0.05)
}

fn generate_theft_drop() -> f64 {
    rand::thread_rng().gen_range(25.0..50.0)
}

fn generate_leak_loss() -> f64 {
    rand::thread_rng().gen_range(2.0..4.0)
}

fn generate_refill_amount() -> f64 {
    rand::thread_rng().gen_range(40.0..80.0)
}

fn round_2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
