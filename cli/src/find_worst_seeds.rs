//! Find the worst or best overall seed, according to some simple heuristics.
//!
//! The idea is to sum up the treasure paths of all sublevels relevant to PoD.
//! Seeds get ranked by that sum.
//!
//! The path lengths are approximated by finding the smallest rectangle
//! that fits the ship, the hole and all treasures
//! and adding the width and height together.
//!
//! This means that close treasures don't increase the rank by much,
//! since multitasking them is easier. Also most sublevels have
//! a grid-like layout, meaning most paths are perpendicular to the axis anyway.
//! Fancy path length calculations wouldn't make the estimates that much better.

use caveripper::assets::AssetManager;
use caveripper::caveinfo::CaveInfo;
use caveripper::layout::{Layout, SpawnObject};
use caveripper::sublevel::Sublevel;

#[test]
fn test_ranking() {
    //println!("Working dir: {}", std::fs::canonicalize(".").unwrap().display());

    //let sublevels = AssetManager::all_sublevels().unwrap();
    AssetManager::init_global("../assets", "..").unwrap();

    let sh2 = Sublevel::try_from("SH-2").unwrap();
    let caveinfo = AssetManager::get_caveinfo(&sh2).unwrap();

    let mut ranked = Vec::new();
    for i in 0..10_000 {
        let path = estimate_path_lengths(caveinfo, i);
        //println!("Path lenght for seed {i} is {path}");
        ranked.push((path as i32, i));
    }

    ranked.sort();
    println!("Best seed is {:X} with score {}.", ranked[0].1, ranked[0].0);
    println!("Worst seed is {:X} with score {}.", ranked.last().unwrap().1, ranked.last().unwrap().0);

}

fn estimate_path_lengths(sublevel: &CaveInfo, seed: u32) -> f32 {
    let layout = Layout::generate(seed, sublevel);
    let mut relevant_object_coords = Vec::new();

    // Iterate over all the relevant objects in a level.
    for unit in layout.map_units {
        for spawn_point in unit.spawnpoints {
            let coordinates = (spawn_point.x, spawn_point.z);
            for spawn_object in spawn_point.contains {
                use SpawnObject::*;
                match spawn_object {
                    // Treasure, ship and hole
                    Item(_) | Ship | Hole(_) => {
                        relevant_object_coords.push(coordinates)
                    }
                    // Enemy with treasure
                    Teki(teki_info, _offset) => {
                        if teki_info.carrying.is_some() {
                            // TODO also apply the offset.
                            relevant_object_coords.push(coordinates)
                        }
                    }
                    // For simplicity, I'm ignoring geysers.
                    // But they do make in difference in SH-7
                    Geyser(_) => {}

                    _ => {}
                }
            }
        }
    }
    // At least the ship and a hole should be there.
    // In the sublevels with only a geyser there should always be a treasure.
    assert!(relevant_object_coords.len() >= 2);

    // Calculate the rectangle that encloses all the objects.
    let obj = relevant_object_coords.pop().unwrap();
    let mut min_x = obj.0;
    let mut min_z = obj.1;
    let mut max_x = obj.0;
    let mut max_z = obj.1;

    for (x, z) in relevant_object_coords {
        min_x = min(min_x, x);
        min_z = min(min_z, z);
        max_x = max(max_x, x);
        max_z = max(max_z, z);
    }

    let width = f32::abs(max_x - min_x);
    let height = f32::abs(max_z - min_z);
    return width + height;
}

/// Can't use `std::cmp::min` because f32 isn't `Ord`.
fn min(first: f32, second: f32) -> f32 {
    if first < second { first } else { second }
}

/// Can't use `std::cmp::max` because f32 isn't `Ord`.
fn max(first: f32, second: f32) -> f32 {
    if first > second { first } else { second }
}


