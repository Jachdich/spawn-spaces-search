use rustmatica::util::{UVec3, Vec3};
use rustmatica::BlockState;
use std::cmp::{min, max};

const MAX_X: usize = 368;
const MAX_Z: usize = 368;
const ITSMAX: usize = MAX_X * 19 * MAX_Z;

fn check_position(pos: UVec3, spawns: &[u64]) -> usize {
    let mut count = 0;
    let pos = Vec3::new(pos.x as i32, pos.y as i32, pos.z as i32);

    // optimisation: only check blocks < 128 away in any direction
    let xmin = max(pos.x - 128, 0);
    let xmax = min(pos.x + 128, MAX_X as i32 - 1);
    let zmin = max(pos.z - 128, 0);
    let zmax = min(pos.z + 128, MAX_Z as i32 - 1);
    // loop through all possible blocks around the player and count if they're spawnable
    for x in xmin..=xmax {
        for y in 0..18 {
            for z in zmin..=zmax {
                let dx = pos.x - x;
                let dy = pos.y - y;
                let dz = pos.z - z;
                let index = (z * 368 * 18 + y * 368 + x) as usize;
                let d2 = dx * dx + dy * dy + dz * dz; // dist^2

                // branchless hack :trollface:
                count += (
                        d2 > 24 * 24 // outside inner no-spawn sphere
                        && d2 <= 128 * 128 // inside outer despawn sphere
                        && (spawns[index / 64] & (0b1 << (index % 64))) != 0 // calc index of required int, get required bit to check if spawnable
                    ) as usize;
            }
        }
    }
    count
}

fn main() {
    let schem = rustmatica::Litematic::read_file("/home/james/.minecraft/schematics/slime spawning areas.litematic").unwrap();
    let reg = &schem.regions[0];
    
    // which blocks to count, 64 block positions stored per u64 in each bit
    // basically each entry in this array is 64 entries really, and the xyz is
    // used to calculate effectively a bit index
    let mut is_spawnable = [0 as u64; 18*MAX_X*MAX_Z / 64];
    for x in reg.x_range() {
        for y in 0..18 {
            for z in reg.z_range() {
                let idx = z * (MAX_X * 18) + y * MAX_X + x;
                match reg.get_block(UVec3::new(x as usize, y as usize, z as usize)) {
                    BlockState::SmoothStoneSlab{..} | BlockState::SpruceSlab{..} => is_spawnable[idx / 64] |= 1 << (idx % 64),
                    _ => (),
                }
            }
        }
    }

    println!("Starting");
    let mut max_pos = UVec3::new(0, 0, 0);
    let mut max = 0;
    let mut its = 0;

    let start_time = std::time::Instant::now();

    // loop through all possible player positions
    for x in reg.x_range() {
        for y in 0..=20 {
            for z in reg.z_range() {
                let pos = UVec3::new(x, y, z);

                // check valid spawning coords
                let res = check_position(pos, &is_spawnable.clone());
                if res > max {
                    max = res;
                    max_pos = pos;
                }
                its += 1;
            }
            println!("{}/{} = {}%, {}it/{}s = {}it/s", its, ITSMAX, its as f64 / ITSMAX as f64 * 100.0, its, start_time.elapsed().as_secs(), its as f64 / (start_time.elapsed().as_millis() as f64 / 1000.0));
        }
    }
    println!("Max pos {:?} with {} spaces", max_pos, max);
}
