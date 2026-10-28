use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalContact {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_ft: f32,
    pub heading: f32,
    pub speed_kts: f32,
    pub contact_type: ContactType,
    pub threat: ThreatClass,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContactType {
    Air,
    Surface,
    Subsurface,
    Ground,
    Drone,
    Missile,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatClass {
    Hostile,
    Friendly,
    Neutral,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum C2Message {
    ContactUpdate(Vec<TacticalContact>),
    EngagementZone { center_lat: f64, center_lon: f64, radius_nm: f32, active: bool },
    DroneOrbit { id: String, center_lat: f64, center_lon: f64, radius_nm: f32, alt_ft: f32 },
    SensorFov { id: String, lat: f64, lon: f64, az: f32, el: f32, fov_deg: f32 },
    Alert { level: String, message: String },
    Heartbeat { timestamp: u64 },
}

impl ThreatClass {
    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            ThreatClass::Hostile  => [1.0, 0.2, 0.15, 0.9],
            ThreatClass::Friendly => [0.0, 1.0, 0.45, 0.85],
            ThreatClass::Neutral  => [0.0, 0.7, 1.0, 0.7],
            ThreatClass::Unknown  => [0.5, 0.5, 0.5, 0.5],
        }
    }

    pub fn hex(&self) -> &'static str {
        match self {
            ThreatClass::Hostile  => "#ff3326",
            ThreatClass::Friendly => "#00ff73",
            ThreatClass::Neutral  => "#00b3ff",
            ThreatClass::Unknown  => "#808080",
        }
    }
}
