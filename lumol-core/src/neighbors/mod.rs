// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! Neighbor objects
//! 
//! # Neighbors
//!
//! An object that knows which particles in the system are close to 
//! eachother. This information is used to calculate forces for MD-simulations.
use crate::{UnitCell, ParticleVec};

mod countdown;
use self::countdown::CountDown;

mod cutoffs;
use self::cutoffs::Cutoffs;

mod all_pairs;
pub use self::all_pairs::AllPairs;

mod directed_linked_list;
pub use self::directed_linked_list::DirectedLinkedList;

mod statistics;
use self::statistics::Statistics;

/// An enum with structs that implement the Neighbors trait
/// 
/// Note: Ideally this this enum should be replaced by a trait object such as Box<Neighbors>, 
/// but it couldn't figure out how to do this. The problem is that each_i and each_j have 
/// generic type parameters (See rustc(E0038)).
/// 
/// Alternatively the crate named 'ambassador' can be used to implement the 
/// Neighbors trait fo NeighborlistKind
#[derive(Clone)]
pub enum Neighbors {
    /// Iterate over all pairs of two particles when calculating forces.
    /// This corresponds to not having a neighborlist
    AllPairs(AllPairs),
    /// Directed Linked list (Useful for MD)
    Directed(Box<DirectedLinkedList>)
}

impl Neighbors {

    /// Construct a new neighborlists
    pub fn new_all_pairs() -> Neighbors {
        Neighbors::AllPairs(AllPairs::new())
    }

    /// Construct a new directed neighborlists
    pub fn new_directed_linkedlist(
        // The maximal cutoff for the pair potential
        max_cutoff: f64,
        // The maximal distance that a particle can move before
        // The neighborlist needs to be updated
        skin: f64,
        // Minimal number of steps from a neighborlist update to the first
        // neighborlist update check
        delay: u64,
        // Number of steps between every neighborlist update check
        steps_per_update_check: u64,
        // Number of neighborlist updates between each neighborlist sanity check.
        // If the value is None, then sanity checks are not performed.
        // Note that these sanity checks are not neccesary for the algorithm to work.
        updates_per_sanity_check: Option<u64>,
    ) -> Neighbors {
        let countdown = CountDown::new(delay, steps_per_update_check, updates_per_sanity_check);
        let cutoffs = Cutoffs::new(max_cutoff, skin);    
        Neighbors::Directed(Box::new(DirectedLinkedList::new(countdown, cutoffs)))
    }

    /// Investigate if the neighborlist needs to be updated and update if neccesary
    pub fn ensure_updated(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.ensure_updated(cell, particles),
            Neighbors::Directed(neighbors) => neighbors.ensure_updated(cell, particles),
        }
    }

    /// Force the neighborlist to be updated
    pub fn update_neighbors(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.update_neighbors(cell, particles),
            Neighbors::Directed(neighbors) => neighbors.update_neighbors(cell, particles),
        }
    }

    /// Print statistics regarding neighborlist updates
    pub fn print_statistics(&self) {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.print_statistics(),
            Neighbors::Directed(neighbors) => neighbors.print_statistics(),
        }
    }

    /// Iterate over nodes that are the starting point of at least one edge
    #[inline]
    pub fn each_i<OP> (&self, op: OP) where OP: Fn(usize) -> () + Sync + Send {
        match self {
            Neighbors::AllPairs(neighbors) => neighbors.each_i(op),
            Neighbors::Directed(neighbors) => neighbors.each_i(op),
        }
    }

    /// Iterate over the endpoints of edges that start at i
    #[inline]
    pub fn each_j<OP> (&self, i: usize, op: OP) where  OP: FnMut(usize) -> () {
        match self {
            Neighbors::AllPairs(nlist) => nlist.each_j(i, op),
            Neighbors::Directed(nlist) => nlist.each_j(i, op),
        }
    }
}
