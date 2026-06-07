//! Clause representation for CNF formulas.

use crate::Literal;

/// A disjunctive clause (OR of literals) in CNF.
#[derive(Clone, PartialEq, Eq)]
pub struct Clause {
    literals: Vec<Literal>,
    learned: bool,
}

impl Clause {
    /// Create an empty clause.
    pub fn new() -> Self {
        Clause {
            literals: Vec::new(),
            learned: false,
        }
    }

    /// Create a clause from a vector of literals.
    pub fn from_literals(lits: Vec<Literal>) -> Self {
        let mut seen = std::collections::HashSet::new();
        let mut literals = Vec::new();
        for lit in lits {
            if seen.insert(lit) {
                literals.push(lit);
            }
        }
        Clause { literals, learned: false }
    }

    /// Create a clause from signed integers (DIMACS-like).
    pub fn from_ints(vals: &[i32]) -> Self {
        Self::from_literals(vals.iter().map(|&v| Literal::from_int(v)).collect())
    }

    /// Get the literals in this clause.
    pub fn literals(&self) -> &[Literal] {
        &self.literals
    }

    /// Number of literals in the clause.
    pub fn len(&self) -> usize {
        self.literals.len()
    }

    /// Whether the clause is empty (contains no literals).
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    /// Whether this is a unit clause (exactly one literal).
    pub fn is_unit(&self) -> bool {
        self.literals.len() == 1
    }

    /// Whether this is a learned clause (from conflict analysis).
    pub fn is_learned(&self) -> bool {
        self.learned
    }

    /// Mark this clause as learned.
    pub fn mark_learned(&mut self) {
        self.learned = true;
    }

    /// Check if this clause contains a specific literal.
    pub fn contains(&self, lit: Literal) -> bool {
        self.literals.contains(&lit)
    }

    /// Evaluate the clause under a partial assignment.
    /// Returns `Some(true)` if satisfied, `Some(false)` if falsified, `None` if unresolved.
    pub fn evaluate(&self, assignment: &[Option<bool>]) -> Option<bool> {
        let mut has_unassigned = false;
        for &lit in &self.literals {
            match lit.evaluate(assignment) {
                Some(true) => return Some(true),
                Some(false) => {}
                None => has_unassigned = true,
            }
        }
        if has_unassigned {
            None
        } else {
            Some(false)
        }
    }

    /// Get the unit literal if this is a unit clause under the given assignment.
    /// Returns `None` if not a unit clause or if the clause is satisfied.
    pub fn unit_literal(&self, assignment: &[Option<bool>]) -> Option<Literal> {
        let mut unassigned = None;
        for &lit in &self.literals {
            match lit.evaluate(assignment) {
                Some(true) => return None, // already satisfied
                Some(false) => {}
                None => {
                    if unassigned.is_some() {
                        return None; // more than one unassigned
                    }
                    unassigned = Some(lit);
                }
            }
        }
        unassigned
    }

    /// Variables referenced by this clause.
    pub fn variables(&self) -> Vec<u32> {
        let mut vars: Vec<u32> = self.literals.iter().map(|l| l.var()).collect();
        vars.sort_unstable();
        vars.dedup();
        vars
    }

    /// Resolve this clause with another clause on the given variable.
    /// Returns a new clause with the resolvent.
    pub fn resolve(&self, other: &Clause, var: u32) -> Clause {
        let pos = Literal::positive(var);
        let neg = Literal::negative(var);
        let mut result = Vec::new();
        for &lit in &self.literals {
            if lit != pos && lit != neg {
                result.push(lit);
            }
        }
        for &lit in other.literals.iter() {
            if lit != pos && lit != neg && !result.contains(&lit) {
                result.push(lit);
            }
        }
        Clause::from_literals(result)
    }

    /// Add a literal to this clause.
    pub fn push(&mut self, lit: Literal) {
        if !self.literals.contains(&lit) {
            self.literals.push(lit);
        }
    }
}

impl std::fmt::Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.literals.iter().map(|l| format!("{:?}", l)).collect::<Vec<_>>().join(" ∨ "))
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_clause() {
        let c = Clause::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn test_clause_from_literals() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::negative(1),
        ]);
        assert_eq!(c.len(), 2);
        assert!(!c.is_unit());
    }

    #[test]
    fn test_unit_clause() {
        let c = Clause::from_literals(vec![Literal::positive(3)]);
        assert!(c.is_unit());
    }

    #[test]
    fn test_duplicate_literals_deduped() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(0),
        ]);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn test_evaluate_satisfied() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::negative(1),
        ]);
        let assignment = vec![Some(true), Some(false)];
        assert_eq!(c.evaluate(&assignment), Some(true));
    }

    #[test]
    fn test_evaluate_falsified() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]);
        let assignment = vec![Some(false), Some(false)];
        assert_eq!(c.evaluate(&assignment), Some(false));
    }

    #[test]
    fn test_evaluate_unresolved() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]);
        let assignment = vec![Some(false)];
        assert_eq!(c.evaluate(&assignment), None);
    }

    #[test]
    fn test_unit_literal_detection() {
        let c = Clause::from_literals(vec![
            Literal::positive(0),
            Literal::positive(1),
        ]);
        let assignment = vec![Some(false)];
        let unit = c.unit_literal(&assignment);
        assert_eq!(unit, Some(Literal::positive(1)));
    }

    #[test]
    fn test_resolve_clauses() {
        let c1 = Clause::from_literals(vec![Literal::positive(0), Literal::positive(1)]);
        let c2 = Clause::from_literals(vec![Literal::negative(0), Literal::positive(2)]);
        let resolvent = c1.resolve(&c2, 0);
        assert!(resolvent.contains(Literal::positive(1)));
        assert!(resolvent.contains(Literal::positive(2)));
        assert!(!resolvent.contains(Literal::positive(0)));
    }

    #[test]
    fn test_learned_clause() {
        let mut c = Clause::from_literals(vec![Literal::positive(0)]);
        assert!(!c.is_learned());
        c.mark_learned();
        assert!(c.is_learned());
    }

    #[test]
    fn test_from_ints() {
        let c = Clause::from_ints(&[1, -2, 3]);
        assert_eq!(c.len(), 3);
        assert!(c.contains(Literal::positive(0)));
        assert!(c.contains(Literal::negative(1)));
        assert!(c.contains(Literal::positive(2)));
    }
}
