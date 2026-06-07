//! # sat-solver
//!
//! A Boolean satisfiability (SAT) solver implementing the DPLL algorithm with
//! clause learning and conflict-driven backtracking.
//!
//! ## Features
//!
//! - DPLL complete search with unit propagation
//! - Pure literal elimination
//! - Conflict-driven clause learning (CDCL-style)
//! - Conflict analysis with resolution proofs
//! - Backtracking with decision levels
//!
//! ## Example
//!
//! ```
//! use sat_solver::{Literal, Clause, Solver};
//!
//! let mut solver = Solver::new();
//! // (x1 OR x2) AND (NOT x1 OR x3)
//! solver.add_clause(Clause::from_literals(vec![
//!     Literal::positive(0),
//!     Literal::negative(1),
//! ]));
//! solver.add_clause(Clause::from_literals(vec![
//!     Literal::negative(0),
//!     Literal::positive(2),
//! ]));
//! let result = solver.solve();
//! assert!(result.is_sat());
//! ```

mod literal;
mod clause;
mod solver;
mod conflict;
mod backtrack;

pub use literal::Literal;
pub use clause::Clause;
pub use solver::{Solver, SatResult};
pub use conflict::ConflictAnalyzer;
pub use backtrack::BacktrackStack;
