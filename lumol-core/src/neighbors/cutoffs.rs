// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

use crate::{UnitCell, ParticleVec, Vector3D};

/// The Cutoffs object knows how far a particle needs to travel before the neighborlist needs to be updated
#[derive(Clone)]
pub struct Cutoffs {
    // The minimal distance between two partucles, that are not part of the neighbor list
    pub max_cutoff: f64,
    // The maximal distance that a particle can move without forcing a neighbor update
    pub skin: f64,   
}

impl Cutoffs {

    /// Construct a new Cutoff
    pub fn new(
        max_cutoff: f64, 
        skin: f64, 
    ) -> Self { 
        Self { 
            max_cutoff, 
            skin, 
        } 
    }

    
    /// The the squared distance between two particles is smaller than max_cutoff, then the
    /// pair must the present in the neighborlist.
    pub fn max_cutoff2(&self) -> f64 {
        self.max_cutoff.powi(2)
    }

    /// If the squared distance travelled by any particle is bigger than skin2, then it is time 
    /// to update the neigborlist
    pub fn skin2(&self) -> f64 {
        self.skin.powi(2)
    }


    /// The the squared distance between two particles is smaller than update_cutoff2, then the
    /// pair should be added to the neighborlist during an update
    pub fn update_cutoff2(&self) -> f64 {
        (self.max_cutoff+2.0*self.skin).powi(2)
    }

    /// Returns true if any particle has moved long enough to warrant a neighborlist update
    pub fn needs_update ( 
        &self,
        positions_at_last_update: &[Vector3D],
        cell: &UnitCell,
        particles: &ParticleVec
    ) -> bool {

        let skin2= self.skin2();
        assert_eq!(positions_at_last_update.len(), particles.position.len()); 
        for (xi_nblist, xi) in positions_at_last_update.iter().zip(particles.position.iter()) {
        
            // Return true if the particle has moved far enough to warrant an update
            let r2 = cell.distance2(xi_nblist, xi);
            if r2 > skin2 {  
                return true 
            }        
        };
        false
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cutoff() {
        let cutoff = Cutoffs::new(3.0, 0.5);
        assert_eq!(cutoff.max_cutoff2(), 9.0);
        assert_eq!(cutoff.skin2(), 0.25);
        assert_eq!(cutoff.update_cutoff2(), 16.0);
    }
}