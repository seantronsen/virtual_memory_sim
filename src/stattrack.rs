use std::ops::{Add, AddAssign};

#[derive(Debug, PartialEq, Copy, Clone)]
struct StatTracker {
    num_page_faults: usize,
    num_page_hits: usize,
    num_page_replacements: usize,
    num_tlb_faults: usize,
    num_tlb_hits: usize,
    num_tlb_replacements: usize,
    num_frame_hits: usize,
    num_frame_faults: usize,
    num_frame_replacements: usize,
    num_correct_memory_accesses: usize,
}

impl StatTracker {
    fn new() -> Self {
        Self {
            num_page_faults: 0,
            num_page_hits: 0,
            num_page_replacements: 0,
            num_tlb_faults: 0,
            num_tlb_hits: 0,
            num_tlb_replacements: 0,
            num_frame_hits: 0,
            num_frame_faults: 0,
            num_frame_replacements: 0,
            num_correct_memory_accesses: 0,
        }
    }
}

impl Add<StatTracker> for StatTracker {
    type Output = StatTracker;

    fn add(self, rhs: StatTracker) -> Self::Output {
        Self::Output {
            num_page_faults: self.num_page_faults + rhs.num_page_faults,
            num_page_hits: self.num_page_hits + rhs.num_page_hits,
            num_page_replacements: self.num_page_replacements + rhs.num_page_replacements,
            num_tlb_faults: self.num_tlb_faults + rhs.num_tlb_faults,
            num_tlb_hits: self.num_tlb_hits + rhs.num_tlb_hits,
            num_tlb_replacements: self.num_tlb_replacements + rhs.num_tlb_replacements,
            num_frame_hits: self.num_frame_hits + rhs.num_frame_hits,
            num_frame_faults: self.num_frame_faults + rhs.num_frame_faults,
            num_frame_replacements: self.num_frame_replacements + rhs.num_frame_replacements,
            num_correct_memory_accesses: self.num_correct_memory_accesses
                + rhs.num_correct_memory_accesses,
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
--------------------------------------------------
num_page_faults: {:08},
num_page_hits: {:08},
num_page_replacements: {:08},
num_tlb_faults: {:08},
num_tlb_hits: {:08},
num_tlb_replacements: {:08},
num_frame_hits: {:08},
num_frame_faults: {:08},
num_frame_replacements: {:08},
num_correct_memory_accesses: {:08},
               ",
            self.num_page_faults,
            self.num_page_hits,
            self.num_page_replacements,
            self.num_tlb_faults,
            self.num_tlb_hits,
            self.num_tlb_replacements,
            self.num_frame_hits,
            self.num_frame_faults,
            self.num_frame_replacements,
            self.num_correct_memory_accesses,
        )
    }
}
