//! Literal representation for Boolean variables.

use std::fmt;

/// A Boolean literal: a variable that may be negated.
///
/// Encoded as `index = 2 * var` for positive, `2 * var + 1` for negative.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    /// Internal encoding: even = positive, odd = negative.
    code: u32,
}

impl Literal {
    /// Create a positive (non-negated) literal for variable `var`.
    pub fn positive(var: u32) -> Self {
        Literal { code: var * 2 }
    }

    /// Create a negative (negated) literal for variable `var`.
    pub fn negative(var: u32) -> Self {
        Literal { code: var * 2 + 1 }
    }

    /// Create a literal from a signed integer (positive = true, negative = negated).
    /// Variable indices start at 1 for this interface.
    pub fn from_int(val: i32) -> Self {
        if val > 0 {
            Literal::positive((val - 1) as u32)
        } else {
            Literal::negative((-val - 1) as u32)
        }
    }

    /// The variable index (0-based).
    pub fn var(self) -> u32 {
        self.code / 2
    }

    /// Whether this literal is positive (not negated).
    pub fn is_positive(self) -> bool {
        self.code.is_multiple_of(2)
    }

    /// Whether this literal is negative (negated).
    pub fn is_negative(self) -> bool {
        self.code % 2 == 1
    }

    /// Negate this literal.
    pub fn negated(self) -> Literal {
        Literal { code: self.code ^ 1 }
    }

    /// Evaluate this literal under a (partial) truth assignment.
    /// Returns `None` if the variable is unassigned.
    pub fn evaluate(self, assignment: &[Option<bool>]) -> Option<bool> {
        let var = self.var() as usize;
        if var >= assignment.len() {
            return None;
        }
        match assignment[var] {
            Some(val) => {
                if self.is_positive() {
                    Some(val)
                } else {
                    Some(!val)
                }
            }
            None => None,
        }
    }

    /// Internal code for hashing/indexing.
    pub fn code(self) -> u32 {
        self.code
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_positive() {
            write!(f, "x{}", self.var())
        } else {
            write!(f, "¬x{}", self.var())
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_literal() {
        let lit = Literal::positive(3);
        assert_eq!(lit.var(), 3);
        assert!(lit.is_positive());
        assert!(!lit.is_negative());
    }

    #[test]
    fn test_negative_literal() {
        let lit = Literal::negative(5);
        assert_eq!(lit.var(), 5);
        assert!(!lit.is_positive());
        assert!(lit.is_negative());
    }

    #[test]
    fn test_negation() {
        let pos = Literal::positive(2);
        let neg = pos.negated();
        assert_eq!(neg.var(), 2);
        assert!(neg.is_negative());
        assert_eq!(neg.negated(), pos);
    }

    #[test]
    fn test_from_int() {
        let pos = Literal::from_int(3);
        assert_eq!(pos.var(), 2);
        assert!(pos.is_positive());

        let neg = Literal::from_int(-3);
        assert_eq!(neg.var(), 2);
        assert!(neg.is_negative());
    }

    #[test]
    fn test_evaluate_assigned() {
        let lit = Literal::positive(1);
        let assignment = vec![Some(false), Some(true)];
        assert_eq!(lit.evaluate(&assignment), Some(true));

        let neg_lit = Literal::negative(1);
        assert_eq!(neg_lit.evaluate(&assignment), Some(false));
    }

    #[test]
    fn test_evaluate_unassigned() {
        let lit = Literal::positive(5);
        let assignment = vec![Some(true)]; // only var 0
        assert_eq!(lit.evaluate(&assignment), None);
    }

    #[test]
    fn test_ordering() {
        let a = Literal::positive(1);
        let b = Literal::negative(1);
        let c = Literal::positive(2);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn test_display_format() {
        assert_eq!(format!("{}", Literal::positive(3)), "x3");
        assert_eq!(format!("{}", Literal::negative(3)), "¬x3");
    }
}
