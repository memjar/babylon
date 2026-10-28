use crate::renderer::Vertex;

pub struct SonarState {
    pub bearings: Vec<SonarContact>,
    pub own_heading: f32,
    pub depth: f32,
    pub waterfall_offset: f32,
}

pub struct SonarContact {
    pub bearing: f32,
    pub frequency: f32,
    pub intensity: f32,
    pub classification: SonarClass,
}

#[derive(Clone, Copy)]
pub enum SonarClass {
    Submarine,
    Surface,
    Biologic,
    Unknown,
}

impl SonarState {
    pub fn demo() -> Self {
        Self {
            bearings: vec![
                SonarContact { bearing: 45.0, frequency: 120.0, intensity: 0.8, classification: SonarClass::Submarine },
                SonarContact { bearing: 180.0, frequency: 220.0, intensity: 0.5, classification: SonarClass::Surface },
                SonarContact { bearing: 270.0, frequency: 60.0, intensity: 0.3, classification: SonarClass::Biologic },
                SonarContact { bearing: 320.0, frequency: 340.0, intensity: 0.6, classification: SonarClass::Unknown },
            ],
            own_heading: 0.0,
            depth: 250.0,
            waterfall_offset: 0.0,
        }
    }

    pub fn tick(&mut self, time: f32) {
        self.own_heading = (time * 0.1).sin() * 10.0;
        self.depth = 250.0 + (time * 0.05).sin() * 20.0;
        self.waterfall_offset = time * 20.0;
        for (i, c) in self.bearings.iter_mut().enumerate() {
            c.bearing += (time * 0.2 + i as f32).sin() * 0.1;
            c.intensity = (0.3 + (time * 0.5 + i as f32 * 2.0).sin() * 0.3).clamp(0.1, 1.0);
        }
    }
}

pub fn build_sonar(verts: &mut Vec<Vertex>, state: &SonarState, w: f32, h: f32, time: f32) {
    let ppi_cx = w * 0.3;
    let ppi_cy = h * 0.5;
    let ppi_r = h.min(w * 0.5) * 0.38;
    build_ppi_display(verts, state, ppi_cx, ppi_cy, ppi_r, w, h, time);

    let wf_x = w * 0.55;
    let wf_w = w * 0.4;
    let wf_y = 40.0;
    let wf_h = h - 80.0;
    build_waterfall(verts, state, wf_x, wf_y, wf_w, wf_h, w, h, time);

    build_depth_gauge(verts, state.depth, w, h);
}

fn build_ppi_display(verts: &mut Vec<Vertex>, state: &SonarState, cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
    let green = [0.0, 0.8, 0.2, 0.15];
    let bright = [0.0, 1.0, 0.3, 0.6];
    let dim = [0.0, 0.5, 0.15, 0.25];

    circle_outline(verts, cx, cy, r, 1.5, bright, 48, w, h);
    for ring in 1..=3 {
        circle_outline(verts, cx, cy, r * ring as f32 / 4.0, 1.0, dim, 32, w, h);
    }

    for i in 0..12 {
        let a = i as f32 * std::f32::consts::PI / 6.0;
        let x0 = cx + a.cos() * (r - 6.0);
        let y0 = cy + a.sin() * (r - 6.0);
        let x1 = cx + a.cos() * r;
        let y1 = cy + a.sin() * r;
        line_seg(verts, x0, y0, x1, y1, 1.0, dim, w, h);
    }

    let sweep_a = time * 0.6;
    let steps = 20;
    let arc = 0.5;
    for i in 0..steps {
        let a0 = sweep_a - arc * i as f32 / steps as f32;
        let a1 = sweep_a - arc * (i + 1) as f32 / steps as f32;
        let alpha = 0.1 * (1.0 - i as f32 / steps as f32);
        let c = [0.0, 1.0, 0.3, alpha];
        push_tri(verts, [cx, cy], [cx + a0.cos() * r, cy + a0.sin() * r], [cx + a1.cos() * r, cy + a1.sin() * r], c, w, h);
    }

    for contact in &state.bearings {
        let a = contact.bearing.to_radians();
        let dist = 0.4 + contact.intensity * 0.4;
        let px = cx + a.cos() * r * dist;
        let py = cy + a.sin() * r * dist;
        let size = 3.0 + contact.intensity * 4.0;
        let pulse = 0.5 + (time * 2.0 + contact.bearing).sin() * 0.3;
        let c = match contact.classification {
            SonarClass::Submarine => [1.0, 0.2, 0.2, pulse],
            SonarClass::Surface => [0.2, 0.8, 1.0, pulse],
            SonarClass::Biologic => [0.2, 1.0, 0.4, pulse * 0.6],
            SonarClass::Unknown => [1.0, 1.0, 0.2, pulse],
        };
        diamond(verts, px, py, size, c, w, h);
    }
}

fn build_waterfall(verts: &mut Vec<Vertex>, state: &SonarState, x: f32, y: f32, wf_w: f32, wf_h: f32, w: f32, h: f32, time: f32) {
    let bg = [0.0, 0.03, 0.0, 0.8];
    rect_filled(verts, x, y, wf_w, wf_h, bg, w, h);
    let border = [0.0, 0.6, 0.2, 0.4];
    line_seg(verts, x, y, x + wf_w, y, 1.0, border, w, h);
    line_seg(verts, x, y + wf_h, x + wf_w, y + wf_h, 1.0, border, w, h);
    line_seg(verts, x, y, x, y + wf_h, 1.0, border, w, h);
    line_seg(verts, x + wf_w, y, x + wf_w, y + wf_h, 1.0, border, w, h);

    let rows = 60;
    let row_h = wf_h / rows as f32;
    for row in 0..rows {
        let ry = y + row as f32 * row_h;
        let age = row as f32 / rows as f32;

        for contact in &state.bearings {
            let bx = x + (contact.bearing / 360.0) * wf_w;
            let spread = 4.0 + age * 8.0;
            let intensity = contact.intensity * (1.0 - age * 0.7);
            let noise = ((time * 3.0 + row as f32 * 0.7 + contact.bearing).sin() * 0.15).abs();
            let alpha = (intensity + noise) * 0.6;
            let c = match contact.classification {
                SonarClass::Submarine => [0.0, intensity, 0.0, alpha],
                SonarClass::Surface => [0.0, intensity * 0.5, intensity, alpha],
                SonarClass::Biologic => [0.0, intensity * 0.8, intensity * 0.3, alpha * 0.5],
                SonarClass::Unknown => [intensity * 0.5, intensity * 0.5, 0.0, alpha],
            };
            rect_filled(verts, bx - spread, ry, spread * 2.0, row_h, c, w, h);
        }
    }

    let cursor_y = y + (time * 20.0 % wf_h);
    line_seg(verts, x, cursor_y, x + wf_w, cursor_y, 1.0, [0.0, 1.0, 0.3, 0.5], w, h);
}

fn build_depth_gauge(verts: &mut Vec<Vertex>, depth: f32, w: f32, h: f32) {
    let gx = 20.0;
    let gy = h * 0.2;
    let gh = h * 0.6;
    let gw = 12.0;
    let green = [0.0, 0.8, 0.2, 0.3];
    let bright = [0.0, 1.0, 0.3, 0.7];
    rect_filled(verts, gx, gy, gw, gh, [0.0, 0.05, 0.0, 0.4], w, h);
    let fill = (depth / 500.0).clamp(0.0, 1.0);
    let fh = gh * fill;
    rect_filled(verts, gx, gy + gh - fh, gw, fh, green, w, h);
    let mark_y = gy + gh - fh;
    line_seg(verts, gx - 3.0, mark_y, gx + gw + 3.0, mark_y, 2.0, bright, w, h);
    for d in (0..=500).step_by(100) {
        let y = gy + gh - gh * d as f32 / 500.0;
        line_seg(verts, gx + gw, y, gx + gw + 4.0, y, 1.0, [0.0, 0.6, 0.2, 0.3], w, h);
    }
}

fn circle_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, thick: f32, color: [f32; 4], segs: usize, w: f32, h: f32) {
    let tau = 2.0 * std::f32::consts::PI;
    for i in 0..segs {
        let a0 = tau * i as f32 / segs as f32;
        let a1 = tau * (i + 1) as f32 / segs as f32;
        let inner = r - thick * 0.5;
        let outer = r + thick * 0.5;
        push_tri(verts, [cx + a0.cos() * inner, cy + a0.sin() * inner], [cx + a1.cos() * inner, cy + a1.sin() * inner], [cx + a1.cos() * outer, cy + a1.sin() * outer], color, w, h);
        push_tri(verts, [cx + a0.cos() * inner, cy + a0.sin() * inner], [cx + a1.cos() * outer, cy + a1.sin() * outer], [cx + a0.cos() * outer, cy + a0.sin() * outer], color, w, h);
    }
}

fn line_seg(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, thick: f32, color: [f32; 4], w: f32, h: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 { return; }
    let nx = -dy / len * thick * 0.5;
    let ny = dx / len * thick * 0.5;
    push_tri(verts, [x0 + nx, y0 + ny], [x0 - nx, y0 - ny], [x1 - nx, y1 - ny], color, w, h);
    push_tri(verts, [x0 + nx, y0 + ny], [x1 - nx, y1 - ny], [x1 + nx, y1 + ny], color, w, h);
}

fn diamond(verts: &mut Vec<Vertex>, cx: f32, cy: f32, s: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri(verts, [cx, cy - s], [cx + s, cy], [cx, cy + s], color, w, h);
    push_tri(verts, [cx, cy - s], [cx - s, cy], [cx, cy + s], color, w, h);
}

fn rect_filled(verts: &mut Vec<Vertex>, x: f32, y: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri(verts, [x, y], [x + rw, y], [x + rw, y + rh], color, w, h);
    push_tri(verts, [x, y], [x + rw, y + rh], [x, y + rh], color, w, h);
}

fn to_ndc(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [x / w * 2.0 - 1.0, 1.0 - y / h * 2.0]
}

fn push_tri(verts: &mut Vec<Vertex>, p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], color: [f32; 4], w: f32, h: f32) {
    verts.push(Vertex { position: to_ndc(p0[0], p0[1], w, h), color });
    verts.push(Vertex { position: to_ndc(p1[0], p1[1], w, h), color });
    verts.push(Vertex { position: to_ndc(p2[0], p2[1], w, h), color });
}
