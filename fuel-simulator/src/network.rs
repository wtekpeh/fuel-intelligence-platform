#[derive(Debug, Clone, Copy)]
pub enum NetworkStatus {
    Online,
    Offline,
}

pub struct NetworkSimulator {
    tick_count: usize,
}

impl NetworkSimulator {
    pub fn new() -> Self {
        Self { tick_count: 0 }
    }

    pub fn current_status(&mut self) -> NetworkStatus {
        self.tick_count += 1;

        match self.tick_count {
            8..=14 => NetworkStatus::Offline,
            28..=35 => NetworkStatus::Offline,
            _ => NetworkStatus::Online,
        }
    }
}
