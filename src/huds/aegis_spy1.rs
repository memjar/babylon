use crate::renderer::Vertex;

pub struct AegisState {
    pub tracks: Vec<AegisTrack>,
    pub own_heading: f32,
    pub engagement_range: f32,
}

pub struct AegisTrack {
    pub bearing: f32,
    pub range_nm: f32,
    pub altitude: f32,
    pub hostile: bool,
    pub track_id: u16,
    pub speed_kts: f32,
}

impl AegisState {
    pub fn demo() -> Self {
        Self {
            tracks: vec![
                AegisTrack { bearing: 30.0, range_nm: 45.0, altitude: 25000.0, hostile: true, track_id: 7201, speed_kts: 480.0 },
                AegisTrack { bearing: 120.0, range_nm: 22.0, altitude: 500.0, hostile: true, track_id: 7203, speed_kts: 560.0 },
                AegisTrack { bearing: 210.0, range_nm: 60.0, altitude: 35000.0, hostile: false, track_id: 4102, speed_kts: 420.0 },
                AegisTrack { bearing: 300.0, range_nm: 35.0, altitude: 15000.0, hostile: false, track_id: 4105, speed_kts: 380.0 },
                AegisTrack { bearing: 75.0, range_nm: 80.0, altitude: 0.0, hostile: false, track_id: 5001, speed_kts: 18.0 },
                AegisTrack { bearing: 160.0, range_nm: 12.0, altitude: 100.0, hostile: true, track_id: 7210, speed_kts: 620.0 },
            ],
            own_heading: 0.0,
            engagement_range: 50.0,
        }
    }

    pub fn tick(&mut self, time: f32) {
        self.own_heading = (time * 0.05).sin() * 8.0;
        for (i, t) in self.tracks.iter_mut().enumerate() {
            t.bearing += (time * 0.1 + i as f32 * 0.7).sin() * 0.15;
            t.range_nm += (time * 0.08 + i as f32).cos() * 0.1;
        }
    }
}

pub fn build_aegis(verts: &mut Vec<Vertex>, state: &AegisState, w: f32, h: f32, time: f32) {
    let cx = w * 0.42;
    let cy = h * 0.5;
    let r = h.min(w * 0.7) * 0.4;
    let green = [0.0, 1.0, 0.4, 0.7];
    let dim = [0.0, 0.6, 0.25, 0.3];

    build_ppi(verts, cx, cy, r, w, h, dim, green);
    build_range_rings(verts, cx, cy, r, w, h, dim);
    build_engagement_zone(verts, cx, cy, r, state.engagement_range, w, h, time);
    build_sweep(verts, cx, cy, r, w, h, time);
    build_tracks(verts, &state.tracks, cx, cy, r, w, h, time);
    build_track_table(verts, &state.tracks, w, h, green, dim);
}

fn build_ppi(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, dim: [f32; 4], green: [f32; 4]) {
    circle_outline(verts, cx, cy, r, 2.0, green, 64, w, h);
    for i in 0..36 {
        let a = i as f32 * std::f32::consts::PI / 18.0;
        let r0 = if i % 3 == 0 { r - 10.0 } else { r - 5.0 };
        let x0 = cx + a.cos() * r0;
        let y0 = cy + a.sin() * r0;
        let x1 = cx + a.cos() * r;
        let y1 = cy + a.sin() * r;
        let c = if i % 9 == 0 { green } else { dim };
        line_seg(verts, x0, y0, x1, y1, 1.0, c, w, h);
    }
    line_seg(verts, cx - 4.0, cy, cx + 4.0, cy, 1.5, green, w, h);
    line_seg(verts, cx, cy - 4.0, cx, cy + 4.0, 1.5, green, w, h);
}

fn build_range_rings(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, dim: [f32; 4]) {
    for ring in 1..=4 {
        let rr = r * ring as f32 / 5.0;
        circle_outline(verts, cx, cy, rr, 0.8, [dim[0], dim[1], dim[2], dim[3] * 0.5], 48, w, h);
    }
}

fn build_engagement_zone(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, range_nm: f32, w: f32, h: f32, time: f32) {
    let er = r * (range_nm / 100.0).min(1.0);
    let pulse = 0.15 + (time * 1.5).sin() * 0.05;
    let c = [1.0, 0.3, 0.0, pulse];
    circle_outline(verts, cx, cy, er, 1.5, c, 48, w, h);
}

fn build_sweep(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
    let angle = time * 1.2;
    let steps = 24;
    let arc = 0.15;
    for i in 0..steps {
        let a0 = angle - arc * i as f32 / steps as f32;
        let a1 = angle - arc * (i + 1) as f32 / steps as f32;
        let alpha = 0.08 * (1.0 - i as f32 / steps as f32);
        push_tri(verts, [cx, cy], [cx + a0.cos() * r, cy + a0.sin() * r], [cx + a1.cos() * r, cy + a1.sin() * r], [0.0, 1.0, 0.4, alpha], w, h);
    }
}

fn build_tracks(verts: &mut Vec<Vertex>, tracks: &[AegisTrack], cx: f32, cy: f32, r: f32, w: f32, h: f32, time: f32) {
    for (i, t) in tracks.iter().enumerate() {
        let a = t.bearing.to_radians();
        let dist = (t.range_nm / 100.0).min(0.95);
        let px = cx + a.cos() * r * dist;
        let py = cy + a.sin() * r * dist;
        let pulse = 0.7 + (time * 2.5 + i as f32 * 1.8).sin() * 0.3;

        if t.hostile {
            let c = [1.0, 0.15, 0.1, pulse];
            diamond(verts, px, py, 6.0, c, w, h);
            let vel_a = a + 0.1;
            line_seg(verts, px, py, px + vel_a.cos() * 12.0, py + vel_a.sin() * 12.0, 1.0, [1.0, 0.3, 0.2, 0.5], w, h);
        } else if t.altitude < 10.0 {
            let c = [0.2, 0.6, 1.0, pulse * 0.7];
            rect_centered(verts, px, py, 8.0, 8.0, c, w, h);
        } else {
            let c = [0.0, 1.0, 0.5, pulse * 0.8];
            tri_up(verts, px, py, 5.0, c, w, h);
        }
    }
}

fn build_track_table(verts: &mut Vec<Vertex>, tracks: &[AegisTrack], w: f32, h: f32, green: [f32; 4], dim: [f32; 4]) {
    let tx = w * 0.72;
    let ty = 50.0;
    let tw = w * 0.25;
    let row_h = 20.0;
    let header_h = 24.0;

    rect_filled(verts, tx, ty, tw, header_h, [0.0, 0.08, 0.02, 0.5], w, h);
    line_seg(verts, tx, ty + header_h, tx + tw, ty + header_h, 1.0, green, w, h);

    for (i, t) in tracks.iter().enumerate() {
        let ry = ty + header_h + i as f32 * row_h;
        let bg = if i % 2 == 0 { [0.0, 0.04, 0.01, 0.3] } else { [0.0, 0.06, 0.02, 0.3] };
        rect_filled(verts, tx, ry, tw, row_h, bg, w, h);

        let status_c = if t.hostile { [1.0, 0.2, 0.1, 0.8] } else { [0.0, 0.8, 0.4, 0.6] };
        rect_filled(verts, tx + 4.0, ry + 6.0, 8.0, 8.0, status_c, w, h);

        let bar_w = (t.range_nm / 100.0).min(1.0) * 60.0;
        rect_filled(verts, tx + tw - 70.0, ry + 6.0, bar_w, 8.0, dim, w, h);
    }
    line_seg(verts, tx, ty, tx, ty + header_h + tracks.len() as f32 * row_h, 1.0, dim, w, h);
    line_seg(verts, tx + tw, ty, tx + tw, ty + header_h + tracks.len() as f32 * row_h, 1.0, dim, w, h);
}

fn circle_outline(verts: &mut Vec<Vertex>, cx: f32, cy: f32, r: f32, thick: f32, color: [f32;4], segs: usize, w: f32, h: f32) {
    let tau = std::f32::consts::TAU;
    for i in 0..segs {
        let a0=tau*i as f32/segs as f32;let a1=tau*(i+1)as f32/segs as f32;
        let ri=r-thick*0.5;let ro=r+thick*0.5;
        push_tri(verts,[cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ri,cy+a1.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],color,w,h);
        push_tri(verts,[cx+a0.cos()*ri,cy+a0.sin()*ri],[cx+a1.cos()*ro,cy+a1.sin()*ro],[cx+a0.cos()*ro,cy+a0.sin()*ro],color,w,h);
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

fn diamond(verts: &mut Vec<Vertex>, cx:f32,cy:f32,s:f32,color:[f32;4],w:f32,h:f32) {
    push_tri(verts,[cx,cy-s],[cx+s,cy],[cx,cy+s],color,w,h);
    push_tri(verts,[cx,cy-s],[cx-s,cy],[cx,cy+s],color,w,h);
}

fn tri_up(verts: &mut Vec<Vertex>, cx:f32,cy:f32,s:f32,color:[f32;4],w:f32,h:f32) {
    push_tri(verts,[cx,cy-s],[cx+s*0.866,cy+s*0.5],[cx-s*0.866,cy+s*0.5],color,w,h);
}

fn rect_centered(verts: &mut Vec<Vertex>, cx:f32,cy:f32,rw:f32,rh:f32,color:[f32;4],w:f32,h:f32) {
    rect_filled(verts,cx-rw*0.5,cy-rh*0.5,rw,rh,color,w,h);
}

fn to_ndc(x:f32,y:f32,w:f32,h:f32)->[f32;2]{[x/w*2.0-1.0,1.0-y/h*2.0]}

fn push_tri(verts:&mut Vec<Vertex>,p0:[f32;2],p1:[f32;2],p2:[f32;2],color:[f32;4],w:f32,h:f32) {
    verts.push(Vertex{position:to_ndc(p0[0],p0[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p1[0],p1[1],w,h),color});
    verts.push(Vertex{position:to_ndc(p2[0],p2[1],w,h),color});
}
