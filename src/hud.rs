use crate::renderer::Vertex;

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
    fn color(&self) -> [f32; 4] {
        match self {
            ThreatLevel::Hostile => [1.0, 0.2, 0.15, 0.9],
            ThreatLevel::Friendly => [0.0, 1.0, 0.45, 0.85],
            ThreatLevel::Neutral => [0.0, 0.7, 1.0, 0.7],
            ThreatLevel::Unknown => [0.5, 0.5, 0.5, 0.5],
        }
    }
}

pub struct TacticalHud {
    pub contacts: Vec<Contact>,
    pub sweep_angle: f32,
    pub time: f32,
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
            sweep_angle: 0.0,
            time: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.time += 1.0 / 60.0;
        self.sweep_angle = (self.sweep_angle + 0.02) % (2.0 * std::f32::consts::PI);
        for (i, c) in self.contacts.iter_mut().enumerate() {
            let phase = i as f32 * 1.3;
            c.x += (self.time * 0.3 + phase).sin() * 0.001;
            c.y += (self.time * 0.25 + phase).cos() * 0.001;
        }
    }

    pub fn build_vertices(&self, w: f32, h: f32, time: f32) -> Vec<Vertex> {
        let mut verts = Vec::with_capacity(4096);
        let cx = w / 2.0;
        let cy = h / 2.0;
        let radar_r = h.min(w) * 0.35;

        self.build_radar_rings(&mut verts, cx, cy, radar_r, w, h);
        self.build_sweep(&mut verts, cx, cy, radar_r, w, h, time);
        self.build_contacts(&mut verts, cx, cy, radar_r, w, h, time);
        self.build_crosshair(&mut verts, cx, cy, radar_r, w, h);
        self.build_scanlines(&mut verts, w, h, time);

        verts
    }

    fn build_radar_rings(&self, verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32) {
        let green = [0.0, 0.7, 0.4, 0.25];
        for ring in 1..=4 {
            let rr = r * ring as f32 / 4.0;
            let segments = 64;
            for i in 0..segments {
                let a0 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                let a1 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
                let thickness = 1.0;
                let x0 = cx + a0.cos() * rr;
                let y0 = cy + a0.sin() * rr;
                let x1 = cx + a1.cos() * rr;
                let y1 = cy + a1.sin() * rr;
                let x0o = cx + a0.cos() * (rr + thickness);
                let y0o = cy + a0.sin() * (rr + thickness);
                let x1o = cx + a1.cos() * (rr + thickness);
                let y1o = cy + a1.sin() * (rr + thickness);
                push_quad_ndc(verts, x0, y0, x1, y1, x1o, y1o, x0o, y0o, green, w, h);
            }
        }
    }

    fn build_sweep(&self, verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
        let angle = time * 0.8;
        let sweep_len = 0.3;
        let steps = 12;
        for i in 0..steps {
            let a = angle - sweep_len * i as f32 / steps as f32;
            let a_next = angle - sweep_len * (i + 1) as f32 / steps as f32;
            let alpha = 0.15 * (1.0 - i as f32 / steps as f32);
            let color = [0.0, 1.0, 0.5, alpha];
            let x0 = cx;
            let y0 = cy;
            let x1 = cx + a.cos() * r;
            let y1 = cy + a.sin() * r;
            let x2 = cx + a_next.cos() * r;
            let y2 = cy + a_next.sin() * r;
            push_tri_ndc(verts, x0, y0, x1, y1, x2, y2, color, w, h);
        }
    }

    fn build_contacts(&self, verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
        for (i, contact) in self.contacts.iter().enumerate() {
            let px = cx + contact.x * r;
            let py = cy + contact.y * r;
            let pulse = 1.0 + (time * 3.0 + i as f32 * 1.5).sin() * 0.3;
            let size = 5.0 * pulse;
            let mut color = contact.threat.color();
            color[3] *= pulse.min(1.0);

            match contact.threat {
                ThreatLevel::Hostile => {
                    push_diamond_ndc(verts, px, py, size * 1.3, color, w, h);
                }
                ThreatLevel::Friendly => {
                    push_tri_marker_ndc(verts, px, py, size, color, w, h);
                }
                _ => {
                    push_rect_ndc(verts, px - size, py - size, size * 2.0, size * 2.0, color, w, h);
                }
            }
        }
    }

    fn build_crosshair(&self, verts: &mut Vec<Vertex>, cx: f32, cy: f32, _r: f32, w: f32, h: f32) {
        let color = [0.0, 0.8, 0.4, 0.5];
        let len = 12.0;
        let thick = 1.0;
        push_rect_ndc(verts, cx - len, cy - thick / 2.0, len * 2.0, thick, color, w, h);
        push_rect_ndc(verts, cx - thick / 2.0, cy - len, thick, len * 2.0, color, w, h);
    }

    fn build_scanlines(&self, verts: &mut Vec<Vertex>, w: f32, h: f32, time: f32) {
        let scroll = (time * 40.0) % 4.0;
        let mut y = scroll;
        while y < h {
            let alpha = 0.03;
            push_rect_ndc(verts, 0.0, y, w, 1.0, [0.0, 0.0, 0.0, alpha], w, h);
            y += 4.0;
        }
    }
}

fn to_ndc(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [x / w * 2.0 - 1.0, 1.0 - y / h * 2.0]
}

fn push_tri_ndc(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], w: f32, h: f32) {
    verts.push(Vertex { position: to_ndc(x0, y0, w, h), color });
    verts.push(Vertex { position: to_ndc(x1, y1, w, h), color });
    verts.push(Vertex { position: to_ndc(x2, y2, w, h), color });
}

fn push_quad_ndc(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri_ndc(verts, x0, y0, x1, y1, x2, y2, color, w, h);
    push_tri_ndc(verts, x0, y0, x2, y2, x3, y3, color, w, h);
}

fn push_rect_ndc(verts: &mut Vec<Vertex>, x: f32, y: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    push_quad_ndc(verts, x, y, x + rw, y, x + rw, y + rh, x, y + rh, color, w, h);
}

fn push_diamond_ndc(verts: &mut Vec<Vertex>, cx: f32, cy: f32, size: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri_ndc(verts, cx, cy - size, cx + size, cy, cx, cy + size, color, w, h);
    push_tri_ndc(verts, cx, cy - size, cx - size, cy, cx, cy + size, color, w, h);
}

fn push_tri_marker_ndc(verts: &mut Vec<Vertex>, cx: f32, cy: f32, size: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri_ndc(verts, cx, cy - size, cx + size * 0.866, cy + size * 0.5, cx - size * 0.866, cy + size * 0.5, color, w, h);
}
