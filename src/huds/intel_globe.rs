use crate::renderer::Vertex;
use crate::hud::{Contact, ThreatLevel};

fn source_color(label: &str, threat: &ThreatLevel) -> [f32; 4] {
    if label.starts_with("FL") || label.starts_with("ADSB") {
        [0.31, 0.76, 0.97, 0.7] // flights — light blue #4fc3f7
    } else if label.starts_with("M") && label.len() < 30 && label.contains('.') {
        [0.94, 0.33, 0.31, 0.9] // quakes — red #ef5350
    } else if label.starts_with("FIRE") {
        [1.0, 0.43, 0.0, 0.85] // fires — orange #ff6d00
    } else if label.starts_with("STORM") {
        [0.39, 0.71, 0.96, 0.75] // weather — blue #64b5f6
    } else if label.starts_with("SAT") || label.starts_with("ISS") || label.starts_with("NORAD") {
        [0.8, 0.6, 1.0, 0.8] // satellites — purple
    } else if label.starts_with("CAM") || label.starts_with("CCTV") {
        [1.0, 1.0, 0.4, 0.6] // cctv — yellow
    } else {
        threat.color()
    }
}

const LAT_COUNT: usize = 12;
const LON_COUNT: usize = 24;
const SEGMENTS: usize = 80;
const TILT: f32 = 0.4014; // 23 deg
const SPIN_RATE: f32 = 0.15;

pub struct GlobeState {
    pub rotation: f32,
    pub contacts: Vec<GlobeContact>,
}

pub struct GlobeContact {
    pub lat: f32,
    pub lon: f32,
    pub color: [f32; 4],
    pub label: String,
    pub pulse: f32,
}

impl GlobeState {
    pub fn demo() -> Self {
        Self {
            rotation: 0.0,
            contacts: vec![
                GlobeContact { lat: 33.42, lon: -112.08, color: [1.0, 0.2, 0.15, 0.9], label: "BANDIT-1".into(), pulse: 0.0 },
                GlobeContact { lat: 51.5, lon: -0.12, color: [0.0, 1.0, 0.45, 0.85], label: "LONDON-HQ".into(), pulse: 0.3 },
                GlobeContact { lat: 35.68, lon: 139.69, color: [0.0, 0.7, 1.0, 0.7], label: "TOKYO-NET".into(), pulse: 0.6 },
                GlobeContact { lat: -33.87, lon: 151.21, color: [0.5, 0.5, 0.5, 0.5], label: "SYD-NODE".into(), pulse: 0.9 },
                GlobeContact { lat: 55.75, lon: 37.62, color: [1.0, 0.2, 0.15, 0.9], label: "THREAT-RU".into(), pulse: 0.2 },
                GlobeContact { lat: 39.91, lon: 116.39, color: [1.0, 0.6, 0.0, 0.85], label: "SIGINT-BJ".into(), pulse: 0.5 },
            ],
        }
    }

    pub fn tick(&mut self, time: f32) {
        self.rotation = time * SPIN_RATE;
        for c in &mut self.contacts {
            c.pulse = (time * 2.5 + c.pulse * 6.28).sin() * 0.5 + 0.5;
        }
    }

    pub fn ingest(&mut self, contacts: &[Contact]) {
        self.contacts.clear();
        for c in contacts {
            let color = source_color(&c.label, &c.threat);
            self.contacts.push(GlobeContact {
                lat: c.y * 90.0,
                lon: c.x * 180.0,
                color,
                label: c.label.clone(),
                pulse: 0.0,
            });
        }
    }
}

fn project(lat_deg: f32, lon_deg: f32, rotation: f32, radius: f32, cx: f32, cy: f32) -> (f32, f32, f32) {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians() + rotation;

    let x0 = lat.cos() * lon.sin();
    let y0 = -lat.sin();
    let z0 = lat.cos() * lon.cos();

    let yt = y0 * TILT.cos() - z0 * TILT.sin();
    let zt = y0 * TILT.sin() + z0 * TILT.cos();

    (cx + x0 * radius, cy + yt * radius, zt)
}

fn to_ndc(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [x / w * 2.0 - 1.0, 1.0 - y / h * 2.0]
}

fn depth_alpha(z: f32, base_alpha: f32) -> f32 {
    let d = 0.15 + 0.85 * ((z + 1.0) / 2.0);
    (d * base_alpha).min(1.0)
}

fn push_line(verts: &mut Vec<Vertex>, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], thick: f32, w: f32, h: f32) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let nx = -dy / len * thick * 0.5;
    let ny = dx / len * thick * 0.5;

    let p0 = to_ndc(x1 + nx, y1 + ny, w, h);
    let p1 = to_ndc(x1 - nx, y1 - ny, w, h);
    let p2 = to_ndc(x2 - nx, y2 - ny, w, h);
    let p3 = to_ndc(x2 + nx, y2 + ny, w, h);

    verts.push(Vertex { position: p0, color });
    verts.push(Vertex { position: p1, color });
    verts.push(Vertex { position: p2, color });
    verts.push(Vertex { position: p0, color });
    verts.push(Vertex { position: p2, color });
    verts.push(Vertex { position: p3, color });
}

fn push_rect(verts: &mut Vec<Vertex>, x: f32, y: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    let p0 = to_ndc(x, y, w, h);
    let p1 = to_ndc(x + rw, y, w, h);
    let p2 = to_ndc(x + rw, y + rh, w, h);
    let p3 = to_ndc(x, y + rh, w, h);
    verts.push(Vertex { position: p0, color });
    verts.push(Vertex { position: p1, color });
    verts.push(Vertex { position: p2, color });
    verts.push(Vertex { position: p0, color });
    verts.push(Vertex { position: p2, color });
    verts.push(Vertex { position: p3, color });
}

pub fn build_intel_globe(verts: &mut Vec<Vertex>, state: &GlobeState, w: f32, h: f32, time: f32) {
    let cx = w * 0.5;
    let cy = h * 0.48;
    let radius = h.min(w) * 0.32;
    let green: [f32; 4] = [0.0, 1.0, 0.25, 1.0];
    let dim_green: [f32; 4] = [0.0, 0.6, 0.15, 0.4];

    // latitude lines
    for i in 1..=LAT_COUNT {
        let lat = -90.0 + i as f32 * (180.0 / (LAT_COUNT as f32 + 1.0));
        let step = 360.0 / SEGMENTS as f32;
        for s in 0..SEGMENTS {
            let lon1 = s as f32 * step;
            let lon2 = (s + 1) as f32 * step;
            let (x1, y1, z1) = project(lat, lon1, state.rotation, radius, cx, cy);
            let (x2, y2, z2) = project(lat, lon2, state.rotation, radius, cx, cy);
            if z1 < -0.15 && z2 < -0.15 { continue; }
            let a = depth_alpha((z1 + z2) * 0.5, 0.7);
            let c = [green[0], green[1], green[2], a];
            push_line(verts, x1, y1, x2, y2, c, 1.0, w, h);
        }
    }

    // longitude lines
    for i in 0..LON_COUNT {
        let lon = i as f32 * (360.0 / LON_COUNT as f32);
        let step = 180.0 / SEGMENTS as f32;
        for s in 0..SEGMENTS {
            let lat1 = -90.0 + s as f32 * step;
            let lat2 = -90.0 + (s + 1) as f32 * step;
            let (x1, y1, z1) = project(lat1, lon, state.rotation, radius, cx, cy);
            let (x2, y2, z2) = project(lat2, lon, state.rotation, radius, cx, cy);
            if z1 < -0.15 && z2 < -0.15 { continue; }
            let a = depth_alpha((z1 + z2) * 0.5, 0.7);
            let c = [green[0], green[1], green[2], a];
            push_line(verts, x1, y1, x2, y2, c, 1.0, w, h);
        }
    }

    // equator highlight
    {
        let step = 360.0 / SEGMENTS as f32;
        for s in 0..SEGMENTS {
            let lon1 = s as f32 * step;
            let lon2 = (s + 1) as f32 * step;
            let (x1, y1, z1) = project(0.0, lon1, state.rotation, radius, cx, cy);
            let (x2, y2, z2) = project(0.0, lon2, state.rotation, radius, cx, cy);
            if z1 < -0.15 && z2 < -0.15 { continue; }
            let a = depth_alpha((z1 + z2) * 0.5, 1.0);
            let c = [0.0, 1.0, 0.4, a];
            push_line(verts, x1, y1, x2, y2, c, 2.0, w, h);
        }
    }

    // tactical contacts
    for contact in &state.contacts {
        let (px, py, pz) = project(contact.lat, contact.lon, state.rotation, radius * 1.02, cx, cy);
        if pz < -0.1 { continue; }

        let a = depth_alpha(pz, 1.0);
        let dot_size = 4.0 + contact.pulse * 3.0;
        let c = [contact.color[0], contact.color[1], contact.color[2], contact.color[3] * a];
        push_rect(verts, px - dot_size, py - dot_size, dot_size * 2.0, dot_size * 2.0, c, w, h);

        // pulsing ring
        let ring_r = 8.0 + contact.pulse * 12.0;
        let ring_a = c[3] * (1.0 - contact.pulse) * 0.6;
        let rc = [c[0], c[1], c[2], ring_a];
        let ring_segs = 24;
        for s in 0..ring_segs {
            let a1 = s as f32 / ring_segs as f32 * std::f32::consts::TAU;
            let a2 = (s + 1) as f32 / ring_segs as f32 * std::f32::consts::TAU;
            push_line(verts, px + a1.cos() * ring_r, py + a1.sin() * ring_r,
                      px + a2.cos() * ring_r, py + a2.sin() * ring_r, rc, 1.5, w, h);
        }

        // threat arcs from center to hostile contacts
        if contact.color[0] > 0.8 && contact.color[1] < 0.4 {
            let steps = 20;
            let arc_a = a * 0.3;
            let ac = [1.0, 0.15, 0.1, arc_a];
            for s in 0..steps {
                let t1 = s as f32 / steps as f32;
                let t2 = (s + 1) as f32 / steps as f32;
                let ax1 = cx + (px - cx) * t1;
                let ay1 = cy + (py - cy) * t1;
                let ax2 = cx + (px - cx) * t2;
                let ay2 = cy + (py - cy) * t2;
                if s % 2 == 0 {
                    push_line(verts, ax1, ay1, ax2, ay2, ac, 1.0, w, h);
                }
            }
        }
    }

    // HUD frame
    let frame_c = [0.0, 1.0, 0.25, 0.3];
    let corner = 40.0;
    // top-left
    push_line(verts, 10.0, 10.0, 10.0 + corner, 10.0, frame_c, 2.0, w, h);
    push_line(verts, 10.0, 10.0, 10.0, 10.0 + corner, frame_c, 2.0, w, h);
    // top-right
    push_line(verts, w - 10.0, 10.0, w - 10.0 - corner, 10.0, frame_c, 2.0, w, h);
    push_line(verts, w - 10.0, 10.0, w - 10.0, 10.0 + corner, frame_c, 2.0, w, h);
    // bottom-left
    push_line(verts, 10.0, h - 10.0, 10.0 + corner, h - 10.0, frame_c, 2.0, w, h);
    push_line(verts, 10.0, h - 10.0, 10.0, h - 10.0 - corner, frame_c, 2.0, w, h);
    // bottom-right
    push_line(verts, w - 10.0, h - 10.0, w - 10.0 - corner, h - 10.0, frame_c, 2.0, w, h);
    push_line(verts, w - 10.0, h - 10.0, w - 10.0, h - 10.0 - corner, frame_c, 2.0, w, h);

    // title bar
    let title_c = [0.0, 1.0, 0.25, 0.8];
    push_line(verts, 20.0, 24.0, 200.0, 24.0, title_c, 1.0, w, h);
    // "INTEL" text approximation — 5 small blocks
    let tx = 22.0;
    let ty = 14.0;
    let cw = 5.0;
    let ch = 8.0;
    let gap = 8.0;
    for i in 0..5 {
        let bx = tx + i as f32 * gap;
        push_rect(verts, bx, ty, cw, ch, title_c, w, h);
    }

    // status bar bottom
    let sb_y = h - 30.0;
    push_line(verts, 20.0, sb_y, w - 20.0, sb_y, dim_green, 1.0, w, h);

    // contact count indicator
    let count = state.contacts.len();
    for i in 0..count.min(12) {
        let bx = 25.0 + i as f32 * 14.0;
        let bc = if i < count { [0.0, 1.0, 0.25, 0.6] } else { dim_green };
        push_rect(verts, bx, sb_y + 5.0, 10.0, 6.0, bc, w, h);
    }

    // time indicator — pulsing dot
    let pulse_a = 0.5 + 0.5 * (time * 3.0).sin();
    let tc = [0.0, 1.0, 0.25, pulse_a];
    push_rect(verts, w - 40.0, sb_y + 5.0, 8.0, 8.0, tc, w, h);

    // range rings at 30% and 60% radius
    for ring_pct in [0.3f32, 0.6] {
        let rr = radius * ring_pct;
        let ring_segs = 48;
        for s in 0..ring_segs {
            let a1 = s as f32 / ring_segs as f32 * std::f32::consts::TAU;
            let a2 = (s + 1) as f32 / ring_segs as f32 * std::f32::consts::TAU;
            if s % 3 == 0 {
                push_line(verts, cx + a1.cos() * rr, cy + a1.sin() * rr,
                          cx + a2.cos() * rr, cy + a2.sin() * rr, dim_green, 0.8, w, h);
            }
        }
    }
}
