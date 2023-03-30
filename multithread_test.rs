use rustmatica::util::{UVec3, Vec3};
use rustmatica::BlockState;
use std::cmp::{min, max};

const MAX_X: usize = 368;
const MAX_Z: usize = 368;
const ITSMAX: usize = MAX_X * 20 * MAX_Z;

fn check_position(pos: UVec3, spawns: &[u64]) -> usize {
    let mut count = 0;
    let pos = Vec3::new(pos.x as i32, pos.y as i32, pos.z as i32);
    let xmin = max(pos.x - 128, 0);
    let xmax = min(pos.x + 128, MAX_X as i32 - 1);
    let zmin = max(pos.z - 128, 0);
    let zmax = min(pos.z + 128, MAX_Z as i32 - 1);
    for x in xmin..=xmax {
        for y in 0..20 {
            for z in zmin..=zmax {
                let dx = pos.x - x;
                let dy = pos.y - y;
                let dz = pos.z - z;
                let index = (z * 368 * 20 + y * 368 + x) as usize;
                let d2 = dx * dx + dy * dy + dz * dz;
                count += (
                        d2 > 24 * 24
                        && d2 <= 128 * 128
                        && (spawns[index / 64] & (0b1 << (index % 64))) != 0
                    ) as usize;
            }
        }
    }
    count
}

fn check_x_range(x_s: usize, x_e: usize, spawns: &[u64; ITSMAX / 64], tx: std::sync::mpsc::Sender<(usize, UVec3)>) {
    
    let mut max_pos = UVec3::new(0, 0, 0);
    let mut max = 0;
    let mut its = 0;

    let start_time = std::time::Instant::now();
    
    for x in x_s..x_e {
        for y in 0..20 {
            for z in 0..MAX_Z {
                let pos = UVec3::new(x, y, z);
                let res = check_position(pos, spawns);
                if res > max {
                    max = res;
                    max_pos = pos;
                }
                its += 1;
            }
            println!("{}/{} = {}%, {}it/{}s = {}it/s", its, x_e-x_s, its as f64 / (x_e-x_s) as f64 * 100.0, its, start_time.elapsed().as_secs(), its as f64 / (start_time.elapsed().as_millis() as f64 / 1000.0));
        }
    }
}

fn main() {
    let schem = rustmatica::Litematic::read_file("/home/james/.minecraft/schematics/slime spawning areas.litematic").unwrap();
    let reg = &schem.regions[0];
    println!("{} {}", reg.max_x(), reg.max_z());

    let mut is_spawnable = [0 as u64; ITSMAX / 64];
    for x in reg.x_range() {
        for y in 0..20 {
            for z in reg.z_range() {
                let idx = z * (MAX_X * 20) + y * MAX_X + x;
                if let BlockState::SmoothStoneSlab{..} = reg.get_block(UVec3::new(x as usize, y as usize, z as usize)) {
                    is_spawnable[idx / 64] |= 1 << (idx % 64);
                }
            }
        }
    }
    let (tx, rx) = std::sync::mpsc::channel::<(usize, UVec3)>();
    
    println!("Starting");

    let mut max_pos = UVec3::new(0, 0, 0);
    let mut max = 0;

    let step = ITSMAX / 12;

    for i in 0..12 {
        let tx2 = tx.clone();
        let spawns = is_spawnable.clone();
        std::thread::spawn(move || {
            check_x_range(step * i, min(step * i + step, ITSMAX), &spawns, tx2);
        });
    }

    for i in 0..12 {
        let res = rx.recv().unwrap();
        println!("Possible: {:?} {:?}", res.0, res.1);
    }
    
    println!("Max pos {:?} with {} spaces", max_pos, max);
}
