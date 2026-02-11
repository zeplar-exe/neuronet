
#[derive(Clone)]
pub struct AppSettings {
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self { 
        Self { 
            theme: "light".to_string() 
        }
    }
}

#[derive(Clone)]
enum SnapshotFormat {
    JSON,
    Binary
}

#[derive(Clone)]
pub struct SimulationSettings {
    pub distance_per_tick: f64,
    snapshot_format: SnapshotFormat
}

impl Default for SimulationSettings {
    fn default() -> Self { 
        Self { 
            distance_per_tick: 1.0,
            snapshot_format: SnapshotFormat::JSON,
        }
    }
}