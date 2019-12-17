use std::fmt;

///Statistics about neighborlist updates
#[derive(Clone, Default)]
pub struct Statistics {
    /// Total number of steps
    pub steps: u64,
    /// Number of times that the system has checked if a particle has moved far enough
    /// to warrant an update og the neighborlist
    pub update_checks: u64,
    /// Number of times that the neighborlist was updated
    pub updates: u64,
    /// Number sanity checks.
    pub sanity_checks: u64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let steps_per_update_check = (self.steps as f64)/(self.update_checks as f64);
        let steps_per_update = (self.steps as f64)/(self.updates as f64);
        let steps_per_sanity_check = (self.steps as f64)/(self.sanity_checks as f64);
        let update_checks_per_update = (self.update_checks as f64)/(self.updates as f64);
   
        writeln!(f, "Neighborlist statistics:")?;
        writeln!(f, "Steps                          {:10}", self.steps)?;
        writeln!(f, "Update checks                  {:10}", self.update_checks)?;
        writeln!(f, "Updates                        {:10}", self.updates)?;
        writeln!(f, "Sanity checks                  {:10}", self.sanity_checks)?;
        
        writeln!(f, "Steps per update check         {:10.2}", steps_per_update_check)?;
        writeln!(f, "Steps per update               {:10.2}", steps_per_update)?;
        writeln!(f, "Steps per sanity_check         {:10.2}", steps_per_sanity_check)?;
        writeln!(f, "Update checks per update       {:10.2}", update_checks_per_update)
    }
}