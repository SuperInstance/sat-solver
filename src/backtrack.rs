//! Backtracking support with decision levels.

use crate::Literal;

/// A trail entry recording an assignment and its reason.
#[derive(Clone, Debug)]
pub struct TrailEntry {
    /// The literal that was assigned.
    pub literal: Literal,
    /// The decision level at which this was assigned.
    pub level: u32,
    /// Whether this was a decision (vs propagated).
    pub is_decision: bool,
}

/// A stack supporting backtracking to specific decision levels.
#[derive(Clone, Debug)]
pub struct BacktrackStack {
    trail: Vec<TrailEntry>,
    /// Map from decision level to trail index.
    level_starts: Vec<usize>,
    current_level: u32,
}

impl BacktrackStack {
    /// Create a new empty backtrack stack.
    pub fn new() -> Self {
        BacktrackStack {
            trail: Vec::new(),
            level_starts: vec![0],
            current_level: 0,
        }
    }

    /// Current decision level.
    pub fn current_level(&self) -> u32 {
        self.current_level
    }

    /// Push a decision literal, incrementing the decision level.
    pub fn push_decision(&mut self, lit: Literal) {
        self.current_level += 1;
        self.level_starts.push(self.trail.len());
        self.trail.push(TrailEntry {
            literal: lit,
            level: self.current_level,
            is_decision: true,
        });
    }

    /// Push a propagated literal at the current decision level.
    pub fn push_propagation(&mut self, lit: Literal) {
        self.trail.push(TrailEntry {
            literal: lit,
            level: self.current_level,
            is_decision: false,
        });
    }

    /// Backtrack to a given decision level, removing all assignments above it.
    pub fn backtrack_to(&mut self, level: u32) -> Vec<Literal> {
        let mut removed = Vec::new();
        if (level as usize) + 1 < self.level_starts.len() {
            let keep_until = self.level_starts[level as usize + 1];
            while self.trail.len() > keep_until {
                if let Some(entry) = self.trail.pop() {
                    removed.push(entry.literal);
                }
            }
            self.level_starts.truncate(level as usize + 2);
            self.current_level = level;
        }
        removed
    }

    /// Get all literals on the trail.
    pub fn trail(&self) -> &[TrailEntry] {
        &self.trail
    }

    /// Get all literals assigned at or below the given level.
    pub fn trail_up_to_level(&self, level: u32) -> Vec<&TrailEntry> {
        self.trail.iter().filter(|e| e.level <= level).collect()
    }

    /// Convert the trail to a partial assignment.
    pub fn to_assignment(&self, num_vars: usize) -> Vec<Option<bool>> {
        let mut assignment = vec![None; num_vars];
        for entry in &self.trail {
            let var = entry.literal.var() as usize;
            if var < num_vars {
                assignment[var] = Some(entry.literal.is_positive());
            }
        }
        assignment
    }

    /// Check if a literal is already assigned on the trail.
    pub fn is_assigned(&self, var: u32) -> bool {
        self.trail.iter().any(|e| e.literal.var() == var)
    }

    /// Number of assignments on the trail.
    pub fn len(&self) -> usize {
        self.trail.len()
    }

    /// Whether the trail is empty.
    pub fn is_empty(&self) -> bool {
        self.trail.is_empty()
    }

    /// Get the most recent decision literal.
    pub fn last_decision(&self) -> Option<Literal> {
        self.trail.iter().rev().find(|e| e.is_decision).map(|e| e.literal)
    }

    /// Clear the entire trail.
    pub fn clear(&mut self) {
        self.trail.clear();
        self.level_starts = vec![0];
        self.current_level = 0;
    }
}

impl Default for BacktrackStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stack() {
        let stack = BacktrackStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.current_level(), 0);
    }

    #[test]
    fn test_push_decision() {
        let mut stack = BacktrackStack::new();
        stack.push_decision(Literal::positive(0));
        assert_eq!(stack.current_level(), 1);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.last_decision(), Some(Literal::positive(0)));
    }

    #[test]
    fn test_push_propagation() {
        let mut stack = BacktrackStack::new();
        stack.push_propagation(Literal::negative(1));
        assert_eq!(stack.current_level(), 0);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_backtrack() {
        let mut stack = BacktrackStack::new();
        stack.push_decision(Literal::positive(0)); // level 1
        stack.push_propagation(Literal::negative(1)); // level 1
        stack.push_decision(Literal::positive(2)); // level 2
        stack.push_propagation(Literal::negative(3)); // level 2

        let removed = stack.backtrack_to(1);
        assert_eq!(stack.current_level(), 1);
        assert_eq!(removed.len(), 2);
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_to_assignment() {
        let mut stack = BacktrackStack::new();
        stack.push_decision(Literal::positive(0));
        stack.push_propagation(Literal::negative(1));
        let assignment = stack.to_assignment(3);
        assert_eq!(assignment[0], Some(true));
        assert_eq!(assignment[1], Some(false));
        assert_eq!(assignment[2], None);
    }

    #[test]
    fn test_is_assigned() {
        let mut stack = BacktrackStack::new();
        stack.push_decision(Literal::positive(0));
        assert!(stack.is_assigned(0));
        assert!(!stack.is_assigned(1));
    }

    #[test]
    fn test_clear() {
        let mut stack = BacktrackStack::new();
        stack.push_decision(Literal::positive(0));
        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.current_level(), 0);
    }
}
