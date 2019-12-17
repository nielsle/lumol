// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license
#![allow(clippy::cast_lossless)]

//! Benchmark code for examining the Size-scale of Molecular dynamics simulation 
//! of an Argon crystal melt.
//!
//! The code runs 50000 MD-steps for several system sizes
//! Each of these runs are performed with or without a neighborlist
use lumol::{Particle, Molecule, System, UnitCell, Vector3D};
use lumol::energy::{LennardJones, PairInteraction};
use lumol::neighbors::Neighbors;
use lumol::units;

use lumol::sim::{MolecularDynamics, Simulation};
use lumol::sim::{BoltzmannVelocities, InitVelocities};

use std::time::Instant;

fn run_benchmark (
    n: usize,
    neighbors: Neighbors
) -> Result<(), Box<dyn std::error::Error>> {

    let lattice_constant = 3.4;
    let mut system = System::with_cell(UnitCell::cubic(lattice_constant*(n as f64)));

    // Create a cubic crystal of Argon by hand.
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let position = Vector3D::new(
                    i as f64 * lattice_constant, 
                    j as f64 * lattice_constant, 
                    k as f64 * lattice_constant
                );
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
            }
        }
    }

    let lj = Box::new(LennardJones {
        sigma: units::from(3.4, "A")?,
        epsilon: units::from(1.0, "kJ/mol")?,
    });
    system.set_pair_potential(
        ("Ar", "Ar"),
        PairInteraction::new(lj, units::from(8.5, "A")?),
    );

    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);

    system.set_neighbors(neighbors);

    let md = MolecularDynamics::new(units::from(1.0, "fs")?);
    let mut simulation = Simulation::new(Box::new(md));

    // Run 1000 steps to equilibrate and then rescale velocities.
    simulation.run(&mut system, 1000);    
    velocities.init(&mut system);
    let e_initial = system.total_energy();
 
    let now = Instant::now();
    simulation.run(&mut system, 50000);
    let e_final = system.total_energy();

    println!("Elapsed time (ms)    {:20.0}", now.elapsed().as_millis());
    println!("e_initial            {:20.10}",e_initial);
    println!("e_final              {:20.10}",e_final);
    println!("");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Iterate over lattice sizes
    for &n in &[5, 6, 7, 9, 11] {

        println!("Running a test with natoms={} using AllPairs",  n*n*n);
        let neighbors = Neighbors::new_all_pairs();
        run_benchmark(n, neighbors)?;
        
        println!("Running a test with natoms={} using DirectedLinkedList",  n*n*n);
        let neighbors = Neighbors::new_directed_linkedlist(
                units::from(8.5, "A")?, 
                units::from(1.0, "A")?, 
                0, 
                2, 
                None
        );
        run_benchmark(n, neighbors)?;

    };

    Ok(())
}
