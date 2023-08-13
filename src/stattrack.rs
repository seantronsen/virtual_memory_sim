use std::ops::{Add, AddAssign};

/* IDEA:
 * Tracking the statistics of each memory operation will likely look messy so it may be best to
 * hold off on the feature addition to the simulation until later, when it's complete. Still, we
 * can scheme for the idea so it's ready to go when the time comes.
 *
 * Overall, the simplest approach would be to have a statistical store of information that we can
 * modify as needed. However, this would require a sleu of mutable references which might drag us
 * down to hell. As such, it will be simpler to create objects as needed and combine them when
 * appropriate.
 *
 * Our current issue with this approach is that it's exceptionally verbose. Any time the programmer
 * wishes to add any tracked stats, they will have to initialize an object and modify it before
 * passing it back.
 *
 *
 */

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StatTracker {
    pub page_faults: usize,
    pub page_hits: usize,
    pub tlb_faults: usize,
    pub tlb_hits: usize,
    pub frame_hits: usize,
    pub frame_faults: usize,
    pub correct_memory_accesses: usize,
}

impl StatTracker {
    pub fn new() -> Self {
        Self {
            page_faults: 0,
            page_hits: 0,
            tlb_faults: 0,
            tlb_hits: 0,
            frame_hits: 0,
            frame_faults: 0,
            correct_memory_accesses: 0,
        }
    }
}

impl Add<StatTracker> for StatTracker {
    type Output = StatTracker;

    fn add(self, rhs: StatTracker) -> Self::Output {
        Self::Output {
            page_faults: self.page_faults + rhs.page_faults,
            page_hits: self.page_hits + rhs.page_hits,
            tlb_faults: self.tlb_faults + rhs.tlb_faults,
            tlb_hits: self.tlb_hits + rhs.tlb_hits,
            frame_hits: self.frame_hits + rhs.frame_hits,
            frame_faults: self.frame_faults + rhs.frame_faults,
            correct_memory_accesses: self.correct_memory_accesses + rhs.correct_memory_accesses,
        }
    }
}

impl AddAssign for StatTracker {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs)
    }
}

impl std::fmt::Display for StatTracker {
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
frame_hits:              {:08}
frame_faults:            {:08}
correct_memory_accesses: {:08}
               ",
            self.page_faults,
            self.page_hits,
            self.tlb_faults,
            self.tlb_hits,
            self.frame_hits,
            self.frame_faults,
            self.correct_memory_accesses,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    mod stattracker_tests {

        use super::*;

        #[test]
        fn new() {
            let tracker = StatTracker::new();
            assert_eq!(tracker.page_faults, 0);
            assert_eq!(tracker.page_hits, 0);
            assert_eq!(tracker.tlb_faults, 0);
            assert_eq!(tracker.tlb_hits, 0);
            assert_eq!(tracker.frame_hits, 0);
            assert_eq!(tracker.frame_faults, 0);
            assert_eq!(tracker.correct_memory_accesses, 0);
        }

        #[test]
        fn equals() {
            assert_eq!(StatTracker::new(), StatTracker::new());
        }

        #[test]
        fn add() {
            let (mut a, mut b) = (StatTracker::new(), StatTracker::new());
            a.page_hits += 1;
            b.page_faults += 5;
            a += b;
            assert!(a.page_hits == 1);
            assert!(a.page_faults == 5);
        }
    }
}
