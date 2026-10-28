use crate::renderer::Vertex;

pub struct ReaperState {
    pub alt_agl: f32,
    pub groundspeed: f32,
    pub heading: f32,
    pub fuel_pct: f32,
    pub sensor_az: f32,
    pub sensor_el: f32,
    pub weapon_status: WeaponStatus,
    pub lat: f32,
    pub lon: f32,
    pub link_quality: f32,
}

pub enum WeaponStatus { Safe, Armed, Firing }

impl ReaperState {
    pub fn demo() -> Self {
        Self {
            alt_agl: 18500.0, groundspeed: 165.0, heading: 182.0,
            fuel_pct: 0.72, sensor_az: 5.2, sensor_el: -45.0,
            weapon_status: WeaponStatus::Armed, lat: 33.42, lon: -112.08,
            link_quality: 0.92,
        }
    }
    pub fn tick(&mut self, time: f32) {
        self.heading = 182.0 + (time * 0.05).sin() * 3.0;
        self.alt_agl = 18500.0 + (time * 0.03).sin() * 200.0;
        self.groundspeed = 165.0 + (time * 0.1).sin() * 5.0;
        self.fuel_pct = (0.72 - time * 0.001).max(0.0);
        self.sensor_az = 5.2 + (time * 0.2).sin() * 8.0;
        self.sensor_el = -45.0 + (time * 0.15).cos() * 5.0;
        self.link_quality = (0.92 + (time * 0.3).sin() * 0.05).clamp(0.0, 1.0);
    }
}

pub fn build_reaper(verts: &mut Vec<Vertex>, state: &ReaperState, w: f32, h: f32, time: f32) {
    let green = [0.0, 0.9, 0.3, 0.8];
    let dim = [0.0, 0.5, 0.2, 0.4];
    let mid_x = w * 0.5;
    let mid_y = h * 0.5;
    let divider = [0.0, 0.6, 0.25, 0.3];

    line_seg(verts, mid_x, 0.0, mid_x, h, 1.0, divider, w, h);
    line_seg(verts, 0.0, mid_y, mid_x, mid_y, 1.0, divider, w, h);

    build_pfd(verts, state, 0.0, 0.0, mid_x, mid_y, w, h, green, dim);
    build_sensor_view(verts, state, 0.0, mid_y, mid_x, h - mid_y, w, h, time);
    build_map_panel(verts, state, mid_x, 0.0, w - mid_x, mid_y, w, h, time, green, dim);
    build_weapons_panel(verts, state, mid_x, mid_y, w - mid_x, h - mid_y, w, h, time, green, dim);
}

fn build_pfd(verts: &mut Vec<Vertex>, state: &ReaperState, px: f32, py: f32, pw: f32, ph: f32, w: f32, h: f32, green: [f32; 4], dim: [f32; 4]) {
    rect_filled(verts, px, py, pw, ph, [0.02, 0.04, 0.02, 0.8], w, h);
    let cx = px + pw * 0.5;
    let cy = py + ph * 0.5;
    build_horizon(verts, cx, cy, pw * 0.35, state.heading, w, h, green, dim);
    let alt_x = px + pw * 0.8;
    let bar_h = ph * 0.5;
    rect_filled(verts, alt_x, cy - bar_h * 0.5, 6.0, bar_h, [0.0, 0.15, 0.05, 0.3], w, h);
    let fill = (state.alt_agl / 25000.0).clamp(0.0, 1.0);
    rect_filled(verts, alt_x, cy + bar_h * 0.5 - bar_h * fill, 6.0, bar_h * fill, green, w, h);
    let spd_x = px + pw * 0.12;
    rect_filled(verts, spd_x, cy - bar_h * 0.5, 6.0, bar_h, [0.0, 0.15, 0.05, 0.3], w, h);
    let sfill = (state.groundspeed / 250.0).clamp(0.0, 1.0);
    rect_filled(verts, spd_x, cy + bar_h * 0.5 - bar_h * sfill, 6.0, bar_h * sfill, green, w, h);
}

fn build_horizon(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, hdg: f32, w: f32, h: f32, green: [f32; 4], dim: [f32; 4]) {
    line_seg(verts, cx - r, cy, cx - 15.0, cy, 1.5, green, w, h);
    line_seg(verts, cx + 15.0, cy, cx + r, cy, 1.5, green, w, h);
    circle_outline_fn(verts, cx, cy, 6.0, 1.5, green, 12, w, h);
    let hdg_rad = hdg.to_radians();
    let nx = hdg_rad.sin() * 20.0;
    let ny = -hdg_rad.cos() * 20.0;
    line_seg(verts, cx, cy - 8.0, cx + nx * 0.5, cy - 8.0 + ny * 0.5, 1.0, dim, w, h);
}

fn build_sensor_view(verts: &mut Vec<Vertex>, state: &ReaperState, px: f32, py: f32, pw: f32, ph: f32, w: f32, h: f32, time: f32) {
    rect_filled(verts, px, py, pw, ph, [0.08, 0.08, 0.08, 0.9], w, h);
    let cx = px + pw * 0.5;
    let cy = py + ph * 0.5;
    let fg = [0.9, 0.9, 0.9, 0.8];
    let gap = 12.0;
    let arm = 25.0;
    line_seg(verts, cx - arm, cy, cx - gap, cy, 1.5, fg, w, h);
    line_seg(verts, cx + gap, cy, cx + arm, cy, 1.5, fg, w, h);
    line_seg(verts, cx, cy - arm, cx, cy - gap, 1.5, fg, w, h);
    line_seg(verts, cx, cy + gap, cx, cy + arm, 1.5, fg, w, h);

    let gate_s = 30.0 + (time * 0.5).sin() * 5.0;
    let gx = cx + state.sensor_az * 2.0;
    let gy = cy - state.sensor_el * 0.5 - 20.0;
    let c = [0.9, 0.9, 0.9, 0.6];
    let hs = gate_s * 0.5;
    let corner = gate_s * 0.2;
    line_seg(verts, gx-hs, gy-hs, gx-hs+corner, gy-hs, 1.5, c, w, h);
    line_seg(verts, gx-hs, gy-hs, gx-hs, gy-hs+corner, 1.5, c, w, h);
    line_seg(verts, gx+hs-corner, gy-hs, gx+hs, gy-hs, 1.5, c, w, h);
    line_seg(verts, gx+hs, gy-hs, gx+hs, gy-hs+corner, 1.5, c, w, h);
    line_seg(verts, gx-hs, gy+hs, gx-hs+corner, gy+hs, 1.5, c, w, h);
    line_seg(verts, gx-hs, gy+hs-corner, gx-hs, gy+hs, 1.5, c, w, h);
    line_seg(verts, gx+hs-corner, gy+hs, gx+hs, gy+hs, 1.5, c, w, h);
    line_seg(verts, gx+hs, gy+hs-corner, gx+hs, gy+hs, 1.5, c, w, h);

    for y_off in (0..(ph as i32)).step_by(3) {
        let noise = ((time * 5.0 + y_off as f32 * 0.3).sin() * 0.015).abs();
        rect_filled(verts, px, py + y_off as f32, pw, 1.0, [0.5, 0.5, 0.5, noise], w, h);
    }
}

fn build_map_panel(verts: &mut Vec<Vertex>, state: &ReaperState, px: f32, py: f32, pw: f32, ph: f32, w: f32, h: f32, time: f32, green: [f32; 4], dim: [f32; 4]) {
    rect_filled(verts, px, py, pw, ph, [0.02, 0.05, 0.02, 0.8], w, h);
    let cx = px + pw * 0.5;
    let cy = py + ph * 0.5;
    let grid_sp = 40.0;
    let mut gx = px;
    while gx < px + pw { line_seg(verts, gx, py, gx, py + ph, 0.5, [0.0, 0.3, 0.1, 0.15], w, h); gx += grid_sp; }
    let mut gy = py;
    while gy < py + ph { line_seg(verts, px, gy, px + pw, gy, 0.5, [0.0, 0.3, 0.1, 0.15], w, h); gy += grid_sp; }

    let uav_size = 8.0;
    let a = state.heading.to_radians();
    let tip = [cx + a.sin() * uav_size, cy - a.cos() * uav_size];
    let left = [cx + (a + 2.5).sin() * uav_size * 0.6, cy - (a + 2.5).cos() * uav_size * 0.6];
    let right = [cx + (a - 2.5).sin() * uav_size * 0.6, cy - (a - 2.5).cos() * uav_size * 0.6];
    push_tri(verts, tip, left, right, green, w, h);

    let orbit_r = 60.0;
    circle_outline_fn(verts, cx, cy, orbit_r, 1.0, [dim[0], dim[1], dim[2], dim[3] * 0.5], 32, w, h);

    rect_filled(verts, px + 5.0, py + ph - 20.0, pw - 10.0, 16.0, [0.0, 0.06, 0.02, 0.5], w, h);
    let link_w = (pw - 14.0) * state.link_quality;
    let link_c = if state.link_quality > 0.8 { green } else { [1.0, 0.6, 0.0, 0.7] };
    rect_filled(verts, px + 7.0, py + ph - 18.0, link_w, 12.0, link_c, w, h);
}

fn build_weapons_panel(verts: &mut Vec<Vertex>, state: &ReaperState, px: f32, py: f32, pw: f32, ph: f32, w: f32, h: f32, time: f32, green: [f32; 4], dim: [f32; 4]) {
    rect_filled(verts, px, py, pw, ph, [0.03, 0.02, 0.02, 0.8], w, h);

    let status_c = match state.weapon_status {
        WeaponStatus::Safe => dim,
        WeaponStatus::Armed => { let p = 0.6 + (time * 2.0).sin() * 0.3; [1.0, 0.5, 0.0, p] }
        WeaponStatus::Firing => { let p = 0.7 + (time * 6.0).sin() * 0.3; [1.0, 0.15, 0.1, p] }
    };
    rect_filled(verts, px + 10.0, py + 10.0, 30.0, 16.0, status_c, w, h);

    for i in 0..4 {
        let sx = px + 10.0 + i as f32 * (pw * 0.22);
        let sy = py + 50.0;
        let sw = pw * 0.18;
        let sh = ph * 0.3;
        rect_filled(verts, sx, sy, sw, sh, [0.05, 0.05, 0.05, 0.5], w, h);
        if i < 2 {
            rect_filled(verts, sx + 2.0, sy + 2.0, sw - 4.0, sh - 4.0, [0.0, 0.4, 0.15, 0.3], w, h);
        } else {
            rect_filled(verts, sx + 2.0, sy + 2.0, sw - 4.0, sh - 4.0, [0.2, 0.2, 0.2, 0.15], w, h);
        }
    }

    let fuel_x = px + 10.0;
    let fuel_y = py + ph - 30.0;
    let fuel_w = pw - 20.0;
    rect_filled(verts, fuel_x, fuel_y, fuel_w, 14.0, [0.1, 0.1, 0.1, 0.4], w, h);
    let fuel_fill = fuel_w * state.fuel_pct;
    let fuel_c = if state.fuel_pct > 0.3 { green } else { [1.0, 0.3, 0.0, 0.7] };
    rect_filled(verts, fuel_x, fuel_y, fuel_fill, 14.0, fuel_c, w, h);
}

fn circle_outline_fn(verts: &mut Vec<Vertex>, cx:f32,cy:f32,r:f32,thick:f32,color:[f32;4],segs:usize,w:f32,h:f32) {
    let tau=std::f32::consts::TAU;
    for i in 0..segs {
        let a0=tau*i as f32/segs as f32;let a1=tau*(i+1)as f32/segs as f32;
        let ri=r-thick*0.5;let ro=r+thick*0.5;
        push_tri(verts,[cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ri,cy+a1.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],color,w,h);
        push_tri(verts,[cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],[cx+a0.cos()*ro,cy+a0.sin()*ro],color,w,h);
    }
}

fn line_seg(verts:&mut Vec<Vertex>,x0:f32,y0:f32,x1:f32,y1:f32,thick:f32,color:[f32;4],w:f32,h:f32) {
    let dx=x1-x0;let dy=y1-y0;let len=(dx*dx+dy*dy).sqrt();if len<0.001{return}
    let nx=-dy/len*thick*0.5;let ny=dx/len*thick*0.5;
    push_tri(verts,[x0+nx,y0+ny],[x0-nx,y0-ny],[x1-nx,y1-ny],color,w,h);
    push_tri(verts,[x0+nx,y0+ny],[x1-nx,y1-ny],[x1+nx,y1+ny],color,w,h);
}

fn rect_filled(verts:&mut Vec<Vertex>,x:f32,y:f32,rw:f32,rh:f32,color:[f32;4],w:f32,h:f32) {
    push_tri(verts,[x,y],[x+rw,y],[x+rw,y+rh],color,w,h);
    push_tri(verts,[x,y],[x+rw,y+rh],[x,y+rh],color,w,h);
}

fn to_ndc(x:f32,y:f32,w:f32,h:f32)->[f32;2]{[x/w*2.0-1.0,1.0-y/h*2.0]}

fn push_tri(verts:&mut Vec<Vertex>,p0:[f32;2],p1:[f32;2],p2:[f32;2],color:[f32;4],w:f32,h:f32) {
    verts.push(Vertex{position:to_ndc(p0[0],p0[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p1[0],p1[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p2[0],p2[1],w,h),color});
}
