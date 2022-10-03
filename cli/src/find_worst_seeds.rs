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

use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use caveripper::assets::AssetManager;
use caveripper::caveinfo::CaveInfo;
use caveripper::layout::{Layout, PlacedDoor, SpawnObject};
use caveripper::sublevel::Sublevel;

#[test]
fn test_ranking_individual_level() {
    let sublevel = "SH-2";

    //println!("Working dir: {}", std::fs::canonicalize(".").unwrap().display());
    AssetManager::init_global("../assets", "..").unwrap();
    let sub = Sublevel::try_from(sublevel).unwrap();
    let caveinfo = AssetManager::get_caveinfo(&sub).unwrap();

    let mut ranked = Vec::new();
    for i in 0..100_000 {
        let path = estimate_path_lengths(caveinfo, i);
        //println!("Path lenght for seed {i} is {path}");
        ranked.push((path as i32, i));
    }

    ranked.sort();
    println!("Best seed is {:X} with score {}.", ranked[0].1, ranked[0].0);
    println!("Worst seed is {:X} with score {}.", ranked.last().unwrap().1, ranked.last().unwrap().0);

    // Best seed is 1188 with score 940.
    // Worst seed is 18448 with score 4945.
}

pub fn multiple_sublevels(start_seed: u32, end_seed: u32) {
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
    for seed in start_seed..end_seed {
        let mut sum = 0.0;
        for level in caveinfos {
            sum += estimate_path_lengths(level, seed);
        }
        //println!("Sum for seed {i} is {sum}");
        ranked.push((sum as i32, seed));

        if (seed - start_seed) % 10_000 == 0 {
            println!("Checked {} seeds", seed - start_seed);
        }
    }

    ranked.sort();
    println!("Best seed is {:X} with score {}.", ranked[0].1, ranked[0].0);
    println!("Worst seed is {:X} with score {}.", ranked.last().unwrap().1, ranked.last().unwrap().0);

    // Print several seeds. Allows for manual sorting later.
    println!("\nList of best seeds:");
    for i in 0..100 {
        println!("{:X} with score {}", ranked[i].1, ranked[i].0);
    }

    println!("\nList of worst seeds:");
    ranked.reverse();
    for i in 0..100 {
        println!("{:X} with score {}", ranked[i].1, ranked[i].0);
    }


    // (Forgot FC and SCx, includes holes)
    // Best seed is 64C0 with score 32635.
    // Worst seed is F5D with score 46315.

    // (Holes excluded, with FC & SCx)
    // Best seed is 13276 with score 40685.
    // Worst seed is 12111 with score 59245.

    // cargo run --release -- leg-day 0x00000000 0x00010000
    // Best seed is 3B18 with score 41010.
    // Worst seed is EA71 with score 59235

    // cargo run --release -- leg-day 0x00000000 0x00100000
    // Best seed is 22A03 with score 39645.
    // Worst seed is 2895B with score 59795.

    // Now with hallway and gated-hole detection
    // Best seed is 102198 with score 52580.
    // Worst seed is 1002F2 with score 72735.
}

fn estimate_path_lengths(sublevel: &CaveInfo, seed: u32) -> f32 {
    let layout = Layout::generate(seed, sublevel);
    let mut relevant_object_coords = Vec::new();

    // Many map units can mean that there is a meme hallway.
    // Constant factor chosen by eyeballing. 10 is to little, 100 is to much.
    let num_units_score = layout.map_units.len() as f32 * 30.0;
    // A gated hole can be very annoying.
    let mut gated_hole_score = 0.0;

    // Iterate over all the relevant objects in a level.
    for unit in &layout.map_units {
        let mut has_hole = false;
        let mut has_gate = false;
        for spawn_point in &unit.spawnpoints {
            let coordinates = (spawn_point.x, spawn_point.z);
            for spawn_object in &spawn_point.contains {
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
                    Hole(_) => has_hole = true,

                    // For simplicity, I'm ignoring geysers.
                    // But they do make in difference in SH-7
                    Geyser(_) => { }

                    _ => {}
                }
            }
        }

        // Find any gates in this unit.
        for door in &unit.doors {
            let placed_door: Ref<PlacedDoor> = RefCell::borrow(door);
            match placed_door.seam_spawnpoint.borrow() {
                Some(SpawnObject::Gate(_)) => {
                    has_gate = true
                }
                _ => { }
            }
        }

        // Check whether the hole has a gate in front. That can cost some time
        // if you weren't prepared for it, or have few Pikmin.
        // I guess that 200 is a bit high, but I want to bias results towards
        // more annoying sublevels :)
        if has_hole && has_gate {
            gated_hole_score += 200.0;
            //println!("Gate on hole in seed {seed} in level {}", sublevel.sublevel.short_name());
        }
    }
    // Distance doesn't make sense with only one object.
    assert!(relevant_object_coords.len() >= 2, "Not enough objects on {}", sublevel.sublevel.short_name());

    // Calculate the rectangle that encloses all the objects.
    // Start with one object.
    let obj = relevant_object_coords.pop().unwrap();
    let mut min_x = obj.0;
    let mut min_z = obj.1;
    let mut max_x = obj.0;
    let mut max_z = obj.1;

    // Update the rectangle "sides" if an object is outside of it.
    for (x, z) in relevant_object_coords {
        min_x = min(min_x, x);
        min_z = min(min_z, z);
        max_x = max(max_x, x);
        max_z = max(max_z, z);
    }

    let width = f32::abs(max_x - min_x);
    let height = f32::abs(max_z - min_z);
    return width + height + num_units_score + gated_hole_score;
}

/// Can't use `std::cmp::min` because f32 isn't `Ord`.
fn min(first: f32, second: f32) -> f32 {
    if first < second { first } else { second }
}

/// Can't use `std::cmp::max` because f32 isn't `Ord`.
fn max(first: f32, second: f32) -> f32 {
    if first > second { first } else { second }
}


