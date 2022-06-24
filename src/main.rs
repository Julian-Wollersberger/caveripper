use std::error::Error;
use std::num::ParseIntError;
use cavegen::caveinfo::{FloorInfo, ALL_SUBLEVELS_MAP};
use cavegen::layout::{Layout, SpawnObject};
use cavegen::layout::render::render_layout;
use once_cell::sync::Lazy;
use rand::random;
use simple_logger::SimpleLogger;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    if cfg!(debug_assertions) {
        //SimpleLogger::new().with_level(log::LevelFilter::max()).init()?;
    }

    /*let args = Args::from_args();
    let caveinfo = caveinfo_from_str(&args.sublevel).unwrap();
    let seed: u32 = from_hex_str(&args.seed)?;
    */
    println!("Starting...");
    let caveinfo_gk3 = caveinfo_from_str("GK3").unwrap();
    let caveinfo_scx7 = caveinfo_from_str("SCx7").unwrap();

    for i in 0..1_000_000 {
        let seed = random();

        let layout = Layout::generate(seed, caveinfo_gk3);
        //println!("{}", layout.slug());
        //render_layout(&layout);

        let treasures = count_treasures(layout);

        if treasures != 2 {
            let scx = Layout::generate(seed, caveinfo_scx7);
            let scx_count = count_treasures(scx);
            //println!("seed = {seed}, num treasures = {}, in scx7 = {}", treasures, scx_count);

            if scx_count != 3 {
                println!("seed = {:#X}, num treasures = {}, in scx7 = {}", seed, treasures, scx_count);
                //println!("-------   Found cursed seed!    -----------");
            }
        }
        if i % 100_000 == 0 {
            println!("Searched {i} seeds"); //43:00, 500_000 in 80sec, 1_000_000 in
        }
    }
    println!("Finished");

    // Example output:
    // Starting...
    // Searched 0 seeds
    // seed = 3002853760, num treasures = 1, in scx7 = 2
    // seed = 3336185623, num treasures = 1, in scx7 = 2
    // Searched 100000 seeds
    // seed = 1904794236, num treasures = 1, in scx7 = 2
    // seed = 319519892, num treasures = 1, in scx7 = 2
    // seed = 2949271994, num treasures = 1, in scx7 = 2
    // Searched 200000 seeds
    // seed = 2911967097, num treasures = 1, in scx7 = 2
    // Searched 300000 seeds
    // seed = 1610129740, num treasures = 1, in scx7 = 2
    // Searched 400000 seeds
    // seed = 76606336, num treasures = 1, in scx7 = 2
    // Searched 500000 seeds
    // Searched 600000 seeds
    // seed = 1001964962, num treasures = 1, in scx7 = 2
    // Searched 700000 seeds
    // seed = 571380122, num treasures = 1, in scx7 = 2
    // Searched 800000 seeds
    // Searched 900000 seeds
    // seed = 1548112145, num treasures = 1, in scx7 = 2
    // Finished
    
    Ok(())
}

fn count_treasures(layout: Layout) -> u32 {
    //let mut ship_coords = (f32::NAN, f32::NAN);
    //let mut treasure_coords = Vec::new();
    let mut treasures = 0;

    for map_unit in layout.map_units.iter() {
        for spawn_point in map_unit.spawnpoints.iter() {
            if let Some(spawn_object) = spawn_point.contains.as_ref() {
                match &spawn_object {
                    /*SpawnObject::Ship => {
                        ship_coords = (spawn_point.x, spawn_point.z)
                    }*/
                    SpawnObject::Item(_) => {
                        //treasure_coords.push((spawn_point.x, spawn_point.z))
                        treasures += 1;
                    },
                    SpawnObject::Teki(teki_info) => {
                        if teki_info.carrying.is_some() {
                            treasures += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    //dbg!(ship_coords);
    //dbg!(treasure_coords);

    return treasures;
}


#[derive(StructOpt)]
struct Args {
    #[structopt()]
    sublevel: String,

    #[structopt()]
    seed: String,
}

fn from_hex_str(src: &str) -> Result<u32, ParseIntError> {
    u32::from_str_radix(src.strip_prefix("0x").unwrap_or(src), 16)
}

fn caveinfo_from_str(cave: &str) -> Option<&'static Lazy<FloorInfo>> {
    ALL_SUBLEVELS_MAP.get(&cave.to_ascii_lowercase()).cloned()
}
