// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

use rayon::prelude::{IntoParallelIterator,ParallelIterator};
use crate::{ParticleVec, UnitCell, Vector3D};
use crate::neighbors::{Cutoffs, CountDown};

/// A Directed Verlet neighborlist represents neighborships as a directed graph. 
#[derive(Clone)]
pub struct DirectedLinkedList {
    /// The countdown determines if it ius time to update the neighborlist
    countdown: CountDown,
    /// The cutoffs determines id the configuration needs to be updated
    cutoffs: Cutoffs,
    /// This field is false if the neighbors ubject was never initialized
    initialized: bool,
    /// Number of steps between every update attempt
    edges: Vec<Vec<usize>>,
    /// Snapshot of particle positions, when the neighborlist was last updated
    position_snapshot: Vec<Vector3D>
}

impl DirectedLinkedList {

    /// Construct a new DirectedLinkedList
    pub fn new (
        countdown: CountDown,
        cutoffs: Cutoffs,
    ) ->  DirectedLinkedList {
        DirectedLinkedList {
            countdown,
            cutoffs,
            initialized: false,
            edges: Vec::new(),
            position_snapshot: Vec::new() 
        }
    }

    /// Perform an expensive sanity check of the neighborlist
    /// Warning: this function panic if the neighborlist is invalid
    pub fn sanity_check(&mut self, cell: &UnitCell, particles: &mut ParticleVec) {

        let max_cutoff2 = self.cutoffs.max_cutoff2();

        for i in 0..particles.len() {
            let xi = particles.position[i];
            for j in 0..i {
                let xj = particles.position[j];
                let r2= cell.distance2(&xi, &xj);
                if r2 < max_cutoff2 && !self.edges[i].iter().any(|v: &usize| *v == j) {
                    println!();
                    println!("i {} xi {:?}", i, xi);
                    println!("j {} xj {:?}", j, xj);
                    println!("r2 {:.2} max {:.2}", r2, max_cutoff2);
                    panic!("Invalid neighborlist")
                }
            }
        }
    }

    /// Investigate if the neighborlist needs to be updated and update if neccesary
    pub fn ensure_updated(
        &mut self, 
        cell: &UnitCell,
        particles: &mut ParticleVec
    ) {

        // Determine if it is time to check the neighborlist
        if self.countdown.needs_update_check() {

            // Determine if a particle as moved too far enough to warrant an update
            if self.cutoffs.needs_update(&self.position_snapshot, cell, particles) {
          
                // Perform expensive sanity check once in a while
                if self.countdown.needs_sanity_check() {
                    self.sanity_check(cell, particles);
                }

                // Update of the neighborlist
                self.update_neighbors(cell, particles);        
            }
        }         
    }

    /// Force the neighborlist to be updated
    pub fn update_neighbors(
        &mut self,
        cell: &UnitCell,
        particles: &mut ParticleVec
    ) {

        self.edges =  Vec::new();
        let update_cutoff2 = self.cutoffs.update_cutoff2();

        for i in 0..particles.len() {
            let xi = particles.position[i];
            let mut ni = Vec::new();
            for j in 0..i {
                let xj = particles.position[j];
                if  cell.distance2(&xi, &xj) < update_cutoff2 {
                    ni.push(j);
                }
            }
            
            self.edges.push(ni)
        } 
        
        // Copy particle positions to position_snapshot
        self.position_snapshot = particles.position.to_vec();

        self.initialized = true;

    }

    /// Print statistics regarding neighborlist updates
    pub fn print_statistics(&self) {
        println!("{}", self.countdown.statistics())
    }

    /// Iterate over nodes that are the starting point of at least one edge
    #[inline]
    pub fn each_i<OP> (&self, op: OP) where OP: Fn(usize) -> () + Sync + Send { 
        assert!(self.initialized, "The neighbors object wastn't initialized. use ensure_updated");
        (0..self.edges.len())
            .into_par_iter()
            .for_each(op)
    } 
    
    /// Iterate over the endpoints of edges that start at i
    #[inline]
    pub fn each_j<OP> (&self, i: usize, mut op: OP) where  OP: FnMut(usize) -> () {
        for j in self.edges.get(i).unwrap() {
            op(*j)
        }
    }
}
