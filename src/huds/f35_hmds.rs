use crate::renderer::Vertex;

pub struct F35HmdsState {
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub altitude: f32,
    pub airspeed: f32,
    pub aoa: f32,
    pub g_load: f32,
    pub mach: f32,
    pub targets: Vec<HmdsTarget>,
}

pub struct HmdsTarget {
    pub bearing: f32,
    pub elevation: f32,
    pub range_nm: f32,
    pub designation: String,
    pub locked: bool,
}

impl F35HmdsState {
    pub fn demo() -> Self {
        Self {
            heading: 270.0,
            pitch: 2.5,
            roll: 0.0,
            altitude: 22500.0,
            airspeed: 420.0,
            aoa: 3.2,
            g_load: 1.0,
            mach: 0.82,
            targets: vec![
                HmdsTarget { bearing: 285.0, elevation: 1.0, range_nm: 12.4, designation: "TGT-1".into(), locked: true },
                HmdsTarget { bearing: 260.0, elevation: -0.5, range_nm: 24.8, designation: "TGT-2".into(), locked: false },
                HmdsTarget { bearing: 290.0, elevation: 3.0, range_nm: 8.1, designation: "FRIEND".into(), locked: false },
            ],
        }
    }

    pub fn tick(&mut self, time: f32) {
        self.heading = 270.0 + (time * 0.3).sin() * 5.0;
        self.pitch = 2.5 + (time * 0.2).sin() * 1.5;
        self.roll = (time * 0.15).sin() * 3.0;
        self.altitude = 22500.0 + (time * 0.1).sin() * 200.0;
        self.airspeed = 420.0 + (time * 0.25).sin() * 15.0;
        self.g_load = 1.0 + (time * 0.5).sin() * 0.3;
    }
}

pub fn build_f35_hmds(verts: &mut Vec<Vertex>, state: &F35HmdsState, w: f32, h: f32, time: f32) {
    let cx = w * 0.5;
    let cy = h * 0.5;
    let green = [0.0, 1.0, 0.35, 0.85];
    let dim = [0.0, 0.8, 0.3, 0.4];

    build_velocity_vector(verts, cx, cy, w, h, green);
    build_pitch_ladder(verts, cx, cy, state.pitch, w, h, dim);
    build_heading_tape(verts, cx, state.heading, w, h, green, dim);
    build_altitude_tape(verts, state.altitude, w, h, green, dim);
    build_airspeed_tape(verts, state.airspeed, w, h, green, dim);
    build_target_designators(verts, &state.targets, cx, cy, w, h, time, green);
    build_boresight(verts, cx, cy, w, h, dim);
    build_g_meter(verts, state.g_load, w, h, green);
}

fn build_velocity_vector(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let r = 12.0;
    circle_outline(verts, cx, cy, r, 1.5, color, 24, w, h);
    line_seg(verts, cx - r - 16.0, cy, cx - r, cy, 1.5, color, w, h);
    line_seg(verts, cx + r, cy, cx + r + 16.0, cy, 1.5, color, w, h);
    line_seg(verts, cx, cy - r, cx, cy - r - 10.0, 1.5, color, w, h);
}

fn build_pitch_ladder(verts: &mut Vec<Vertex>, cx: f32, cy: f32, pitch: f32, w: f32, h: f32, color: [f32; 4]) {
    let pix_per_deg = 8.0;
    let half_w = 80.0;
    let gap = 20.0;
    for deg in (-20..=20).step_by(5) {
        if deg == 0 { continue; }
        let y = cy - (deg as f32 - pitch) * pix_per_deg;
        if y < 50.0 || y > h - 50.0 { continue; }
        if deg > 0 {
            line_seg(verts, cx - half_w, y, cx - gap, y, 1.0, color, w, h);
            line_seg(verts, cx + gap, y, cx + half_w, y, 1.0, color, w, h);
        } else {
            let dash = 6.0;
            let mut x = cx - half_w;
            while x < cx - gap {
                line_seg(verts, x, y, (x + dash).min(cx - gap), y, 1.0, color, w, h);
                x += dash * 2.0;
            }
            x = cx + gap;
            while x < cx + half_w {
                line_seg(verts, x, y, (x + dash).min(cx + half_w), y, 1.0, color, w, h);
                x += dash * 2.0;
            }
        }
    }
}

fn build_heading_tape(verts: &mut Vec<Vertex>, cx: f32, hdg: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let tape_y = 60.0;
    let tape_w = 300.0;
    let pix_per_deg = tape_w / 40.0;
    line_seg(verts, cx - tape_w * 0.5, tape_y, cx + tape_w * 0.5, tape_y, 1.0, dim, w, h);
    for deg_off in -25..=25 {
        let deg = ((hdg as i32 + deg_off + 360) % 360) as f32;
        let x = cx + deg_off as f32 * pix_per_deg;
        if x < cx - tape_w * 0.5 || x > cx + tape_w * 0.5 { continue; }
        if deg as i32 % 10 == 0 {
            line_seg(verts, x, tape_y, x, tape_y + 8.0, 1.0, color, w, h);
        } else if deg as i32 % 5 == 0 {
            line_seg(verts, x, tape_y, x, tape_y + 5.0, 1.0, dim, w, h);
        }
    }
    line_seg(verts, cx, tape_y - 6.0, cx + 4.0, tape_y, 1.5, color, w, h);
    line_seg(verts, cx, tape_y - 6.0, cx - 4.0, tape_y, 1.5, color, w, h);
}

fn build_altitude_tape(verts: &mut Vec<Vertex>, alt: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let tape_x = w - 100.0;
    let tape_h = 200.0;
    let cy = h * 0.5;
    line_seg(verts, tape_x, cy - tape_h * 0.5, tape_x, cy + tape_h * 0.5, 1.0, dim, w, h);
    let pix_per_100 = tape_h / 40.0;
    let base = (alt / 100.0) as i32;
    for i in -20..=20 {
        let tick_alt = (base + i) * 100;
        let y = cy - (tick_alt as f32 - alt) / 100.0 * pix_per_100;
        if y < cy - tape_h * 0.5 || y > cy + tape_h * 0.5 { continue; }
        if tick_alt % 1000 == 0 {
            line_seg(verts, tape_x, y, tape_x + 8.0, y, 1.0, color, w, h);
        } else if tick_alt % 500 == 0 {
            line_seg(verts, tape_x, y, tape_x + 5.0, y, 1.0, dim, w, h);
        }
    }
    rect_filled(verts, tape_x + 12.0, cy - 10.0, 70.0, 20.0, [0.0, 0.1, 0.0, 0.6], w, h);
    line_seg(verts, tape_x, cy, tape_x + 12.0, cy, 1.5, color, w, h);
}

fn build_airspeed_tape(verts: &mut Vec<Vertex>, spd: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let tape_x = 100.0;
    let tape_h = 200.0;
    let cy = h * 0.5;
    line_seg(verts, tape_x, cy - tape_h * 0.5, tape_x, cy + tape_h * 0.5, 1.0, dim, w, h);
    let pix_per_10 = tape_h / 40.0;
    let base = (spd / 10.0) as i32;
    for i in -20..=20 {
        let tick_spd = (base + i) * 10;
        let y = cy - (tick_spd as f32 - spd) / 10.0 * pix_per_10;
        if y < cy - tape_h * 0.5 || y > cy + tape_h * 0.5 { continue; }
        if tick_spd % 50 == 0 {
            line_seg(verts, tape_x - 8.0, y, tape_x, y, 1.0, color, w, h);
        } else if tick_spd % 10 == 0 {
            line_seg(verts, tape_x - 5.0, y, tape_x, y, 1.0, dim, w, h);
        }
    }
    rect_filled(verts, tape_x - 70.0, cy - 10.0, 58.0, 20.0, [0.0, 0.1, 0.0, 0.6], w, h);
    line_seg(verts, tape_x - 12.0, cy, tape_x, cy, 1.5, color, w, h);
}

fn build_target_designators(verts: &mut Vec<Vertex>, targets: &[HmdsTarget], cx: f32, cy: f32, w: f32, h: f32, time: f32, color: [f32; 4]) {
    for tgt in targets {
        let offset_x = (tgt.bearing - 270.0) * 8.0;
        let offset_y = -tgt.elevation * 8.0;
        let tx = cx + offset_x;
        let ty = cy + offset_y;
        if tx < 0.0 || tx > w || ty < 0.0 || ty > h { continue; }

        let size = 14.0;
        let c = if tgt.locked {
            let pulse = 0.7 + (time * 4.0).sin() * 0.3;
            [0.0, 1.0, 0.35, pulse]
        } else {
            [0.0, 0.7, 0.3, 0.5]
        };

        line_seg(verts, tx - size, ty - size, tx - size + 6.0, ty - size, 1.5, c, w, h);
        line_seg(verts, tx - size, ty - size, tx - size, ty - size + 6.0, 1.5, c, w, h);
        line_seg(verts, tx + size, ty - size, tx + size - 6.0, ty - size, 1.5, c, w, h);
        line_seg(verts, tx + size, ty - size, tx + size, ty - size + 6.0, 1.5, c, w, h);
        line_seg(verts, tx - size, ty + size, tx - size + 6.0, ty + size, 1.5, c, w, h);
        line_seg(verts, tx - size, ty + size, tx - size, ty + size - 6.0, 1.5, c, w, h);
        line_seg(verts, tx + size, ty + size, tx + size - 6.0, ty + size, 1.5, c, w, h);
        line_seg(verts, tx + size, ty + size, tx + size, ty + size - 6.0, 1.5, c, w, h);

        if tgt.locked {
            diamond_outline(verts, tx, ty, size * 1.5, 1.5, c, w, h);
        }
    }
}

fn build_boresight(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let s = 4.0;
    line_seg(verts, cx - s, cy, cx + s, cy, 1.0, color, w, h);
    line_seg(verts, cx, cy - s, cx, cy + s, 1.0, color, w, h);
}

fn build_g_meter(verts: &mut Vec<Vertex>, g: f32, w: f32, h: f32, color: [f32; 4]) {
    let x = 40.0;
    let cy = h * 0.5;
    let bar_h = 100.0;
    let fill = ((g - 0.0) / 9.0).clamp(0.0, 1.0);
    rect_filled(verts, x, cy - bar_h * 0.5, 6.0, bar_h, [0.0, 0.2, 0.1, 0.2], w, h);
    let fh = bar_h * fill;
    rect_filled(verts, x, cy + bar_h * 0.5 - fh, 6.0, fh, color, w, h);
    line_seg(verts, x - 2.0, cy + bar_h * 0.5 - bar_h / 9.0, x + 8.0, cy + bar_h * 0.5 - bar_h / 9.0, 1.0, [1.0, 0.3, 0.0, 0.5], w, h);
}

fn diamond_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, s: f32, thick: f32, color: [f32; 4], w: f32, h: f32) {
    line_seg(verts, cx, cy - s, cx + s, cy, thick, color, w, h);
    line_seg(verts, cx + s, cy, cx, cy + s, thick, color, w, h);
    line_seg(verts, cx, cy + s, cx - s, cy, thick, color, w, h);
    line_seg(verts, cx - s, cy, cx, cy - s, thick, color, w, h);
}

fn circle_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, thick: f32, color: [f32; 4], segs: usize, w: f32, h: f32) {
    let tau = 2.0 * std::f32::consts::PI;
    for i in 0..segs {
        let a0 = tau * i as f32 / segs as f32;
        let a1 = tau * (i + 1) as f32 / segs as f32;
        let inner = r - thick * 0.5;
        let outer = r + thick * 0.5;
        let p0 = [cx + a0.cos() * inner, cy + a0.sin() * inner];
        let p1 = [cx + a1.cos() * inner, cy + a1.sin() * inner];
        let p2 = [cx + a1.cos() * outer, cy + a1.sin() * outer];
        let p3 = [cx + a0.cos() * outer, cy + a0.sin() * outer];
        push_tri(verts, p0, p1, p2, color, w, h);
        push_tri(verts, p0, p2, p3, color, w, h);
    }
}

fn line_seg(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, thick: f32, color: [f32; 4], w: f32, h: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 { return; }
    let nx = -dy / len * thick * 0.5;
    let ny = dx / len * thick * 0.5;
    let p0 = [x0 + nx, y0 + ny];
    let p1 = [x0 - nx, y0 - ny];
    let p2 = [x1 - nx, y1 - ny];
    let p3 = [x1 + nx, y1 + ny];
    push_tri(verts, p0, p1, p2, color, w, h);
    push_tri(verts, p0, p2, p3, color, w, h);
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
