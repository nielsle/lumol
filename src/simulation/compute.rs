// Cymbalum, an extensible molecular simulation engine
// Copyright (C) 2015-2016 G. Fraux — BSD license

//! Computing properties of a system
use constants::K_BOLTZMANN;
use types::{Matrix3, Vector3D, Zero};
use system::System;

/// The compute trait allow to compute properties of a system, whithout
/// modifying this system. The Output type is the type of the computed
/// property.
pub trait Compute {
    /// The data type of the property
    type Output;
    /// Compute the property
    fn compute(&self, system: &System) -> Self::Output;
}

/******************************************************************************/
/// Compute all the forces acting on the system, and return a vector of
/// force acting on each particles
pub struct Forces;
impl Compute for Forces {
    type Output = Vec<Vector3D>;
    fn compute(&self, system: &System) -> Vec<Vector3D> {
        let natoms = system.size();
        let mut res = vec![Vector3D::new(0.0, 0.0, 0.0); natoms];

        for i in 0..system.size() {
            for j in (i+1)..system.size() {
                let d = system.wraped_vector(i, j);
                let dn = d.normalized();
                let r = d.norm();
                for &(ref potential, ref restriction) in system.pair_potentials(i, j) {
                    if !restriction.is_excluded_pair(system, i, j) {
                        let s = restriction.scaling(system, i, j);
                        let f = s * potential.force(r);
                        res[i] = res[i] + f * dn;
                        res[j] = res[j] - f * dn;
                    }
                }
            }
        }

        for molecule in system.molecules() {
            for bond in molecule.bonds() {
                let (i, j) = (bond.i(), bond.j());
                let d = system.wraped_vector(i, j);
                let dn = d.normalized();
                let r = d.norm();
                for potential in system.bond_potentials(i, j) {
                    let f = potential.force(r);
                    res[i] = res[i] + f * dn;
                    res[j] = res[j] - f * dn;
                }
            }

            for angle in molecule.angles() {
                let (i, j, k) = (angle.i(), angle.j(), angle.k());
                let (theta, d1, d2, d3) = system.angle_and_derivatives(i, j, k);
                for potential in system.angle_potentials(i, j, k) {
                    let f = potential.force(theta);
                    res[i] = res[i] + f * d1;
                    res[j] = res[j] + f * d2;
                    res[k] = res[k] + f * d3;
                }
            }

            for dihedral in molecule.dihedrals() {
                let (i, j, k, m) = (dihedral.i(), dihedral.j(), dihedral.k(), dihedral.m());
                let (phi, d1, d2, d3, d4) = system.dihedral_and_derivatives(i, j, k, m);
                for potential in system.dihedral_potentials(i, j, k, m) {
                    let f = potential.force(phi);
                    res[i] = res[i] + f * d1;
                    res[j] = res[j] + f * d2;
                    res[k] = res[k] + f * d3;
                    res[m] = res[m] + f * d4;
                }
            }
        }

        if let Some(coulomb) = system.coulomb_potential() {
            let forces = coulomb.forces(&system);
            debug_assert!(forces.len() == natoms, "Wrong `forces` size in coulomb potentials");
            for (i, force) in forces.iter().enumerate() {
                res[i] = res[i] + (*force);
            }
        }

        for global in system.global_potentials() {
            let forces = global.forces(&system);
            debug_assert!(forces.len() == natoms, "Wrong `forces` size in global potentials");
            for (i, force) in forces.iter().enumerate() {
                res[i] = res[i] + (*force);
            }
        }
        return res;
    }
}

/******************************************************************************/
/// Compute the potential energy of the system
pub struct PotentialEnergy;
impl Compute for PotentialEnergy {
    type Output = f64;
    fn compute(&self, system: &System) -> f64 {
        let mut energy = 0.0;
        let evaluator = system.energy_evaluator();

        energy += evaluator.pairs();
        energy += evaluator.molecules();
        energy += evaluator.coulomb();
        energy += evaluator.global();

        assert!(energy.is_finite(), "Potential energy is infinite!");
        return energy;
    }
}

/******************************************************************************/
/// Compute the kinetic energy of the system
pub struct KineticEnergy;
impl Compute for KineticEnergy {
    type Output = f64;
    fn compute(&self, system: &System) -> f64 {
        let mut energy = 0.0;
        for particle in system {
            energy += 0.5 * particle.mass * particle.velocity.norm2();
        }
        assert!(energy.is_finite(), "Kinetic energy is infinite!");
        return energy;
    }
}

/******************************************************************************/
/// Compute the total energy of the system
pub struct TotalEnergy;
impl Compute for TotalEnergy {
    type Output = f64;
    fn compute(&self, system: &System) -> f64 {
        let kinetic = KineticEnergy.compute(system);
        let potential = PotentialEnergy.compute(system);
        return kinetic + potential;
    }
}

/******************************************************************************/
/// Compute the instananeous temperature of the system
pub struct Temperature;
impl Compute for Temperature {
    type Output = f64;
    fn compute(&self, system: &System) -> f64 {
        let kinetic = KineticEnergy.compute(system);
        let natoms = system.size() as f64;
        return 1.0/K_BOLTZMANN * 2.0 * kinetic/(3.0 * natoms);
    }
}

/******************************************************************************/
/// Compute the volume of the system
pub struct Volume;
impl Compute for Volume {
    type Output = f64;
    #[inline]
    fn compute(&self, system: &System) -> f64 {
        return system.cell().volume();
    }
}

/******************************************************************************/
/// Compute the virial tensor of the system
pub struct Virial;
impl Compute for Virial {
    type Output = Matrix3;
    fn compute(&self, system: &System) -> Matrix3 {
        let mut res = Matrix3::zero();
        for i in 0..system.size() {
            for j in (i+1)..system.size() {
                for &(ref potential, ref restriction) in system.pair_potentials(i, j) {
                    if !restriction.is_excluded_pair(system, i, j) {
                        let s = restriction.scaling(system, i, j);
                        let d = system.wraped_vector(i, j);
                        res = res + 2.0 * s * potential.virial(&d);
                    }
                }
            }
        }

        // FIXME: implement virial computations for molecular potentials
        // (angles & dihedrals)

        if let Some(coulomb) = system.coulomb_potential() {
            res = res + coulomb.virial(&system);
        }

        for global in system.global_potentials() {
            res = res + global.virial(&system);
        }

        return res;
    }
}

/******************************************************************************/
/// Compute the stress tensor of the system
pub struct Stress;
impl Compute for Stress {
    type Output = Matrix3;
    fn compute(&self, system: &System) -> Matrix3 {
        let mut K = Matrix3::zero(); // Kinetic tensor
        for particle in system.iter() {
            let m = particle.mass;
            let vel = particle.velocity;
            K[(0, 0)] += m * vel.x * vel.x;
            K[(0, 1)] += m * vel.x * vel.y;
            K[(0, 2)] += m * vel.x * vel.z;

            K[(1, 0)] += m * vel.y * vel.x;
            K[(1, 1)] += m * vel.y * vel.y;
            K[(1, 2)] += m * vel.y * vel.z;

            K[(2, 0)] += m * vel.z * vel.x;
            K[(2, 1)] += m * vel.z * vel.y;
            K[(2, 2)] += m * vel.z * vel.z;
        }

        let W = Virial.compute(system);
        let V = Volume.compute(system);
        return 1.0 / V * (K - W);
    }
}

/******************************************************************************/
/// Compute the virial pressure of the system
pub struct Pressure;
impl Compute for Pressure {
    type Output = f64;
    fn compute(&self, system: &System) -> f64 {
        let W = Virial.compute(system);
        let virial = W[(0, 0)] + W[(1, 1)] + W[(2, 2)];
        let V = Volume.compute(system);
        let natoms = system.size() as f64;
        let T = Temperature.compute(system);
        return natoms * K_BOLTZMANN * T / V - virial / (3.0 * V);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use types::*;
    use system::*;
    use potentials::Harmonic;
    use units;

    const EPS : f64 = 1e-8;

    fn testing_system() -> System {
        let mut system = System::from_cell(UnitCell::cubic(10.0));;

        system.add_particle(Particle::new("F"));
        system[0].position = Vector3D::new(0.0, 0.0, 0.0);

        system.add_particle(Particle::new("F"));
        system[1].position = Vector3D::new(1.3, 0.0, 0.0);

        let mut velocities = BoltzmanVelocities::new(units::from(300.0, "K").unwrap());
        velocities.init(&mut system);

        system.add_pair_interaction("F", "F",
            Box::new(Harmonic{
                k: units::from(300.0, "kJ/mol/A^2").unwrap(),
                x0: units::from(1.2, "A").unwrap()
            })
        );
        return system;
    }

    #[test]
    fn forces() {
        let system = &testing_system();
        let res = Forces.compute(system);

        let forces_tot = res[0] + res[1];
        assert_eq!(forces_tot, Vector3D::new(0.0, 0.0, 0.0));

        assert_approx_eq!(res[0].x, 3e-3, EPS);
        assert_approx_eq!(res[0].y, 0.0, EPS);
        assert_approx_eq!(res[0].y, 0.0, EPS);

        assert_approx_eq!(res[1].x, -3e-3, EPS);
        assert_approx_eq!(res[1].y, 0.0, EPS);
        assert_approx_eq!(res[1].y, 0.0, EPS);
    }

    #[test]
    fn force_molecular() {
        let mut system = testing_system();
        system.add_particle(Particle::new("F"));
        system.add_particle(Particle::new("F"));

        system[0].position = Vector3D::new(0.0, 0.0, 0.0);
        system[1].position = Vector3D::new(1.2, 0.0, 0.0);
        system[2].position = Vector3D::new(1.2, 1.2, 0.0);
        system[3].position = Vector3D::new(2.4, 1.2, 0.0);

        system.add_bond(0, 1);
        system.add_bond(1, 2);
        system.add_bond(2, 3);

        system.add_bond_interaction("F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/A^2").unwrap(),
                x0: units::from(1.22, "A").unwrap()
        }));

        system.add_angle_interaction("F", "F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/deg^2").unwrap(),
                x0: units::from(80.0, "deg").unwrap()
        }));

        system.add_dihedral_interaction("F", "F", "F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/deg^2").unwrap(),
                x0: units::from(185.0, "deg").unwrap()
        }));

        let res = Forces.compute(&system);
        let forces_tot = res[0] + res[1] + res[2] + res[3];
        assert_approx_eq!(forces_tot.norm2(), 0.0, 1e-12);
    }

    #[test]
    fn energy() {
        let system = &testing_system();
        let kinetic = KineticEnergy.compute(system);
        let potential = PotentialEnergy.compute(system);
        let total = TotalEnergy.compute(system);

        assert_eq!(kinetic + potential, total);
        assert_eq!(kinetic, 0.0007483016557453699);
        assert_approx_eq!(potential, 1.5e-4, EPS);

        assert_eq!(kinetic, system.kinetic_energy());
        assert_eq!(potential, system.potential_energy());
        assert_eq!(total, system.total_energy());
    }

    #[test]
    fn energy_molecular() {
        let mut system = testing_system();
        system.add_particle(Particle::new("F"));
        system.add_particle(Particle::new("F"));

        system[0].position = Vector3D::new(0.0, 0.0, 0.0);
        system[1].position = Vector3D::new(1.2, 0.0, 0.0);
        system[2].position = Vector3D::new(1.2, 1.2, 0.0);
        system[3].position = Vector3D::new(2.4, 1.2, 0.0);

        system.add_bond(0, 1);
        system.add_bond(1, 2);
        system.add_bond(2, 3);

        system.add_bond_interaction("F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/A^2").unwrap(),
                x0: units::from(1.22, "A").unwrap()
        }));

        system.add_angle_interaction("F", "F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/deg^2").unwrap(),
                x0: units::from(80.0, "deg").unwrap()
        }));

        system.add_dihedral_interaction("F", "F", "F", "F",
            Box::new(Harmonic{
                k: units::from(100.0, "kJ/mol/deg^2").unwrap(),
                x0: units::from(185.0, "deg").unwrap()
        }));

        assert_approx_eq!(PotentialEnergy.compute(&system), 0.040419916002, 1e-12);
    }

    #[test]
    fn temperature() {
        let system = &testing_system();
        let T = Temperature.compute(system);
        assert_approx_eq!(T, 300.0, 1e-9);
        assert_eq!(T, system.temperature());
    }

    #[test]
    fn volume() {
        let system = &testing_system();
        let V = Volume.compute(system);
        assert_eq!(V, 1000.0);
        assert_eq!(V, system.volume());
    }

    #[test]
    fn virial() {
        let system = &testing_system();
        let virial = Virial.compute(system);

        let mut res = Matrix3::zero();
        res[(0, 0)] = 2.0 * 3e-3 * 1.3;

        for i in 0..3 {
            for j in 0..3 {
                assert_approx_eq!(virial[(i, j)], res[(i, j)], 1e-9);
            }
        }
        assert_eq!(virial, system.virial());
    }

    #[test]
    fn stress() {
        let system = &testing_system();
        let stress = Stress.compute(system);
        let P = Pressure.compute(system);

        let trace = (stress[(0, 0)] + stress[(1, 1)] + stress[(2, 2)]) / 3.0;
        assert_approx_eq!(trace, P, 1e-9);
        assert_eq!(stress, system.stress());
    }

    #[test]
    fn pressure() {
        let system = &testing_system();
        let P = Pressure.compute(system);
        assert_approx_eq!(P, units::from(-348.9011556223, "bar").unwrap(), 1e-9);
        assert_eq!(P, system.pressure());
    }
}
