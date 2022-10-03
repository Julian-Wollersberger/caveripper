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
#![allow(unused)]

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

pub fn multiple_sublevels() {
    let pod_sublevels = [
        // Some sublevels don't make sense to include.
        // Always the same in this heuristic:
        // EC1, EC2, HoB5, WFG3, WFG5, SH1, BK7, GK6
        //
        // Almost the same:
        // WFG1, BK2, CoS5
        //
        // Heuristic doesn't work:
        // HoB2, GK2, SCx8, FC2, SH6, SCx4
        //
        // Skipped:
        // BK3, BK5, CoS1, CoS4, GK1, SCx5, SCx9, FC6

        "HoB1",
        "HoB3",
        "HoB4",
        "WFG2",
        "WFG4",
        "SH2",
        "SH3",
        "SH4",
        "SH5", // SH6 needs a different heuristic, sadly.
        "SH7",
        "BK1",
        "BK4",
        "BK6",
        "SCx1",
        "SCx2",
        "SCx3",
        "SCx6",
        "SCx7",
        "FC1",
        "FC3",
        "FC4",
        "FC5",
        "FC7", // Kinda wrong because we take a geyser
        "CoS2",
        "CoS3",
        "GK3",
        "GK4",
        "GK5",
    ];

    AssetManager::init_global("assets", ".").unwrap();
    let caveinfos = pod_sublevels.map(
        |level| AssetManager::get_caveinfo(
            &Sublevel::try_from(level).unwrap()).unwrap());

    let mut ranked = Vec::new();
    for i in 0..100_000 {
        let mut sum = 0.0;
        for level in caveinfos {
            sum += estimate_path_lengths(level, i);
        }
        //println!("Sum for seed {i} is {sum}");
        ranked.push((sum as i32, i));
    }

    ranked.sort();
    println!("Best seed is {:X} with score {}.", ranked[0].1, ranked[0].0);
    println!("Worst seed is {:X} with score {}.", ranked.last().unwrap().1, ranked.last().unwrap().0);

    // (Forgot FC and SCx, includes holes)
    // Best seed is 64C0 with score 32635.
    // Worst seed is F5D with score 46315.

    // (Holes excluded, with FC & SCx)
    // Best seed is 13276 with score 40685.
    // Worst seed is 12111 with score 59245.
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
                    Item(_) | Ship => {
                        relevant_object_coords.push(coordinates)
                    }
                    // Enemy with treasure
                    Teki(teki_info, _offset) => {
                        if teki_info.carrying.is_some() {
                            // TODO also apply the offset.
                            relevant_object_coords.push(coordinates)
                        }
                    }

                    // Although a far away hole is a problem, usually you have
                    // enough time to get to the hole, especially in bad layouts.
                    // But including the hole muddles up the statistics.
                    Hole(_) => { }

                    // For simplicity, I'm ignoring geysers.
                    // But they do make in difference in SH-7
                    Geyser(_) => { }

                    _ => {}
                }
            }
        }
    }
    // Distance doesn't make sense with only one object.
    assert!(relevant_object_coords.len() >= 2, "Not enough objects on {}", sublevel.sublevel.short_name());

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


