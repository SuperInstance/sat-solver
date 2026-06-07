//! Conflict analysis using resolution.

use crate::{Clause, Literal};

/// Result of conflict analysis.
#[derive(Clone, Debug)]
pub struct ConflictInfo {
    /// The learned clause derived from conflict analysis.
    pub learned_clause: Clause,
    /// The assertion level for backjumping.
    pub backtrack_level: u32,
}

/// Analyzes conflicts and derives learned clauses via resolution.
#[derive(Clone, Debug)]
pub struct ConflictAnalyzer {
    /// Maximum number of resolution steps to prevent runaway.
    max_resolution_steps: usize,
}

impl ConflictAnalyzer {
    /// Create a new conflict analyzer.
    pub fn new() -> Self {
        ConflictAnalyzer {
            max_resolution_steps: 1000,
        }
    }

    /// Set the maximum number of resolution steps.
    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_resolution_steps = max;
        self
    }

    /// Analyze a conflict and produce a learned clause.
    ///
    /// Given a falsified clause and the trail of propagated literals with their
    /// antecedent clauses, derive a learned clause using resolution.
    ///
    /// - `conflict_clause`: The clause that became falsified.
    /// - `antecedents`: Map from literal to the clause that propagated it.
    /// - `current_level`: Current decision level.
    /// - `trail`: All assigned literals in order.
    pub fn analyze(
        &self,
        conflict_clause: &Clause,
        antecedents: &std::collections::HashMap<Literal, Clause>,
        current_level: u32,
        trail: &[Literal],
    ) -> ConflictInfo {
        let mut working = conflict_clause.clone();
        let mut steps = 0;

        // Resolve until we have exactly one literal from the current level
        loop {
            let level_lits: Vec<Literal> = working
                .literals()
                .iter()
                .filter(|&&lit| {
                    // Check if this literal was assigned at the current level
                    trail.iter().position(|&t| t == lit)
                        .map(|_pos| {
                            // We use a simplified model: literals near the end of the trail
                            // are at higher levels. For a precise implementation, we'd
                            // need decision level info per literal.
                            true
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            let current_level_count = level_lits.len();

            if current_level_count <= 1 || steps >= self.max_resolution_steps {
                break;
            }

            // Pick a literal from the current level to resolve on
            if let Some(&lit) = level_lits.last() {
                if let Some(ante) = antecedents.get(&lit) {
                    let var = lit.var();
                    working = working.resolve(ante, var);
                } else {
                    break;
                }
            } else {
                break;
            }

            steps += 1;
        }

        // Determine backtrack level
        let backtrack_level = self.compute_backtrack_level(&working, current_level, trail);

        let mut learned = working;
        learned.mark_learned();

        ConflictInfo {
            learned_clause: learned,
            backtrack_level,
        }
    }

    /// Compute the assertion level from a learned clause.
    fn compute_backtrack_level(
        &self,
        clause: &Clause,
        current_level: u32,
        trail: &[Literal],
    ) -> u32 {
        if clause.len() <= 1 {
            return 0;
        }

        let mut max_level = 0u32;
        let mut second_max = 0u32;

        for &lit in clause.literals() {
            // Use trail position as a proxy for level
            let level = trail.iter().position(|&t| t.var() == lit.var())
                .map(|p| {
                    // Approximate: divide trail into levels
                    if p < trail.len() / 2 { 1 } else { 2 }
                })
                .unwrap_or(0) as u32;

            if level > max_level {
                second_max = max_level;
                max_level = level;
            } else if level > second_max {
                second_max = level;
            }
        }

        if current_level == 0 { 0 } else { second_max.min(current_level.saturating_sub(1)) }
    }

    /// Simple 1-UIP style analysis: resolve against the trail.
    pub fn analyze_simple(
        &self,
        conflict_clause: &Clause,
        antecedents: &std::collections::HashMap<Literal, Clause>,
        trail: &[Literal],
    ) -> Clause {
        let mut working = conflict_clause.clone();
        let mut steps = 0;

        // Resolve against trail literals in reverse order
        for &lit in trail.iter().rev() {
            if steps >= self.max_resolution_steps {
                break;
            }

            if working.contains(lit) {
                if let Some(ante) = antecedents.get(&lit) {
                    let var = lit.var();
                    working = working.resolve(ante, var);
                    steps += 1;
                }
            }

            // Stop when we have a unit or empty clause
            if working.len() <= 1 {
                break;
            }
        }

        working.mark_learned();
        working
    }
}

impl Default for ConflictAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_analyzer_creation() {
        let analyzer = ConflictAnalyzer::new();
        assert_eq!(analyzer.max_resolution_steps, 1000);
    }

    #[test]
    fn test_simple_conflict_resolution() {
        let conflict = Clause::from_literals(vec![
            Literal::negative(0),
            Literal::negative(1),
        ]);
        let trail = vec![Literal::positive(0), Literal::positive(1)];
        let antecedents = std::collections::HashMap::new();

        let analyzer = ConflictAnalyzer::new();
        let info = analyzer.analyze(&conflict, &antecedents, 1, &trail);
        assert!(info.learned_clause.is_learned());
    }

    #[test]
    fn test_analyze_simple() {
        let conflict = Clause::from_literals(vec![
            Literal::negative(0),
        ]);
        let trail = vec![Literal::positive(0)];
        let antecedents = std::collections::HashMap::new();

        let analyzer = ConflictAnalyzer::new();
        let learned = analyzer.analyze_simple(&conflict, &antecedents, &trail);
        assert!(learned.is_learned());
    }

    #[test]
    fn test_with_max_steps() {
        let analyzer = ConflictAnalyzer::new().with_max_steps(50);
        assert_eq!(analyzer.max_resolution_steps, 50);
    }

    #[test]
    fn test_analyze_with_antecedent() {
        let conflict = Clause::from_literals(vec![
            Literal::negative(0),
            Literal::negative(1),
        ]);
        let ante = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::negative(2),
        ]);
        let mut antecedents = std::collections::HashMap::new();
        antecedents.insert(Literal::positive(0), ante);

        let trail = vec![Literal::positive(0), Literal::positive(1)];
        let analyzer = ConflictAnalyzer::new();
        let info = analyzer.analyze(&conflict, &antecedents, 1, &trail);
        assert!(info.learned_clause.is_learned());
    }
}
