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
            heading: 270.0, pitch: 2.5, roll: 0.0,
            altitude: 22500.0, airspeed: 420.0, aoa: 3.2, g_load: 1.0, mach: 0.82,
            targets: vec![
                HmdsTarget { bearing: 285.0, elevation: 1.0, range_nm: 12.4, designation: "TGT-1".into(), locked: true },
                HmdsTarget { bearing: 260.0, elevation: -0.5, range_nm: 24.8, designation: "TGT-2".into(), locked: false },
                HmdsTarget { bearing: 290.0, elevation: 3.0, range_nm: 8.1, designation: "FRIEND".into(), locked: false },
                HmdsTarget { bearing: 275.0, elevation: -2.0, range_nm: 32.0, designation: "TGT-4".into(), locked: false },
                HmdsTarget { bearing: 268.0, elevation: 0.5, range_nm: 18.0, designation: "TGT-5".into(), locked: false },
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
        self.aoa = 3.2 + (time * 0.35).sin() * 1.0;
        self.mach = 0.82 + (time * 0.12).sin() * 0.03;
    }
}

pub fn build_f35_hmds(verts: &mut Vec<Vertex>, state: &F35HmdsState, w: f32, h: f32, time: f32) {
    let cx = w * 0.5;
    let cy = h * 0.5;
    let green = [0.0, 1.0, 0.35, 0.85];
    let bright = [0.0, 1.0, 0.35, 1.0];
    let dim = [0.0, 0.8, 0.3, 0.4];
    let faint = [0.0, 0.6, 0.2, 0.2];

    build_horizon_line(verts, cx, cy, state.pitch, state.roll, w, h, dim);
    build_pitch_ladder(verts, cx, cy, state.pitch, state.roll, w, h, green, dim);
    build_bank_angle_arc(verts, cx, cy, state.roll, w, h, green, dim);
    build_velocity_vector(verts, cx, cy, w, h, bright);
    build_heading_tape(verts, cx, state.heading, w, h, green, dim, bright);
    build_altitude_tape(verts, state.altitude, w, h, green, dim, bright);
    build_airspeed_tape(verts, state.airspeed, w, h, green, dim, bright);
    build_aoa_bracket(verts, state.aoa, w, h, green);
    build_target_designators(verts, &state.targets, cx, cy, w, h, time, green, bright);
    build_waterline(verts, cx, cy, w, h, faint);
    build_boresight(verts, cx, cy, w, h, dim);
    build_g_meter(verts, state.g_load, w, h, green, dim, bright);
    build_mach_indicator(verts, state.mach, w, h, green);
    build_hud_frame(verts, w, h, faint);
}

fn build_horizon_line(verts: &mut Vec<Vertex>, cx: f32, cy: f32, pitch: f32, roll: f32, w: f32, h: f32, color: [f32; 4]) {
    let y = cy + pitch * 10.0;
    let roll_rad = roll.to_radians();
    let extent = w * 0.45;
    let gap = 40.0;
    let cos_r = roll_rad.cos();
    let sin_r = roll_rad.sin();
    let lx0 = cx - extent * cos_r;
    let ly0 = y + extent * sin_r;
    let lx1 = cx - gap * cos_r;
    let ly1 = y + gap * sin_r;
    let rx0 = cx + gap * cos_r;
    let ry0 = y - gap * sin_r;
    let rx1 = cx + extent * cos_r;
    let ry1 = y - extent * sin_r;
    line_seg(verts, lx0, ly0, lx1, ly1, 1.5, color, w, h);
    line_seg(verts, rx0, ry0, rx1, ry1, 1.5, color, w, h);
}

fn build_pitch_ladder(verts: &mut Vec<Vertex>, cx: f32, cy: f32, pitch: f32, roll: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let pix_per_deg = 10.0;
    let half_w_major = 120.0;
    let half_w_minor = 60.0;
    let gap = 30.0;
    let roll_rad = roll.to_radians();
    let cos_r = roll_rad.cos();
    let sin_r = roll_rad.sin();

    for deg_x2 in (-60..=60) {
        let deg = deg_x2 as f32 * 0.5;
        if deg.abs() < 0.1 { continue; }
        let is_major = (deg_x2 % 10) == 0;
        let is_half = (deg_x2 % 5) == 0 && !is_major;
        if !is_major && !is_half { continue; }

        let dy = -(deg - pitch) * pix_per_deg;
        let base_y = cy + dy;
        if base_y < 40.0 || base_y > h - 40.0 { continue; }

        let hw = if is_major { half_w_major } else { half_w_minor };
        let c = if is_major { color } else { dim };
        let thick = if is_major { 1.5 } else { 1.0 };

        if deg > 0.0 {
            let lx0 = cx + (-hw) * cos_r - dy * sin_r;
            let ly0 = cy + (-hw) * sin_r + dy * cos_r;
            let lx1 = cx + (-gap) * cos_r - dy * sin_r;
            let ly1 = cy + (-gap) * sin_r + dy * cos_r;
            let rx0 = cx + gap * cos_r - dy * sin_r;
            let ry0 = cy + gap * sin_r + dy * cos_r;
            let rx1 = cx + hw * cos_r - dy * sin_r;
            let ry1 = cy + hw * sin_r + dy * cos_r;
            line_seg(verts, lx0, ly0, lx1, ly1, thick, c, w, h);
            line_seg(verts, rx0, ry0, rx1, ry1, thick, c, w, h);
            if is_major {
                let tick = 8.0;
                let tx = tick * sin_r;
                let ty = tick * cos_r;
                line_seg(verts, lx0, ly0, lx0 + tx, ly0 + ty, thick, c, w, h);
                line_seg(verts, rx1, ry1, rx1 + tx, ry1 + ty, thick, c, w, h);
            }
        } else {
            let dash = 8.0;
            let dash_gap = 6.0;
            let mut pos = gap;
            while pos < hw {
                let end = (pos + dash).min(hw);
                let lx0 = cx + (-end) * cos_r - dy * sin_r;
                let ly0 = cy + (-end) * sin_r + dy * cos_r;
                let lx1 = cx + (-pos) * cos_r - dy * sin_r;
                let ly1 = cy + (-pos) * sin_r + dy * cos_r;
                let rx0 = cx + pos * cos_r - dy * sin_r;
                let ry0 = cy + pos * sin_r + dy * cos_r;
                let rx1 = cx + end * cos_r - dy * sin_r;
                let ry1 = cy + end * sin_r + dy * cos_r;
                line_seg(verts, lx0, ly0, lx1, ly1, thick, c, w, h);
                line_seg(verts, rx0, ry0, rx1, ry1, thick, c, w, h);
                pos += dash + dash_gap;
            }
            if is_major {
                let tick = 8.0;
                let edge_l_x = cx + (-hw) * cos_r - dy * sin_r;
                let edge_l_y = cy + (-hw) * sin_r + dy * cos_r;
                let edge_r_x = cx + hw * cos_r - dy * sin_r;
                let edge_r_y = cy + hw * sin_r + dy * cos_r;
                let tx = -tick * sin_r;
                let ty = -tick * cos_r;
                line_seg(verts, edge_l_x, edge_l_y, edge_l_x + tx, edge_l_y + ty, thick, c, w, h);
                line_seg(verts, edge_r_x, edge_r_y, edge_r_x + tx, edge_r_y + ty, thick, c, w, h);
            }
        }
    }
}

fn build_bank_angle_arc(verts: &mut Vec<Vertex>, cx: f32, cy: f32, roll: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let r = h.min(w) * 0.35;
    let arc_start = -std::f32::consts::PI * 0.5 - 1.1;
    let arc_end = -std::f32::consts::PI * 0.5 + 1.1;
    let segs = 48;
    for i in 0..segs {
        let a0 = arc_start + (arc_end - arc_start) * i as f32 / segs as f32;
        let a1 = arc_start + (arc_end - arc_start) * (i + 1) as f32 / segs as f32;
        let ri = r - 0.5;
        let ro = r + 0.5;
        push_tri(verts, [cx+a0.cos()*ri,cy+a0.sin()*ri], [cx+a1.cos()*ri,cy+a1.sin()*ri], [cx+a1.cos()*ro,cy+a1.sin()*ro], dim, w, h);
        push_tri(verts, [cx+a0.cos()*ri,cy+a0.sin()*ri], [cx+a1.cos()*ro,cy+a1.sin()*ro], [cx+a0.cos()*ro,cy+a0.sin()*ro], dim, w, h);
    }

    let marks = [0.0_f32, 10.0, 20.0, 30.0, 45.0, 60.0, -10.0, -20.0, -30.0, -45.0, -60.0];
    for &deg in &marks {
        let a = -std::f32::consts::FRAC_PI_2 + deg.to_radians();
        let len = if deg.abs() < 0.1 || (deg.abs() - 30.0).abs() < 0.1 || (deg.abs() - 60.0).abs() < 0.1 { 12.0 } else { 7.0 };
        let c = if deg.abs() < 0.1 { color } else { dim };
        let x0 = cx + a.cos() * r;
        let y0 = cy + a.sin() * r;
        let x1 = cx + a.cos() * (r + len);
        let y1 = cy + a.sin() * (r + len);
        line_seg(verts, x0, y0, x1, y1, 1.5, c, w, h);
    }

    let roll_a = -std::f32::consts::FRAC_PI_2 + roll.to_radians();
    let tip_r = r - 4.0;
    let base_r = r - 14.0;
    let spread = 0.05;
    let tip = [cx + roll_a.cos() * tip_r, cy + roll_a.sin() * tip_r];
    let left = [cx + (roll_a - spread).cos() * base_r, cy + (roll_a - spread).sin() * base_r];
    let right = [cx + (roll_a + spread).cos() * base_r, cy + (roll_a + spread).sin() * base_r];
    push_tri(verts, tip, left, right, color, w, h);
}

fn build_velocity_vector(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let r = 16.0;
    circle_outline(verts, cx, cy, r, 2.0, color, 32, w, h);
    line_seg(verts, cx - r - 22.0, cy, cx - r, cy, 2.0, color, w, h);
    line_seg(verts, cx + r, cy, cx + r + 22.0, cy, 2.0, color, w, h);
    line_seg(verts, cx, cy - r, cx, cy - r - 14.0, 2.0, color, w, h);
    line_seg(verts, cx - r - 22.0, cy, cx - r - 22.0, cy + 5.0, 1.5, color, w, h);
    line_seg(verts, cx + r + 22.0, cy, cx + r + 22.0, cy + 5.0, 1.5, color, w, h);
}

fn build_heading_tape(verts: &mut Vec<Vertex>, cx: f32, hdg: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4], bright: [f32; 4]) {
    let tape_y = 55.0;
    let tape_w = 500.0;
    let pix_per_deg = tape_w / 50.0;

    line_seg(verts, cx - tape_w * 0.5, tape_y, cx + tape_w * 0.5, tape_y, 1.5, dim, w, h);
    line_seg(verts, cx - tape_w * 0.5, tape_y, cx - tape_w * 0.5, tape_y - 4.0, 1.0, dim, w, h);
    line_seg(verts, cx + tape_w * 0.5, tape_y, cx + tape_w * 0.5, tape_y - 4.0, 1.0, dim, w, h);

    for deg_off in -30..=30 {
        let deg = ((hdg as i32 + deg_off + 360) % 360) as f32;
        let x = cx + deg_off as f32 * pix_per_deg;
        if x < cx - tape_w * 0.5 || x > cx + tape_w * 0.5 { continue; }

        let deg_i = deg as i32;
        if deg_i % 10 == 0 {
            line_seg(verts, x, tape_y, x, tape_y + 12.0, 1.5, color, w, h);
            let label_w = 16.0;
            let label_h = 3.0;
            rect_filled(verts, x - label_w * 0.5, tape_y + 14.0, label_w, label_h, color, w, h);
        } else if deg_i % 5 == 0 {
            line_seg(verts, x, tape_y, x, tape_y + 8.0, 1.0, dim, w, h);
        } else {
            line_seg(verts, x, tape_y, x, tape_y + 4.0, 0.8, [dim[0], dim[1], dim[2], dim[3] * 0.5], w, h);
        }
    }

    let caret_s = 6.0;
    push_tri(verts, [cx, tape_y - 2.0], [cx - caret_s, tape_y - caret_s - 2.0], [cx + caret_s, tape_y - caret_s - 2.0], bright, w, h);

    rect_filled(verts, cx - 20.0, tape_y - caret_s - 2.0 - 14.0, 40.0, 12.0, [0.0, 0.12, 0.04, 0.7], w, h);
    line_seg(verts, cx - 20.0, tape_y - caret_s - 2.0 - 14.0, cx + 20.0, tape_y - caret_s - 2.0 - 14.0, 1.0, bright, w, h);
    line_seg(verts, cx - 20.0, tape_y - caret_s - 2.0 - 2.0, cx + 20.0, tape_y - caret_s - 2.0 - 2.0, 1.0, bright, w, h);
    line_seg(verts, cx - 20.0, tape_y - caret_s - 2.0 - 14.0, cx - 20.0, tape_y - caret_s - 2.0 - 2.0, 1.0, bright, w, h);
    line_seg(verts, cx + 20.0, tape_y - caret_s - 2.0 - 14.0, cx + 20.0, tape_y - caret_s - 2.0 - 2.0, 1.0, bright, w, h);
}

fn build_altitude_tape(verts: &mut Vec<Vertex>, alt: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4], bright: [f32; 4]) {
    let tape_x = w - 130.0;
    let tape_h = 350.0;
    let cy = h * 0.5;

    line_seg(verts, tape_x, cy - tape_h * 0.5, tape_x, cy + tape_h * 0.5, 1.5, dim, w, h);
    line_seg(verts, tape_x, cy - tape_h * 0.5, tape_x + 6.0, cy - tape_h * 0.5, 1.0, dim, w, h);
    line_seg(verts, tape_x, cy + tape_h * 0.5, tape_x + 6.0, cy + tape_h * 0.5, 1.0, dim, w, h);

    let pix_per_100 = tape_h / 50.0;
    let base = (alt / 100.0) as i32;
    for i in -25..=25 {
        let tick_alt = (base + i) * 100;
        let y = cy - (tick_alt as f32 - alt) / 100.0 * pix_per_100;
        if y < cy - tape_h * 0.5 || y > cy + tape_h * 0.5 { continue; }
        if tick_alt % 1000 == 0 {
            line_seg(verts, tape_x, y, tape_x + 12.0, y, 1.5, color, w, h);
            let bar_w = 24.0;
            rect_filled(verts, tape_x + 14.0, y - 1.5, bar_w, 3.0, color, w, h);
        } else if tick_alt % 500 == 0 {
            line_seg(verts, tape_x, y, tape_x + 8.0, y, 1.0, dim, w, h);
        } else {
            line_seg(verts, tape_x, y, tape_x + 4.0, y, 0.8, [dim[0], dim[1], dim[2], dim[3] * 0.5], w, h);
        }
    }

    let box_w = 80.0;
    let box_h = 22.0;
    let box_x = tape_x + 14.0;
    rect_filled(verts, box_x, cy - box_h * 0.5, box_w, box_h, [0.0, 0.12, 0.04, 0.75], w, h);
    line_seg(verts, box_x, cy - box_h * 0.5, box_x + box_w, cy - box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x, cy + box_h * 0.5, box_x + box_w, cy + box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x, cy - box_h * 0.5, box_x, cy + box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x + box_w, cy - box_h * 0.5, box_x + box_w, cy + box_h * 0.5, 1.5, bright, w, h);

    line_seg(verts, tape_x, cy, box_x, cy, 2.0, bright, w, h);

    let digits = alt / 1000.0;
    let fill_pct = (digits / 50.0).clamp(0.0, 1.0);
    let fill_w = (box_w - 6.0) * fill_pct;
    rect_filled(verts, box_x + 3.0, cy - 4.0, fill_w, 8.0, [0.0, 0.8, 0.3, 0.5], w, h);
}

fn build_airspeed_tape(verts: &mut Vec<Vertex>, spd: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4], bright: [f32; 4]) {
    let tape_x = 130.0;
    let tape_h = 350.0;
    let cy = h * 0.5;

    line_seg(verts, tape_x, cy - tape_h * 0.5, tape_x, cy + tape_h * 0.5, 1.5, dim, w, h);
    line_seg(verts, tape_x - 6.0, cy - tape_h * 0.5, tape_x, cy - tape_h * 0.5, 1.0, dim, w, h);
    line_seg(verts, tape_x - 6.0, cy + tape_h * 0.5, tape_x, cy + tape_h * 0.5, 1.0, dim, w, h);

    let pix_per_10 = tape_h / 50.0;
    let base = (spd / 10.0) as i32;
    for i in -25..=25 {
        let tick_spd = (base + i) * 10;
        let y = cy - (tick_spd as f32 - spd) / 10.0 * pix_per_10;
        if y < cy - tape_h * 0.5 || y > cy + tape_h * 0.5 { continue; }
        if tick_spd % 50 == 0 {
            line_seg(verts, tape_x - 12.0, y, tape_x, y, 1.5, color, w, h);
            let bar_w = 24.0;
            rect_filled(verts, tape_x - 14.0 - bar_w, y - 1.5, bar_w, 3.0, color, w, h);
        } else if tick_spd % 10 == 0 {
            line_seg(verts, tape_x - 8.0, y, tape_x, y, 1.0, dim, w, h);
        }
    }

    let box_w = 80.0;
    let box_h = 22.0;
    let box_x = tape_x - 14.0 - box_w;
    rect_filled(verts, box_x, cy - box_h * 0.5, box_w, box_h, [0.0, 0.12, 0.04, 0.75], w, h);
    line_seg(verts, box_x, cy - box_h * 0.5, box_x + box_w, cy - box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x, cy + box_h * 0.5, box_x + box_w, cy + box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x, cy - box_h * 0.5, box_x, cy + box_h * 0.5, 1.5, bright, w, h);
    line_seg(verts, box_x + box_w, cy - box_h * 0.5, box_x + box_w, cy + box_h * 0.5, 1.5, bright, w, h);

    line_seg(verts, box_x + box_w, cy, tape_x, cy, 2.0, bright, w, h);

    let fill_pct = (spd / 600.0).clamp(0.0, 1.0);
    let fill_w = (box_w - 6.0) * fill_pct;
    rect_filled(verts, box_x + 3.0, cy - 4.0, fill_w, 8.0, [0.0, 0.8, 0.3, 0.5], w, h);
}

fn build_aoa_bracket(verts: &mut Vec<Vertex>, aoa: f32, w: f32, h: f32, color: [f32; 4]) {
    let x = 55.0;
    let cy = h * 0.5;

    let bracket_h = 60.0;
    let bracket_w = 8.0;
    line_seg(verts, x, cy - bracket_h, x, cy + bracket_h, 1.5, color, w, h);
    line_seg(verts, x, cy - bracket_h, x + bracket_w, cy - bracket_h, 1.0, color, w, h);
    line_seg(verts, x, cy + bracket_h, x + bracket_w, cy + bracket_h, 1.0, color, w, h);

    line_seg(verts, x + bracket_w + 2.0, cy, x + bracket_w + 10.0, cy, 2.0, color, w, h);

    let aoa_y = cy - (aoa - 3.0) * 15.0;
    let aoa_y_clamped = aoa_y.clamp(cy - bracket_h + 5.0, cy + bracket_h - 5.0);
    let sz = 5.0;
    push_tri(verts, [x + bracket_w + 14.0, aoa_y_clamped], [x + bracket_w + 14.0 + sz, aoa_y_clamped - sz], [x + bracket_w + 14.0 + sz, aoa_y_clamped + sz], color, w, h);

    if aoa > 5.0 {
        let warn = [1.0, 0.5, 0.0, 0.8];
        push_tri(verts, [x - 6.0, cy - bracket_h - 8.0], [x - 10.0, cy - bracket_h - 16.0], [x - 2.0, cy - bracket_h - 16.0], warn, w, h);
    }
}

fn build_target_designators(verts: &mut Vec<Vertex>, targets: &[HmdsTarget], cx: f32, cy: f32, w: f32, h: f32, time: f32, color: [f32; 4], bright: [f32; 4]) {
    for (idx, tgt) in targets.iter().enumerate() {
        let offset_x = (tgt.bearing - 270.0) * 10.0;
        let offset_y = -tgt.elevation * 10.0;
        let tx = cx + offset_x;
        let ty = cy + offset_y;
        if tx < 30.0 || tx > w - 30.0 || ty < 30.0 || ty > h - 30.0 { continue; }

        let size = 18.0;
        let c = if tgt.locked {
            let pulse = 0.7 + (time * 4.0).sin() * 0.3;
            [bright[0], bright[1], bright[2], pulse]
        } else {
            [color[0], color[1], color[2], 0.6]
        };

        let corner = 8.0;
        let thick = if tgt.locked { 2.0 } else { 1.5 };
        line_seg(verts, tx - size, ty - size, tx - size + corner, ty - size, thick, c, w, h);
        line_seg(verts, tx - size, ty - size, tx - size, ty - size + corner, thick, c, w, h);
        line_seg(verts, tx + size - corner, ty - size, tx + size, ty - size, thick, c, w, h);
        line_seg(verts, tx + size, ty - size, tx + size, ty - size + corner, thick, c, w, h);
        line_seg(verts, tx - size, ty + size - corner, tx - size, ty + size, thick, c, w, h);
        line_seg(verts, tx - size, ty + size, tx - size + corner, ty + size, thick, c, w, h);
        line_seg(verts, tx + size, ty + size - corner, tx + size, ty + size, thick, c, w, h);
        line_seg(verts, tx + size - corner, ty + size, tx + size, ty + size, thick, c, w, h);

        if tgt.locked {
            diamond_outline(verts, tx, ty, size * 1.6, 2.0, c, w, h);
            let flash = 0.5 + (time * 3.0 + idx as f32).sin() * 0.5;
            circle_outline(verts, tx, ty, size * 2.2, 1.0, [c[0], c[1], c[2], c[3] * flash * 0.3], 24, w, h);
        }

        let range_bar_w = (tgt.range_nm / 40.0).clamp(0.0, 1.0) * 30.0;
        rect_filled(verts, tx - 15.0, ty + size + 4.0, range_bar_w, 2.0, [c[0], c[1], c[2], c[3] * 0.5], w, h);

        let vel_len = 10.0;
        let vel_a = (tgt.bearing - 270.0).to_radians() + 0.3;
        line_seg(verts, tx, ty, tx + vel_a.cos() * vel_len, ty - vel_a.sin() * vel_len, 1.0, [c[0], c[1], c[2], c[3] * 0.4], w, h);
    }
}

fn build_waterline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let arm = 20.0;
    let dip = 6.0;
    let mid_dip = 3.0;
    line_seg(verts, cx - arm, cy, cx - arm * 0.4, cy + dip, 1.0, color, w, h);
    line_seg(verts, cx - arm * 0.4, cy + dip, cx, cy + mid_dip, 1.0, color, w, h);
    line_seg(verts, cx, cy + mid_dip, cx + arm * 0.4, cy + dip, 1.0, color, w, h);
    line_seg(verts, cx + arm * 0.4, cy + dip, cx + arm, cy, 1.0, color, w, h);
}

fn build_boresight(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let s = 6.0;
    line_seg(verts, cx - s, cy, cx + s, cy, 1.0, color, w, h);
    line_seg(verts, cx, cy - s, cx, cy + s, 1.0, color, w, h);
}

fn build_g_meter(verts: &mut Vec<Vertex>, g: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4], _bright: [f32; 4]) {
    let x = 30.0;
    let cy = h * 0.5 + 120.0;
    let bar_h = 80.0;
    let bar_w = 8.0;

    rect_filled(verts, x, cy - bar_h * 0.5, bar_w, bar_h, [0.0, 0.2, 0.1, 0.2], w, h);
    let fill = (g / 9.0).clamp(0.0, 1.0);
    let fh = bar_h * fill;
    rect_filled(verts, x, cy + bar_h * 0.5 - fh, bar_w, fh, color, w, h);

    let one_g_y = cy + bar_h * 0.5 - bar_h / 9.0;
    line_seg(verts, x - 3.0, one_g_y, x + bar_w + 3.0, one_g_y, 1.5, dim, w, h);

    for gmark in [3.0_f32, 5.0, 7.0, 9.0] {
        let my = cy + bar_h * 0.5 - bar_h * gmark / 9.0;
        line_seg(verts, x + bar_w, my, x + bar_w + 4.0, my, 1.0, dim, w, h);
    }

    let warn_y = cy + bar_h * 0.5 - bar_h * 7.0 / 9.0;
    line_seg(verts, x - 2.0, warn_y, x + bar_w + 2.0, warn_y, 1.0, [1.0, 0.3, 0.0, 0.4], w, h);
}

fn build_mach_indicator(verts: &mut Vec<Vertex>, mach: f32, w: f32, h: f32, color: [f32; 4]) {
    let x = 36.0;
    let y = h * 0.5 + 180.0;
    let bar_w = 40.0;
    let bar_h = 6.0;
    rect_filled(verts, x, y, bar_w, bar_h, [0.0, 0.15, 0.05, 0.2], w, h);
    let fill = (mach / 2.0).clamp(0.0, 1.0) * bar_w;
    rect_filled(verts, x, y, fill, bar_h, [color[0], color[1], color[2], 0.5], w, h);
    let m1_x = x + bar_w * 0.5;
    line_seg(verts, m1_x, y - 2.0, m1_x, y + bar_h + 2.0, 1.0, [1.0, 0.3, 0.0, 0.3], w, h);
}

fn build_hud_frame(verts: &mut Vec<Vertex>, w: f32, h: f32, color: [f32; 4]) {
    let m = 20.0;
    let tick = 25.0;
    line_seg(verts, m, m, m + tick, m, 1.0, color, w, h);
    line_seg(verts, m, m, m, m + tick, 1.0, color, w, h);
    line_seg(verts, w - m - tick, m, w - m, m, 1.0, color, w, h);
    line_seg(verts, w - m, m, w - m, m + tick, 1.0, color, w, h);
    line_seg(verts, m, h - m, m + tick, h - m, 1.0, color, w, h);
    line_seg(verts, m, h - m - tick, m, h - m, 1.0, color, w, h);
    line_seg(verts, w - m - tick, h - m, w - m, h - m, 1.0, color, w, h);
    line_seg(verts, w - m, h - m - tick, w - m, h - m, 1.0, color, w, h);
}

fn diamond_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, s: f32, thick: f32, color: [f32; 4], w: f32, h: f32) {
    line_seg(verts, cx, cy - s, cx + s, cy, thick, color, w, h);
    line_seg(verts, cx + s, cy, cx, cy + s, thick, color, w, h);
    line_seg(verts, cx, cy + s, cx - s, cy, thick, color, w, h);
    line_seg(verts, cx - s, cy, cx, cy - s, thick, color, w, h);
}

fn circle_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, thick: f32, color: [f32; 4], segs: usize, w: f32, h: f32) {
    let tau = std::f32::consts::TAU;
    for i in 0..segs {
        let a0 = tau * i as f32 / segs as f32;
        let a1 = tau * (i + 1) as f32 / segs as f32;
        let ri = r - thick * 0.5;
        let ro = r + thick * 0.5;
        push_tri(verts, [cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ri,cy+a1.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],color,w,h);
        push_tri(verts, [cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],[cx+a0.cos()*ro,cy+a0.sin()*ro],color,w,h);
    }
}

fn line_seg(verts: &mut Vec<Vertex>, x0:f32,y0:f32,x1:f32,y1:f32,thick:f32,color:[f32;4],w:f32,h:f32) {
    let dx=x1-x0;let dy=y1-y0;let len=(dx*dx+dy*dy).sqrt();if len<0.001{return}
    let nx=-dy/len*thick*0.5;let ny=dx/len*thick*0.5;
    push_tri(verts,[x0+nx,y0+ny],[x0-nx,y0-ny],[x1-nx,y1-ny],color,w,h);
    push_tri(verts,[x0+nx,y0+ny],[x1-nx,y1-ny],[x1+nx,y1+ny],color,w,h);
}

fn rect_filled(verts: &mut Vec<Vertex>, x:f32,y:f32,rw:f32,rh:f32,color:[f32;4],w:f32,h:f32) {
    push_tri(verts,[x,y],[x+rw,y],[x+rw,y+rh],color,w,h);
    push_tri(verts,[x,y],[x+rw,y+rh],[x,y+rh],color,w,h);
}

fn to_ndc(x:f32,y:f32,w:f32,h:f32)->[f32;2]{[x/w*2.0-1.0,1.0-y/h*2.0]}

fn push_tri(verts: &mut Vec<Vertex>, p0:[f32;2],p1:[f32;2],p2:[f32;2],color:[f32;4],w:f32,h:f32) {
    verts.push(Vertex{position:to_ndc(p0[0],p0[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p1[0],p1[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p2[0],p2[1],w,h),color});
}
