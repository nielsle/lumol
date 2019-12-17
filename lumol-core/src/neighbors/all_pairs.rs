// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{ParticleVec, UnitCell};
 /// A neighbors object that iterates over all pairs of two particles when 
 /// calculating forces. This corresponds to not having a neighborlist
#[derive(Clone)]
pub struct AllPairs {
    /// The number of atoms in the system
    natoms: usize,
    /// This field is false if the neighbors object needs to be initialized
    initialized: bool,
}

impl AllPairs {
    /// Construct an AllPairs
    pub fn new() -> AllPairs {
        AllPairs { 
            natoms: 0 ,
            initialized: false,
        }
    }

    /// Investigate if the neighborlist needs to be updated and update if neccesary
    pub fn ensure_updated(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {
        self.update_neighbors(cell, particles)
    }

    /// Force the neighborlist to be updated
    pub fn update_neighbors(&mut self, _: &UnitCell, particles: &mut ParticleVec) {
        self.natoms = particles.len();
        self.initialized = true;
    }

    /// Print statistics regarding neighborlist updates
    pub fn print_statistics(&self) {}

    /// Iterate over nodes that are the starting point of at least one edge
    #[inline]
    pub fn each_i<OP>(&self, op: OP)
    where
        OP: Fn(usize) -> () + Sync + Send,
    {
        assert!(self.initialized, "The neighbors object wastn't initialized. use ensure_updated");
        (0..self.natoms).into_par_iter().for_each(op)
    }

    /// Iterate over the endpoints of edges that start at i
    #[inline]
    pub fn each_j<OP>(&self, i: usize, mut op: OP)
    where
        OP: FnMut(usize) -> (),
    {
        for j in 0..i {
            op(j)
        }
    }
}
