use macroquad::prelude::*;
use std::collections::HashMap;

const W: usize = 96;
const H: usize = 48;
const D: usize = 96;
const CS: i32 = 16;
const NC: i32 = (W as i32) / CS;
const HOT: [u8; 7] = [2, 3, 4, 7, 6, 5, 1];

type Meshes = HashMap<(i32, i32), Vec<Mesh>>;

fn hash(x: i32, z: i32) -> f32 {
    let mut n = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((z as u32).wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n ^ (n >> 16)) & 0xffff) as f32 / 65535.0
}

fn vnoise(x: f32, z: f32) -> f32 {
    let (xf, zf) = (x.floor(), z.floor());
    let (u, v) = (x - xf, z - zf);
    let (s, t) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (xi, zi) = (xf as i32, zf as i32);
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    l(
        l(hash(xi, zi), hash(xi + 1, zi), s),
        l(hash(xi, zi + 1), hash(xi + 1, zi + 1), s),
        t,
    )
}

struct World {
    b: Vec<u8>,
}

impl World {
    fn inb(x: i32, y: i32, z: i32) -> bool {
        x >= 0 && y >= 0 && z >= 0 && x < W as i32 && y < H as i32 && z < D as i32
    }
    fn idx(x: i32, y: i32, z: i32) -> usize {
        x as usize + W * (z as usize + D * y as usize)
    }
    fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if Self::inb(x, y, z) {
            self.b[Self::idx(x, y, z)]
        } else {
            0
        }
    }
    fn put(&mut self, x: i32, y: i32, z: i32, v: u8) {
        if Self::inb(x, y, z) {
            let i = Self::idx(x, y, z);
            self.b[i] = v;
        }
    }
    fn solid(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || z < 0 || y < 0 || x >= W as i32 || z >= D as i32 {
            return true;
        }
        self.get(x, y, z) > 0
    }
    fn generate() -> World {
        let mut w = World { b: vec![0; W * H * D] };
        for x in 0..W as i32 {
            for z in 0..D as i32 {
                let (fx, fz) = (x as f32, z as f32);
                let h = (11.0 + vnoise(fx / 26.0, fz / 26.0) * 15.0 + vnoise(fx / 9.0, fz / 9.0) * 4.0) as i32;
                let beach = h <= 13;
                for y in 0..=h {
                    let b = if y == h {
                        if beach { 6 } else { 1 }
                    } else if y + 4 > h {
                        if beach { 6 } else { 2 }
                    } else {
                        3
                    };
                    w.put(x, y, z, b);
                }
                if !beach && hash(x * 3, z * 7) > 0.985 && x > 3 && z > 3 && x < W as i32 - 4 && z < D as i32 - 4 {
                    let t = 4;
                    for y in 1..=t {
                        w.put(x, h + y, z, 4);
                    }
                    for dx in -2i32..=2 {
                        for dz in -2i32..=2 {
                            for dy in (t - 1)..=(t + 1) {
                                let extra = if dy > t { 2 } else { 0 };
                                if dx.abs() + dz.abs() + extra > 3 {
                                    continue;
                                }
                                if w.get(x + dx, h + dy, z + dz) == 0 {
                                    w.put(x + dx, h + dy, z + dz, 5);
                                }
                            }
                        }
                    }
                }
            }
        }
        w
    }
    fn hit(&self, p: Vec3) -> bool {
        for x in (p.x - 0.29).floor() as i32..=(p.x + 0.29).floor() as i32 {
            for y in p.y.floor() as i32..=(p.y + 1.79).floor() as i32 {
                for z in (p.z - 0.29).floor() as i32..=(p.z + 0.29).floor() as i32 {
                    if self.solid(x, y, z) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

const N: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
const SH: [f32; 6] = [0.82, 0.82, 1.0, 0.5, 0.68, 0.68];
const V: [[[f32; 3]; 4]; 6] = [
    [[1., 0., 0.], [1., 1., 0.], [1., 1., 1.], [1., 0., 1.]],
    [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
    [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]],
    [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
    [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
    [[0., 0., 0.], [0., 1., 0.], [1., 1., 0.], [1., 0., 0.]],
];

// block colours: [top, side, bottom]
fn base(b: u8, part: usize) -> [f32; 3] {
    let c: [[u8; 3]; 3] = match b {
        1 => [[95, 174, 58], [120, 128, 59], [107, 74, 43]],
        2 => [[122, 82, 48]; 3],
        3 => [[139, 141, 144], [139, 141, 144], [122, 124, 127]],
        4 => [[160, 122, 69], [110, 79, 42], [160, 122, 69]],
        5 => [[47, 125, 50]; 3],
        6 => [[226, 210, 143]; 3],
        _ => [[199, 154, 90]; 3],
    };
    let [r, g, bl] = c[part];
    [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0]
}

fn build_chunk(w: &World, cx: i32, cz: i32) -> Vec<Mesh> {
    let mut out = Vec::new();
    let mut vs: Vec<Vertex> = Vec::new();
    let mut is: Vec<u16> = Vec::new();
    for y in 0..H as i32 {
        for z in cz * CS..cz * CS + CS {
            for x in cx * CS..cx * CS + CS {
                let b = w.get(x, y, z);
                if b == 0 {
                    continue;
                }
                for f in 0..6 {
                    let n = N[f];
                    if w.get(x + n[0], y + n[1], z + n[2]) != 0 {
                        continue;
                    }
                    let part = if f == 2 { 0 } else if f == 3 { 2 } else { 1 };
                    let c = base(b, part);
                    let s = SH[f] * (0.92 + hash(x * 7 + f as i32, y * 13 + z) * 0.16);
                    let col = Color::new(c[0] * s, c[1] * s, c[2] * s, 1.0);
                    let o = vs.len() as u16;
                    for v in V[f].iter() {
                        vs.push(Vertex::new(x as f32 + v[0], y as f32 + v[1], z as f32 + v[2], 0.0, 0.0, col));
                    }
                    is.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3]);
                    if vs.len() >= 2400 {
                        out.push(Mesh {
                            vertices: std::mem::take(&mut vs),
                            indices: std::mem::take(&mut is),
                            texture: None,
                        });
                    }
                }
            }
        }
    }
    if !vs.is_empty() {
        out.push(Mesh { vertices: vs, indices: is, texture: None });
    }
    out
}

fn set_block(w: &mut World, m: &mut Meshes, x: i32, y: i32, z: i32, v: u8) {
    if !World::inb(x, y, z) {
        return;
    }
    w.put(x, y, z, v);
    let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
    let mut list = vec![(cx, cz)];
    if x.rem_euclid(CS) == 0 { list.push((cx - 1, cz)); }
    if x.rem_euclid(CS) == CS - 1 { list.push((cx + 1, cz)); }
    if z.rem_euclid(CS) == 0 { list.push((cx, cz - 1)); }
    if z.rem_euclid(CS) == CS - 1 { list.push((cx, cz + 1)); }
    for (a, b) in list {
        if a >= 0 && b >= 0 && a < NC && b < NC {
            m.insert((a, b), build_chunk(w, a, b));
        }
    }
}

fn act(w: &mut World, m: &mut Meshes, eye: Vec3, f: Vec3, place: bool, sel: u8, pos: Vec3) {
    let mut prev: Option<(i32, i32, i32)> = None;
    let mut t = 0.0;
    while t < 6.0 {
        let q = eye + f * t;
        let c = (q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32);
        if w.get(c.0, c.1, c.2) > 0 {
            if place {
                if let Some(pc) = prev {
                    set_block(w, m, pc.0, pc.1, pc.2, sel);
                    if w.hit(pos) {
                        set_block(w, m, pc.0, pc.1, pc.2, 0);
                    }
                }
            } else if c.1 > 0 {
                set_block(w, m, c.0, c.1, c.2, 0);
            }
            return;
        }
        prev = Some(c);
        t += 0.04;
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Blockworld".to_string(),
        window_width: 1024,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    simulate_mouse_with_touch(false);
    let mut world = World::generate();
    let mut meshes: Meshes = HashMap::new();
    for cx in 0..NC {
        for cz in 0..NC {
            meshes.insert((cx, cz), build_chunk(&world, cx, cz));
        }
    }
    let spawn = || {
        let mut y = H as i32 - 1;
        while y > 0 && world.get(48, y, 48) == 0 {
            y -= 1;
        }
        vec3(48.5, y as f32 + 1.01, 48.5)
    };
    let mut pos = spawn();
    let (mut yaw, mut pitch, mut vy) = (0.0f32, -0.2f32, 0.0f32);
    let (mut ground, mut place, mut sel) = (false, false, 0usize);
    let (mut joy_id, mut joy_o, mut joy_v) = (None::<u64>, Vec2::ZERO, Vec2::ZERO);
    let (mut look_id, mut look_last, mut look_d, mut look_t) = (None::<u64>, Vec2::ZERO, 0.0f32, 0.0f64);
    let mut jump_id = None::<u64>;
    let mut last_mp = mouse_position_local();
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        let dt = get_frame_time().min(0.05);
        let (sw, sh) = (screen_width(), screen_height());
        let jump_c = vec2(sw - 80.0, sh - 100.0);
        let mode_r = Rect::new(sw - 130.0, 10.0, 120.0, 44.0);
        let slot = |i: usize| Rect::new((sw - 344.0) / 2.0 + i as f32 * 50.0, sh - 54.0, 44.0, 44.0);
        let mut do_act: Option<bool> = None;

        for t in touches() {
            match t.phase {
                TouchPhase::Started => {
                    let p = t.position;
                    if p.distance(jump_c) < 55.0 {
                        jump_id = Some(t.id);
                    } else if mode_r.contains(p) {
                        place = !place;
                    } else if let Some(i) = (0..7).find(|i| slot(*i).contains(p)) {
                        sel = i;
                    } else if p.x < sw * 0.4 && joy_id.is_none() {
                        joy_id = Some(t.id);
                        joy_o = p;
                        joy_v = Vec2::ZERO;
                    } else if look_id.is_none() {
                        look_id = Some(t.id);
                        look_last = p;
                        look_d = 0.0;
                        look_t = get_time();
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(t.id) == joy_id {
                        joy_v = ((t.position - joy_o) / 55.0).clamp_length_max(1.0);
                    } else if Some(t.id) == look_id {
                        let d = t.position - look_last;
                        look_last = t.position;
                        look_d += d.length();
                        yaw -= d.x * 0.006;
                        pitch = (pitch - d.y * 0.006).clamp(-1.5, 1.5);
                    }
                }
                _ => {
                    if Some(t.id) == joy_id { joy_id = None; joy_v = Vec2::ZERO; }
                    if Some(t.id) == jump_id { jump_id = None; }
                    if Some(t.id) == look_id {
                        if t.phase == TouchPhase::Ended && look_d < 14.0 && get_time() - look_t < 0.32 {
                            do_act = Some(place);
                        }
                        look_id = None;
                    }
                }
            }
        }

        // keyboard + mouse
        let mut fwd = -joy_v.y;
        let mut strf = joy_v.x;
        if is_key_down(KeyCode::W) { fwd += 1.0; }
        if is_key_down(KeyCode::S) { fwd -= 1.0; }
        if is_key_down(KeyCode::D) { strf += 1.0; }
        if is_key_down(KeyCode::A) { strf -= 1.0; }
        for (i, k) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6, KeyCode::Key7].iter().enumerate() {
            if is_key_pressed(*k) { sel = i; }
        }
        let mp = mouse_position_local();
        let d = mp - last_mp;
        last_mp = mp;
        yaw -= d.x * 2.5;
        pitch = (pitch + d.y * 2.5).clamp(-1.5, 1.5);
        if is_mouse_button_pressed(MouseButton::Left) { do_act = Some(false); }
        if is_mouse_button_pressed(MouseButton::Right) { do_act = Some(true); }

        // physics
        let (sy, cy) = yaw.sin_cos();
        let mut wish = vec3(-sy * fwd + cy * strf, 0.0, -cy * fwd - sy * strf);
        if wish.length() > 1.0 { wish = wish.normalize(); }
        let mv = wish * 5.0 * dt;
        pos.x += mv.x;
        if world.hit(pos) { pos.x -= mv.x; }
        pos.z += mv.z;
        if world.hit(pos) { pos.z -= mv.z; }
        if (jump_id.is_some() || is_key_down(KeyCode::Space)) && ground { vy = 8.2; }
        vy -= 26.0 * dt;
        pos.y += vy * dt;
        ground = false;
        if world.hit(pos) {
            pos.y -= vy * dt;
            if vy < 0.0 { ground = true; }
            vy = 0.0;
        }
        if pos.y < -20.0 { pos = spawn(); vy = 0.0; }

        let eye = pos + vec3(0.0, 1.62, 0.0);
        let f = vec3(-sy * pitch.cos(), pitch.sin(), -cy * pitch.cos());
        if let Some(pl) = do_act {
            act(&mut world, &mut meshes, eye, f, pl, HOT[sel], pos);
        }

        // render
        clear_background(Color::new(0.53, 0.75, 0.95, 1.0));
        set_camera(&Camera3D {
            position: eye,
            target: eye + f,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 1.2,
            ..Default::default()
        });
        for ms in meshes.values() {
            for m in ms {
                draw_mesh(m);
            }
        }
        set_default_camera();
        draw_line(sw / 2.0 - 9.0, sh / 2.0, sw / 2.0 + 9.0, sh / 2.0, 2.0, WHITE);
        draw_line(sw / 2.0, sh / 2.0 - 9.0, sw / 2.0, sh / 2.0 + 9.0, 2.0, WHITE);
        for i in 0..7 {
            let r = slot(i);
            let c = base(HOT[i], 0);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.05, 0.1, 0.05, 0.6));
            draw_rectangle(r.x + 9.0, r.y + 9.0, 26.0, 26.0, Color::new(c[0], c[1], c[2], 1.0));
            if i == sel {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 4.0, Color::new(1.0, 0.82, 0.29, 1.0));
            }
        }
        draw_rectangle(mode_r.x, mode_r.y, mode_r.w, mode_r.h, Color::new(0.05, 0.1, 0.05, 0.6));
        draw_text(if place { "PLACE" } else { "BREAK" }, mode_r.x + 22.0, mode_r.y + 30.0, 30.0, WHITE);
        draw_circle(jump_c.x, jump_c.y, 55.0, Color::new(1.0, 1.0, 1.0, if jump_id.is_some() { 0.35 } else { 0.18 }));
        draw_text("JUMP", jump_c.x - 25.0, jump_c.y + 8.0, 28.0, WHITE);
        if joy_id.is_some() {
            draw_circle_lines(joy_o.x, joy_o.y, 55.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.5));
            let k = joy_o + joy_v * 55.0;
            draw_circle(k.x, k.y, 22.0, Color::new(1.0, 1.0, 1.0, 0.4));
        }
        next_frame().await;
    }
}
