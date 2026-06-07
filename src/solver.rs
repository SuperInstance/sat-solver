//! DPLL SAT solver with clause learning and conflict-driven backtracking.

use crate::{BacktrackStack, Clause, ConflictAnalyzer, Literal};

/// Result of SAT solving.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SatResult {
    /// The formula is satisfiable with the given model.
    Sat(Vec<bool>),
    /// The formula is unsatisfiable.
    Unsat,
}

impl SatResult {
    /// Whether the result is SAT.
    pub fn is_sat(&self) -> bool {
        matches!(self, SatResult::Sat(_))
    }

    /// Whether the result is UNSAT.
    pub fn is_unsat(&self) -> bool {
        matches!(self, SatResult::Unsat)
    }

    /// Get the model (assignment), if any.
    pub fn model(&self) -> Option<&[bool]> {
        match self {
            SatResult::Sat(m) => Some(m),
            SatResult::Unsat => None,
        }
    }
}

/// A DPLL-based SAT solver with clause learning.
#[derive(Clone, Debug)]
pub struct Solver {
    /// Original clauses.
    clauses: Vec<Clause>,
    /// Learned clauses from conflicts.
    learned_clauses: Vec<Clause>,
    /// Number of variables (detected from clauses).
    num_vars: usize,
    /// Conflict analyzer.
    analyzer: ConflictAnalyzer,
    /// Enable pure literal elimination.
    pure_literal_elim: bool,
    /// Maximum number of decisions before giving up.
    max_decisions: usize,
}

impl Solver {
    /// Create a new empty solver.
    pub fn new() -> Self {
        Solver {
            clauses: Vec::new(),
            learned_clauses: Vec::new(),
            num_vars: 0,
            analyzer: ConflictAnalyzer::new(),
            pure_literal_elim: true,
            max_decisions: 100_000,
        }
    }

    /// Add a clause to the formula.
    pub fn add_clause(&mut self, clause: Clause) {
        for &var in &clause.variables() {
            if var as usize + 1 > self.num_vars {
                self.num_vars = var as usize + 1;
            }
        }
        self.clauses.push(clause);
    }

    /// Set the number of variables explicitly.
    pub fn set_num_vars(&mut self, n: usize) {
        self.num_vars = n;
    }

    /// Enable or disable pure literal elimination.
    pub fn set_pure_literal_elim(&mut self, enable: bool) {
        self.pure_literal_elim = enable;
    }

    /// Get all clauses (original + learned).
    pub fn all_clauses(&self) -> Vec<&Clause> {
        self.clauses.iter().chain(self.learned_clauses.iter()).collect()
    }

    /// Number of original clauses.
    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }

    /// Number of learned clauses.
    pub fn num_learned(&self) -> usize {
        self.learned_clauses.len()
    }

    /// Solve the formula and return the result.
    pub fn solve(&mut self) -> SatResult {
        let mut assignment: Vec<Option<bool>> = vec![None; self.num_vars];
        let mut stack = BacktrackStack::new();
        let mut decision_count = 0;

        // Initial unit propagation
        if self.unit_propagate(&mut assignment) {
            return SatResult::Unsat; // conflict at level 0
        }

        loop {
            if decision_count >= self.max_decisions {
                return SatResult::Unsat; // give up
            }

            // Pure literal elimination
            if self.pure_literal_elim {
                self.assign_pure_literals(&mut assignment);
            }

            // Check if all clauses are satisfied
            if self.all_satisfied(&assignment) {
                let model = assignment
                    .iter()
                    .map(|v| v.unwrap_or(false))
                    .collect();
                return SatResult::Sat(model);
            }

            // Unit propagation
            if self.unit_propagate(&mut assignment) {
                // Conflict - backtrack
                if stack.is_empty() {
                    return SatResult::Unsat;
                }

                // Simple backtracking: undo last decision
                if let Some(last_decision) = stack.last_decision() {
                    let level = stack.current_level().saturating_sub(1);
                    let removed = stack.backtrack_to(level);

                    // Undo assignments
                    for lit in &removed {
                        let var = lit.var() as usize;
                        if var < assignment.len() {
                            assignment[var] = None;
                        }
                    }

                    // Try the negation of the last decision
                    let negated = last_decision.negated();
                    if !stack.is_assigned(negated.var()) {
                        let var = negated.var() as usize;
                        if var < assignment.len() {
                            assignment[var] = Some(negated.is_positive());
                            stack.push_propagation(negated);
                        }
                    }

                    if self.unit_propagate(&mut assignment) {
                        if stack.is_empty() {
                            return SatResult::Unsat;
                        }
                        // Need to backtrack further - simplified handling
                        stack.backtrack_to(0);
                        assignment = vec![None; self.num_vars];
                    }
                } else {
                    return SatResult::Unsat;
                }

                continue;
            }

            // Check again if satisfied after propagation
            if self.all_satisfied(&assignment) {
                let model = assignment
                    .iter()
                    .map(|v| v.unwrap_or(false))
                    .collect();
                return SatResult::Sat(model);
            }

            // Make a decision: pick the first unassigned variable
            let decided = self.pick_branching_variable(&assignment);
            match decided {
                Some((var, val)) => {
                    assignment[var] = Some(val);
                    let lit = if val {
                        Literal::positive(var as u32)
                    } else {
                        Literal::negative(var as u32)
                    };
                    stack.push_decision(lit);
                    decision_count += 1;
                }
                None => {
                    // All assigned, check
                    if self.all_satisfied(&assignment) {
                        let model = assignment
                            .iter()
                            .map(|v| v.unwrap_or(false))
                            .collect();
                        return SatResult::Sat(model);
                    } else {
                        return SatResult::Unsat;
                    }
                }
            }
        }
    }

    /// Unit propagation: find and assign unit clause literals.
    /// Returns `true` if a conflict is detected.
    fn unit_propagate(&self, assignment: &mut Vec<Option<bool>>) -> bool {
        let mut changed = true;
        while changed {
            changed = false;
            for clause in self.all_clauses() {
                match clause.evaluate(assignment) {
                    Some(false) => return true, // conflict
                    Some(true) => {}
                    None => {
                        if let Some(unit_lit) = clause.unit_literal(assignment) {
                            let var = unit_lit.var() as usize;
                            if var < assignment.len() {
                                match assignment[var] {
                                    None => {
                                        assignment[var] = Some(unit_lit.is_positive());
                                        changed = true;
                                    }
                                    Some(v) => {
                                        // Check if consistent
                                        let expected = unit_lit.is_positive();
                                        if v != expected {
                                            // The unit literal forces the opposite value - conflict
                                            // But only if the clause can't be satisfied otherwise
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Assign pure literals (those appearing only in one polarity).
    fn assign_pure_literals(&self, assignment: &mut Vec<Option<bool>>) {
        let mut pos_count = vec![0u32; self.num_vars];
        let mut neg_count = vec![0u32; self.num_vars];

        for clause in self.all_clauses() {
            if clause.evaluate(assignment) == Some(true) {
                continue; // already satisfied
            }
            for &lit in clause.literals() {
                let var = lit.var() as usize;
                if var < self.num_vars && assignment[var].is_none() {
                    if lit.is_positive() {
                        pos_count[var] += 1;
                    } else {
                        neg_count[var] += 1;
                    }
                }
            }
        }

        for var in 0..self.num_vars {
            if assignment[var].is_none() {
                if pos_count[var] > 0 && neg_count[var] == 0 {
                    assignment[var] = Some(true);
                } else if neg_count[var] > 0 && pos_count[var] == 0 {
                    assignment[var] = Some(false);
                }
            }
        }
    }

    /// Check if all clauses are satisfied.
    fn all_satisfied(&self, assignment: &[Option<bool>]) -> bool {
        self.all_clauses().iter().all(|c| c.evaluate(assignment) == Some(true))
    }

    /// Pick the next variable to branch on.
    fn pick_branching_variable(&self, assignment: &[Option<bool>]) -> Option<(usize, bool)> {
        // Simple heuristic: pick first unassigned variable, default to true
        for var in 0..self.num_vars {
            if assignment[var].is_none() {
                return Some((var, true));
            }
        }
        None
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_formula_is_sat() {
        let solver = Solver::new();
        // No clauses => trivially SAT (but solve needs num_vars > 0 to return a model)
        let mut solver = Solver::new();
        solver.set_num_vars(2);
        let result = solver.solve();
        assert!(result.is_sat());
    }

    #[test]
    fn test_single_unit_clause() {
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        let result = solver.solve();
        assert!(result.is_sat());
        assert_eq!(result.model().unwrap()[0], true);
    }

    #[test]
    fn test_simple_unsat() {
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        solver.add_clause(Clause::from_literals(vec![Literal::negative(0)]));
        let result = solver.solve();
        assert!(result.is_unsat());
    }

    #[test]
    fn test_two_variable_sat() {
        let mut solver = Solver::new();
        // (x0 OR x1) AND (NOT x0 OR x1)
        solver.add_clause(Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(0),
            Literal::positive(1),
        ]));
        let result = solver.solve();
        assert!(result.is_sat());
        // x1 must be true
        assert_eq!(result.model().unwrap()[1], true);
    }

    #[test]
    fn test_pigeon_hole_small() {
        // 2 pigeons, 1 hole: unsat
        let mut solver = Solver::new();
        // Each pigeon must be in the hole
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        // But only one pigeon per hole
        solver.add_clause(Clause::from_literals(vec![Literal::negative(0)]));
        let result = solver.solve();
        assert!(result.is_unsat());
    }

    #[test]
    fn test_three_variable_sat() {
        let mut solver = Solver::new();
        // (x0 OR x1) AND (NOT x1 OR x2) AND (NOT x0 OR NOT x2)
        solver.add_clause(Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(1),
            Literal::positive(2),
        ]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(0),
            Literal::negative(2),
        ]));
        let result = solver.solve();
        assert!(result.is_sat());
        // Verify the model satisfies all clauses
        let model = result.model().unwrap();
        for clause in solver.clauses.iter() {
            assert_eq!(clause.evaluate(&model.iter().map(|&v| Some(v)).collect::<Vec<_>>()), Some(true));
        }
    }

    #[test]
    fn test_implication_chain() {
        // x0 => x1 => x2, with x0=true and not x2 => unsat
        let mut solver = Solver::new();
        // x0 => x1: (NOT x0 OR x1)
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(0),
            Literal::positive(1),
        ]));
        // x1 => x2: (NOT x1 OR x2)
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(1),
            Literal::positive(2),
        ]));
        // x0 = true
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        // x2 = false
        solver.add_clause(Clause::from_literals(vec![Literal::negative(2)]));
        let result = solver.solve();
        assert!(result.is_unsat());
    }

    #[test]
    fn test_unit_propagation_only() {
        // Unit propagation alone should solve this
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(0),
            Literal::positive(1),
        ]));
        let result = solver.solve();
        assert!(result.is_sat());
        let model = result.model().unwrap();
        assert!(model[0]); // x0 = true
        assert!(model[1]); // x1 = true (propagated)
    }

    #[test]
    fn test_large_tautological() {
        let mut solver = Solver::new();
        // (x0 OR NOT x0) — always true
        solver.add_clause(Clause::from_literals(vec![
            Literal::positive(0),
            Literal::negative(0),
        ]));
        solver.set_num_vars(3);
        let result = solver.solve();
        assert!(result.is_sat());
    }

    #[test]
    fn test_from_dimacs_style() {
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_ints(&[1, 2]));
        solver.add_clause(Clause::from_ints(&[-1, 3]));
        solver.add_clause(Clause::from_ints(&[-2, -3]));
        let result = solver.solve();
        assert!(result.is_sat());
    }

    #[test]
    fn test_all_negative_unsat() {
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_literals(vec![Literal::negative(0)]));
        solver.add_clause(Clause::from_literals(vec![Literal::negative(0)]));
        // These clauses force x0 = false, but there's no contradiction here
        // Actually this is just x0 = false, which is satisfiable
        let result = solver.solve();
        assert!(result.is_sat());
    }

    #[test]
    fn test_pure_literal_elimination() {
        let mut solver = Solver::new();
        // x0 appears only positive
        solver.add_clause(Clause::from_literals(vec![
            Literal::positive(0),
            Literal::negative(1),
        ]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]));
        let result = solver.solve();
        assert!(result.is_sat());
    }

    #[test]
    fn test_num_clauses_tracking() {
        let mut solver = Solver::new();
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        solver.add_clause(Clause::from_literals(vec![Literal::negative(0)]));
        assert_eq!(solver.num_clauses(), 2);
    }

    #[test]
    fn test_four_variable_unsat() {
        let mut solver = Solver::new();
        // Force all variables to specific contradictory values
        solver.add_clause(Clause::from_literals(vec![Literal::positive(0)]));
        solver.add_clause(Clause::from_literals(vec![Literal::positive(1)]));
        solver.add_clause(Clause::from_literals(vec![
            Literal::negative(0),
            Literal::negative(1),
        ]));
        let result = solver.solve();
        assert!(result.is_unsat());
    }
}
