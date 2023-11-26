/// The `Tracker` struct is a simple collection of named performance data counters used for
/// collecting data points on the simulation. The data collected is used to conduct light
/// statistical analysis about the performance of an algorithm.
#[derive(Debug, PartialEq)]
pub struct Tracker {
    pub page_hits: usize,
    pub tlb_hits: usize,
    pub tlb_flushes: usize,
    pub attempted_memory_accesses: usize,
    pub correct_memory_accesses: usize,
}

impl Tracker {
    /// Create a new instance of the `Tracker` struct with all counters initialized to zero.
    pub fn new() -> Self {
        Self {
            page_hits: 0,
            tlb_hits: 0,
            tlb_flushes: 0,
            attempted_memory_accesses: 0,
            correct_memory_accesses: 0,
        }
    }
}

impl std::fmt::Display for Tracker {
    /// Display format specification for the `Tracker` struct implemented to simplify the process
    /// of outputting statistics to the terminal.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a standard library formatter instance. For most use cases,
    /// this is provided automatically as this method is not meant to be called directly.
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Stats Tracked
---------------------------------
page_hits:                {:08}
tlb_hits:                 {:08}
tlb_flushes:              {:08}
attempted_memory_acceses: {:08}
correct_memory_accesses:  {:08}


tlb hit ratio:            {:.06}
page hit ratio:           {:.06}
               ",
            self.page_hits,
            self.tlb_hits,
            self.tlb_flushes,
            self.attempted_memory_accesses,
            self.correct_memory_accesses,
            self.tlb_hits as f32 / self.attempted_memory_accesses as f32,
            self.page_hits as f32 / self.attempted_memory_accesses as f32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    mod tracker_tests {

        use super::*;

        #[test]
        fn new() {
            let tracker = Tracker::new();
            assert_eq!(tracker.page_hits, 0);
            assert_eq!(tracker.tlb_hits, 0);
            assert_eq!(tracker.tlb_flushes, 0);
            assert_eq!(tracker.correct_memory_accesses, 0);
        }

        #[test]
        fn equals() {
            assert_eq!(Tracker::new(), Tracker::new());
        }

        #[test]
        fn to_string() {
            let tracker = Tracker::new();
            let str = tracker.to_string();
            assert!(!str.is_empty())
        }
    }
}
