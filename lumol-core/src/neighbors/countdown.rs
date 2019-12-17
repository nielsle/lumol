// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license
use crate::neighbors::Statistics;

/// Determines when to perform neighborlist checks. (It is too expensive to do it after each step)
/// Also gathers statistics about neighborlist updates
#[derive(Clone)]
pub struct CountDown {
    /// Number of steps before first neighborlist checks after a neighborlist update
    delay: u64,
    /// Minimal number of steps between each update attempt
    steps_per_update_check: u64,
    /// Minimal number of updates between each sanity check of the neighborlist
    /// (Set this field ti None to disable expensive sanity check)
    updates_per_sanity_check: Option<u64>,

    
    /// Counts the number of MD-steps
    /// This number is reset after each neighborlist update
    step_counter: u64,
    /// Counts the number of updates
    /// This number is reset after each sanity check
    update_counter: u64,
    /// statistics in neighbor updates
    statistics: Statistics
}

impl CountDown {
   
    /// Create a new countdown object
    pub fn new(
        delay: u64, 
        steps_per_update_check: u64, 
        updates_per_sanity_check: Option<u64>
    ) -> Self { 
        Self { 
            delay,
            steps_per_update_check, 
            updates_per_sanity_check,
            step_counter: 0,
            update_counter: 0,
            statistics: Statistics::default()
        } 
    }

    /// Return true if it is time to perform a neighborlist check
    pub fn needs_update_check (&mut self) -> bool {

        // Increase counters
        self.step_counter +=1;
        self.statistics.steps += 1;

        // Return false if last update was within "delay" steps
        let steps_after_delay= match self.step_counter.checked_sub(self.delay + 1) {
            Some(t) => t,
            None => return false
        };

        // Return false if "steps_after_delay" isn't divisible with "steps_per_update_check"
        if (steps_after_delay % self.steps_per_update_check) == 0 {
            self.statistics.update_checks += 1;
            return true
        };
        
        false
     }

    /// Return true if it is time to perform an expensive sanitycheck
    pub fn needs_sanity_check (&mut self) -> bool {

        // Increase counters
        self.step_counter = 0;
        self.update_counter +=1;
        self.statistics.updates += 1; 

        // Return false if "update_counter" is divisible with "updates_per_sanity_check"
        if let Some(updates_per_sanity_check) = self.updates_per_sanity_check {
            if self.update_counter % updates_per_sanity_check == 0 {
                self.statistics.sanity_checks += 1;
                self.update_counter = 0;
                return true
            };
        };

        false
    }

    /// Returns a reference to the statistics object
    pub fn statistics(&self) -> &Statistics{
        &self.statistics
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown() {
        let mut countdown = CountDown::new(5,2,None);

        let expected = [
            // Return false 5 times
            false, false, false, false, false,
            // Return true every second time due
            true, false, true, false, true, false, true, false
        ];
    
        for needs_update in &expected {
            assert_eq!(countdown.needs_update_check(), *needs_update)
        } 

        assert_eq!(countdown.needs_sanity_check(), false);

        for needs_update in &expected {
            assert_eq!(countdown.needs_update_check(), *needs_update)
        } 
    }
}
