/// Criteria categories for deterministic execution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeterministicCriteria {
    /// Same inputs always produce the same output value
    OutputConsistency,
    /// Same inputs always emit the same number of events
    EventCountConsistency,
    /// Same inputs always emit the same event content
    EventContentConsistency,
    /// Same inputs always produce the same storage writes
    StorageConsistency,
    /// Contract does not use non-deterministic sources
    NoNonDeterministicSources,
    /// Iteration over collections is order-independent
    CollectionOrderIndependence,
    /// Cross-contract calls produce consistent results
    CrossContractConsistency,
}

impl DeterministicCriteria {
    pub fn description(&self) -> &'static str {
        match self {
            Self::OutputConsistency => "Output Consistency: identical inputs produce identical output values",
            Self::EventCountConsistency => "Event Count Consistency: identical inputs produce identical event counts",
            Self::EventContentConsistency => "Event Content Consistency: identical inputs emit identical event data",
            Self::StorageConsistency => "Storage Consistency: identical inputs write the same storage keys and values",
            Self::NoNonDeterministicSources => "No Non-Deterministic Sources: contract avoids randomness, unix time, or external data",
            Self::CollectionOrderIndependence => "Collection Order Independence: results do not depend on iteration order of maps/sets",
            Self::CrossContractConsistency => "Cross-Contract Consistency: cross-contract calls produce deterministic results",
        }
    }
}

/// Set of deterministic criteria to verify for a given operation.
pub struct VerificationCriteria {
    pub criteria: Vec<DeterministicCriteria>,
}

impl VerificationCriteria {
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    pub fn add(&mut self, c: DeterministicCriteria) {
        self.criteria.push(c);
    }

    pub fn all() -> Self {
        let mut v = Self::new();
        v.add(DeterministicCriteria::OutputConsistency);
        v.add(DeterministicCriteria::EventCountConsistency);
        v.add(DeterministicCriteria::EventContentConsistency);
        v.add(DeterministicCriteria::StorageConsistency);
        v.add(DeterministicCriteria::NoNonDeterministicSources);
        v.add(DeterministicCriteria::CollectionOrderIndependence);
        v.add(DeterministicCriteria::CrossContractConsistency);
        v
    }

    pub fn len(&self) -> u32 {
        self.criteria.len() as u32
    }

    pub fn get(&self, i: u32) -> Option<&DeterministicCriteria> {
        self.criteria.get(i as usize)
    }
}
