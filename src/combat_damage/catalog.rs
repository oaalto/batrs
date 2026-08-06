mod generated {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/combat_damage_catalog.rs"));
}

pub use generated::*;
