use crate::renderer::Vertex;
use crate::huds::f35_hmds::{F35HmdsState, build_f35_hmds};
use crate::huds::sonar_waterfall::{SonarState, build_sonar};

pub struct Contact {
    pub x: f32,
    pub y: f32,
    pub threat: ThreatLevel,
    pub label: String,
    pub signal: f32,
}

#[derive(Clone, Copy)]
pub enum ThreatLevel {
    Hostile,
    Friendly,
    Neutral,
    Unknown,
}

impl ThreatLevel {
    pub fn color(&self) -> [f32; 4] {
        match self {
            ThreatLevel::Hostile => [1.0, 0.2, 0.15, 0.9],
            ThreatLevel::Friendly => [0.0, 1.0, 0.45, 0.85],
            ThreatLevel::Neutral => [0.0, 0.7, 1.0, 0.7],
            ThreatLevel::Unknown => [0.5, 0.5, 0.5, 0.5],
        }
    }
}

pub enum HudMode {
    Radar,
    F35Hmds,
    Sonar,
}

pub struct TacticalHud {
    pub contacts: Vec<Contact>,
    pub mode: HudMode,
    pub time: f32,
    pub f35: F35HmdsState,
    pub sonar: SonarState,
}

impl TacticalHud {
    pub fn new() -> Self {
        let contacts = vec![
            Contact { x: 0.3, y: -0.2, threat: ThreatLevel::Hostile, label: "TANGO-1".into(), signal: -45.0 },
            Contact { x: -0.4, y: 0.1, threat: ThreatLevel::Friendly, label: "ALPHA-2".into(), signal: -30.0 },
            Contact { x: 0.1, y: 0.4, threat: ThreatLevel::Neutral, label: "CIV-7".into(), signal: -65.0 },
            Contact { x: -0.2, y: -0.3, threat: ThreatLevel::Hostile, label: "TANGO-3".into(), signal: -52.0 },
            Contact { x: 0.5, y: 0.2, threat: ThreatLevel::Unknown, label: "UNK-9".into(), signal: -78.0 },
            Contact { x: -0.1, y: 0.5, threat: ThreatLevel::Friendly, label: "BRAVO-1".into(), signal: -38.0 },
        ];
        Self {
            contacts,
            mode: HudMode::F35Hmds,
            time: 0.0,
            f35: F35HmdsState::demo(),
            sonar: SonarState::demo(),
        }
    }

    pub fn tick(&mut self) {
        self.time += 1.0 / 60.0;
        for (i, c) in self.contacts.iter_mut().enumerate() {
            let phase = i as f32 * 1.3;
            c.x += (self.time * 0.3 + phase).sin() * 0.001;
            c.y += (self.time * 0.25 + phase).cos() * 0.001;
        }
        self.f35.tick(self.time);
        self.sonar.tick(self.time);
    }

    pub fn build_vertices(&self, w: f32, h: f32, time: f32) -> Vec<Vertex> {
        let mut verts = Vec::with_capacity(8192);
        match self.mode {
            HudMode::Radar => {
                crate::huds::radar::build_radar(&mut verts, &self.contacts, w * 0.5, h * 0.5, h.min(w) * 0.35, w, h, time);
            }
            HudMode::F35Hmds => {
                build_f35_hmds(&mut verts, &self.f35, w, h, time);
            }
            HudMode::Sonar => {
                build_sonar(&mut verts, &self.sonar, w, h, time);
            }
        }
        build_scanlines(&mut verts, w, h, time);
        verts
    }
}

fn build_scanlines(verts: &mut Vec<Vertex>, w: f32, h: f32, time: f32) {
    let scroll = (time * 40.0) % 4.0;
    let mut y = scroll;
    while y < h {
        push_rect_ndc(verts, 0.0, y, w, 1.0, [0.0, 0.0, 0.0, 0.03], w, h);
        y += 4.0;
    }
}

fn to_ndc(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [x / w * 2.0 - 1.0, 1.0 - y / h * 2.0]
}

fn push_rect_ndc(verts: &mut Vec<Vertex>, x: f32, y: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    verts.push(Vertex { position: to_ndc(x, y, w, h), color });
    verts.push(Vertex { position: to_ndc(x + rw, y, w, h), color });
    verts.push(Vertex { position: to_ndc(x + rw, y + rh, w, h), color });
    verts.push(Vertex { position: to_ndc(x, y, w, h), color });
    verts.push(Vertex { position: to_ndc(x + rw, y + rh, w, h), color });
    verts.push(Vertex { position: to_ndc(x, y + rh, w, h), color });
}
