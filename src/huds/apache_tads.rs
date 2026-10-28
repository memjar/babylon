use crate::renderer::Vertex;

pub struct TadsState {
    pub fov: f32,
    pub zoom: f32,
    pub target_range: f32,
    pub laser_armed: bool,
    pub laser_firing: bool,
    pub az: f32,
    pub el: f32,
    pub gate_x: f32,
    pub gate_y: f32,
    pub gate_size: f32,
    pub whot: bool,
}

impl TadsState {
    pub fn demo() -> Self {
        Self {
            fov: 4.2,
            zoom: 16.0,
            target_range: 3200.0,
            laser_armed: true,
            laser_firing: false,
            az: 12.4,
            el: -3.8,
            gate_x: 0.0,
            gate_y: 0.0,
            gate_size: 40.0,
            whot: true,
        }
    }

    pub fn tick(&mut self, time: f32) {
        self.az = 12.4 + (time * 0.15).sin() * 2.0;
        self.el = -3.8 + (time * 0.1).cos() * 1.0;
        self.target_range = 3200.0 + (time * 0.08).sin() * 100.0;
        self.gate_x = (time * 0.2).sin() * 15.0;
        self.gate_y = (time * 0.15).cos() * 10.0;
        self.laser_firing = (time * 0.5).sin() > 0.8;
        self.zoom = 16.0 + (time * 0.05).sin() * 4.0;
    }
}

pub fn build_tads(verts: &mut Vec<Vertex>, state: &TadsState, w: f32, h: f32, time: f32) {
    let fg = [0.85, 0.85, 0.85, 0.9];
    let dim = [0.6, 0.6, 0.6, 0.5];
    let cx = w * 0.5;
    let cy = h * 0.5;

    build_flir_frame(verts, w, h, dim);
    build_crosshair(verts, cx, cy, w, h, fg);
    build_acquisition_gate(verts, cx + state.gate_x, cy + state.gate_y, state.gate_size, w, h, time, fg);
    build_range_bar(verts, state.target_range, w, h, fg, dim);
    build_status_block(verts, state, w, h, fg, dim);
    build_los_reticle(verts, cx, cy, w, h, dim);
    if state.laser_firing {
        build_laser_indicator(verts, w, h, time);
    }
    build_grayscale_noise(verts, w, h, time);
}

fn build_flir_frame(verts: &mut Vec<Vertex>, w: f32, h: f32, color: [f32; 4]) {
    let margin = 30.0;
    let tick = 12.0;
    let c = [color[0], color[1], color[2], color[3] * 0.3];
    for i in 0..4 {
        let (x0, y0, x1, y1) = match i {
            0 => (margin, margin, margin + tick, margin),
            1 => (margin, margin, margin, margin + tick),
            2 => (w - margin - tick, margin, w - margin, margin),
            3 => (w - margin, margin, w - margin, margin + tick),
            _ => unreachable!(),
        };
        line_seg(verts, x0, y0, x1, y1, 1.5, c, w, h);
    }
    for i in 0..4 {
        let (x0, y0, x1, y1) = match i {
            0 => (margin, h - margin, margin + tick, h - margin),
            1 => (margin, h - margin - tick, margin, h - margin),
            2 => (w - margin - tick, h - margin, w - margin, h - margin),
            3 => (w - margin, h - margin - tick, w - margin, h - margin),
            _ => unreachable!(),
        };
        line_seg(verts, x0, y0, x1, y1, 1.5, c, w, h);
    }
}

fn build_crosshair(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let gap = 15.0;
    let len = 35.0;
    line_seg(verts, cx - len, cy, cx - gap, cy, 1.5, color, w, h);
    line_seg(verts, cx + gap, cy, cx + len, cy, 1.5, color, w, h);
    line_seg(verts, cx, cy - len, cx, cy - gap, 1.5, color, w, h);
    line_seg(verts, cx, cy + gap, cx, cy + len, 1.5, color, w, h);
    circle_outline(verts, cx, cy, 3.0, 1.5, color, 12, w, h);
}

fn build_acquisition_gate(verts: &mut Vec<Vertex>, cx: f32, cy: f32, size: f32, w: f32, h: f32, time: f32, color: [f32; 4]) {
    let pulse = 0.7 + (time * 3.0).sin() * 0.3;
    let c = [color[0], color[1], color[2], color[3] * pulse];
    let hs = size * 0.5;
    let corner = size * 0.25;
    line_seg(verts, cx - hs, cy - hs, cx - hs + corner, cy - hs, 2.0, c, w, h);
    line_seg(verts, cx - hs, cy - hs, cx - hs, cy - hs + corner, 2.0, c, w, h);
    line_seg(verts, cx + hs - corner, cy - hs, cx + hs, cy - hs, 2.0, c, w, h);
    line_seg(verts, cx + hs, cy - hs, cx + hs, cy - hs + corner, 2.0, c, w, h);
    line_seg(verts, cx - hs, cy + hs - corner, cx - hs, cy + hs, 2.0, c, w, h);
    line_seg(verts, cx - hs, cy + hs, cx - hs + corner, cy + hs, 2.0, c, w, h);
    line_seg(verts, cx + hs, cy + hs - corner, cx + hs, cy + hs, 2.0, c, w, h);
    line_seg(verts, cx + hs - corner, cy + hs, cx + hs, cy + hs, 2.0, c, w, h);
}

fn build_range_bar(verts: &mut Vec<Vertex>, range: f32, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let bx = w - 60.0;
    let by = h * 0.2;
    let bh = h * 0.6;
    let bw = 8.0;
    rect_filled(verts, bx, by, bw, bh, [0.15, 0.15, 0.15, 0.3], w, h);
    let fill = (range / 6000.0).clamp(0.0, 1.0);
    let fh = bh * fill;
    rect_filled(verts, bx, by + bh - fh, bw, fh, [0.7, 0.7, 0.7, 0.5], w, h);
    for km in 0..=6 {
        let y = by + bh - bh * km as f32 / 6.0;
        line_seg(verts, bx + bw, y, bx + bw + 4.0, y, 1.0, dim, w, h);
    }
    let marker_y = by + bh - bh * (range / 6000.0).clamp(0.0, 1.0);
    line_seg(verts, bx - 4.0, marker_y, bx + bw + 4.0, marker_y, 2.0, color, w, h);
}

fn build_status_block(verts: &mut Vec<Vertex>, state: &TadsState, w: f32, h: f32, color: [f32; 4], dim: [f32; 4]) {
    let bx = 40.0;
    let by = h - 60.0;
    rect_filled(verts, bx, by, 160.0, 40.0, [0.05, 0.05, 0.05, 0.5], w, h);
    let lsr_c = if state.laser_armed { color } else { dim };
    rect_filled(verts, bx + 5.0, by + 5.0, 30.0, 12.0, [lsr_c[0], lsr_c[1], lsr_c[2], 0.3], w, h);
    let mode_c = if state.whot { color } else { dim };
    rect_filled(verts, bx + 40.0, by + 5.0, 30.0, 12.0, [mode_c[0], mode_c[1], mode_c[2], 0.3], w, h);
    rect_filled(verts, bx + 5.0, by + 22.0, 80.0, 12.0, [0.6, 0.6, 0.6, 0.2], w, h);
    let zoom_fill = (state.zoom / 32.0).clamp(0.0, 1.0) * 80.0;
    rect_filled(verts, bx + 5.0, by + 22.0, zoom_fill, 12.0, [0.7, 0.7, 0.7, 0.4], w, h);
}

fn build_los_reticle(verts: &mut Vec<Vertex>, cx: f32, cy: f32, w: f32, h: f32, color: [f32; 4]) {
    let r = 50.0;
    let c = [color[0], color[1], color[2], color[3] * 0.15];
    circle_outline(verts, cx, cy, r, 1.0, c, 32, w, h);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::FRAC_PI_4;
        let x0 = cx + a.cos() * (r - 3.0);
        let y0 = cy + a.sin() * (r - 3.0);
        let x1 = cx + a.cos() * (r + 3.0);
        let y1 = cy + a.sin() * (r + 3.0);
        line_seg(verts, x0, y0, x1, y1, 1.0, c, w, h);
    }
}

fn build_laser_indicator(verts: &mut Vec<Vertex>, w: f32, h: f32, time: f32) {
    let flash = ((time * 8.0).sin() + 1.0) * 0.5;
    let c = [1.0, 0.3, 0.3, flash * 0.7];
    rect_filled(verts, w - 120.0, 40.0, 80.0, 16.0, c, w, h);
}

fn build_grayscale_noise(verts: &mut Vec<Vertex>, w: f32, h: f32, time: f32) {
    let band_y = (time * 80.0) % h;
    let band_h = 2.0;
    let alpha = 0.02 + ((time * 7.0).sin() * 0.01).abs();
    rect_filled(verts, 0.0, band_y, w, band_h, [0.5, 0.5, 0.5, alpha], w, h);
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

fn line_seg(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, thick: f32, color: [f32; 4], w: f32, h: f32) {
    let dx=x1-x0;let dy=y1-y0;let len=(dx*dx+dy*dy).sqrt();
    if len<0.001{return}
    let nx=-dy/len*thick*0.5;let ny=dx/len*thick*0.5;
    push_tri(verts,[x0+nx,y0+ny],[x0-nx,y0-ny],[x1-nx,y1-ny],color,w,h);
    push_tri(verts,[x0+nx,y0+ny],[x1-nx,y1-ny],[x1+nx,y1+ny],color,w,h);
}

fn rect_filled(verts: &mut Vec<Vertex>, x: f32, y: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri(verts,[x,y],[x+rw,y],[x+rw,y+rh],color,w,h);
    push_tri(verts,[x,y],[x+rw,y+rh],[x,y+rh],color,w,h);
}

fn to_ndc(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [x/w*2.0-1.0,1.0-y/h*2.0]
}

fn push_tri(verts: &mut Vec<Vertex>, p0:[f32;2], p1:[f32;2], p2:[f32;2], color:[f32;4], w:f32, h:f32) {
    verts.push(Vertex{position:to_ndc(p0[0],p0[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p1[0],p1[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p2[0],p2[1],w,h),color});
}
