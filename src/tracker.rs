#[derive(Debug, PartialEq)]
pub struct Tracker {
    pub page_faults: usize,
    pub page_hits: usize,
    pub tlb_faults: usize,
    pub tlb_hits: usize,
    pub correct_memory_accesses: usize,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            page_faults: 0,
            page_hits: 0,
            tlb_faults: 0,
            tlb_hits: 0,
            correct_memory_accesses: 0,
        }
    }
}

impl std::fmt::Display for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Stats Tracked
---------------------------------
page_faults:             {:08}
page_hits:               {:08}
tlb_faults:              {:08}
tlb_hits:                {:08}
correct_memory_accesses: {:08}


tlb hit ratio:           {:.06}
page hit ratio:          {:.06}
               ",
            self.page_faults,
            self.page_hits,
            self.tlb_faults,
            self.tlb_hits,
            self.correct_memory_accesses,
            // assumes all accesses were valid
            self.tlb_hits as f32 / self.correct_memory_accesses as f32,
            self.page_hits as f32 / self.correct_memory_accesses as f32,
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
            assert_eq!(tracker.page_faults, 0);
            assert_eq!(tracker.page_hits, 0);
            assert_eq!(tracker.tlb_faults, 0);
            assert_eq!(tracker.tlb_hits, 0);
            assert_eq!(tracker.correct_memory_accesses, 0);
        }

        #[test]
        fn equals() {
            assert_eq!(Tracker::new(), Tracker::new());
        }
    }
}
