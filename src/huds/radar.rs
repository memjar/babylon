use crate::renderer::Vertex;
use crate::hud::{Contact, ThreatLevel};

pub fn build_radar(verts: &mut Vec<Vertex>, contacts: &[Contact], cx: f32, cy: f32, radius: f32, w: f32, h: f32, time: f32) {
    build_rings(verts, cx, cy, radius, w, h);
    build_cardinals(verts, cx, cy, radius, w, h);
    build_sweep(verts, cx, cy, radius, w, h, time);
    build_blips(verts, contacts, cx, cy, radius, w, h, time);
}

fn build_rings(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32) {
    let color = [0.0, 0.7, 0.4, 0.2];
    for ring in 1..=4 {
        let rr = r * ring as f32 / 4.0;
        circle_outline(verts, cx, cy, rr, 1.0, color, 64, w, h);
    }
}

fn build_cardinals(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32) {
    let color = [0.0, 0.6, 0.3, 0.3];
    let tick = 8.0;
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2;
        let x0 = cx + a.cos() * (r - tick);
        let y0 = cy + a.sin() * (r - tick);
        let x1 = cx + a.cos() * r;
        let y1 = cy + a.sin() * r;
        line_seg(verts, x0, y0, x1, y1, 1.5, color, w, h);
    }
}

fn build_sweep(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
    let angle = time * 0.8;
    let steps = 16;
    let arc = 0.35;
    for i in 0..steps {
        let a0 = angle - arc * i as f32 / steps as f32;
        let a1 = angle - arc * (i + 1) as f32 / steps as f32;
        let alpha = 0.12 * (1.0 - i as f32 / steps as f32);
        let color = [0.0, 1.0, 0.5, alpha];
        let p0 = [cx, cy];
        let p1 = [cx + a0.cos() * r, cy + a0.sin() * r];
        let p2 = [cx + a1.cos() * r, cy + a1.sin() * r];
        push_tri(verts, p0, p1, p2, color, w, h);
    }
}

fn build_blips(verts: &mut Vec<Vertex>, contacts: &[Contact], cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
    for (i, c) in contacts.iter().enumerate() {
        let px = cx + c.x * r;
        let py = cy + c.y * r;
        let pulse = 1.0 + (time * 3.0 + i as f32 * 1.5).sin() * 0.25;
        let size = 4.0 * pulse;
        let color = c.threat.color();
        match c.threat {
            ThreatLevel::Hostile => diamond(verts, px, py, size * 1.3, color, w, h),
            ThreatLevel::Friendly => tri_up(verts, px, py, size, color, w, h),
            _ => rect_centered(verts, px, py, size * 2.0, size * 2.0, color, w, h),
        }
    }
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

fn diamond(verts: &mut Vec<Vertex>, cx: f32, cy: f32, s: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri(verts, [cx, cy - s], [cx + s, cy], [cx, cy + s], color, w, h);
    push_tri(verts, [cx, cy - s], [cx - s, cy], [cx, cy + s], color, w, h);
}

fn tri_up(verts: &mut Vec<Vertex>, cx: f32, cy: f32, s: f32, color: [f32; 4], w: f32, h: f32) {
    push_tri(verts, [cx, cy - s], [cx + s * 0.866, cy + s * 0.5], [cx - s * 0.866, cy + s * 0.5], color, w, h);
}

fn rect_centered(verts: &mut Vec<Vertex>, cx: f32, cy: f32, rw: f32, rh: f32, color: [f32; 4], w: f32, h: f32) {
    let x = cx - rw * 0.5;
    let y = cy - rh * 0.5;
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
