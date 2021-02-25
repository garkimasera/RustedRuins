#![cfg_attr(not(global_state_obj), allow(dead_code))]
#![cfg_attr(not(global_state_obj), allow(unused_variables))]
#![cfg_attr(not(global_state_obj), allow(unused_imports))]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate rusted_ruins_geom as geom;
extern crate rusted_ruins_rng as rng;

mod utils;

pub mod basic;
pub mod hashmap;
pub mod obj;
#[macro_use]
pub mod idx_conv;
pub mod gamedata;
#[cfg(feature = "global_state_obj")]
pub mod gobj;
pub mod impl_filebox;
pub mod maptemplate;
pub mod objholder;
pub mod pakutil;
pub mod piece_pattern;
pub mod regiongen;
pub mod saveload;
pub mod sitegen;
