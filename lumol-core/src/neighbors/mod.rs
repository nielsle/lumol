// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! Neighbor objects
//! 
//! # Neighbors
//!
//! An object that knows which particles in the system are close to 
//! eachother. This information is used to calculate forces for MD-simulations.
use crate::{UnitCell, ParticleVec};

mod all_pairs;
pub use self::all_pairs::AllPairs;

/// Keeps track of which particles are neighbors to which
/// 
/// Note: Ideally this this enum should be replaced by a trait object such as Box<Neighbors>, 
/// but it couldn't figure out how to do this. The problem is that each_i and each_j have 
/// generic type parameters (See rustc(E0038)).
/// 
/// Alternatively the crate named 'ambassador' can be used to implement the 
/// a trait for each variant of Neighbor
#[derive(Clone)]
pub enum Neighbors {
    /// Iterate over all pairs of two particles when calculating forces.
    /// This corresponds to not having a neighborlist
    AllPairs(AllPairs),
}

impl Neighbors {

    /// Construct a new neighborlists
    pub fn new_all_pairs() -> Neighbors {
        Neighbors::AllPairs(AllPairs::new())
    }

    /// Investigate if the neighborlist needs to be updated and update if neccesary
    pub fn ensure_updated(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.ensure_updated(cell, particles),
        }
    }

    /// Force the neighborlist to be updated
    pub fn update_neighbors(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.update_neighbors(cell, particles),
        }
    }

    /// Iterate over nodes that are the starting point of at least one edge
    #[inline]
    pub fn each_i<OP> (&self, op: OP) where OP: Fn(usize) -> () + Sync + Send {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.each_i(op),
        }
    }

    /// Iterate over the endpoints of edges that start at i
    #[inline]
    pub fn each_j<OP> (&self, i: usize, op: OP) where  OP: FnMut(usize) -> () {
        match self {
            Neighbors::AllPairs(nlist) => nlist.each_j(i, op),
        }
    }
}