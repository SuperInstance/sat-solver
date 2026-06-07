# sat-solver

A Boolean satisfiability (SAT) solver implementing the DPLL algorithm with clause learning and conflict-driven backtracking.

## Features

- **DPLL Algorithm**: Complete search with systematic variable assignments
- **Unit Propagation**: Automatically deduces forced assignments from unit clauses
- **Pure Literal Elimination**: Assigns literals that appear in only one polarity
- **Clause Learning**: Conflict-driven clause learning (CDCL-style) with resolution
- **Conflict Analysis**: Resolution-based conflict analysis to derive learned clauses
- **Backtracking**: Decision-level backtracking with trail management

## Installation

```toml
[dependencies]
sat-solver = "0.1.0"
```

## Usage

```rust
use sat_solver::{Literal, Clause, Solver};

let mut solver = Solver::new();

// Add clauses: (x0 OR x1) AND (NOT x0 OR x2)
solver.add_clause(Clause::from_literals(vec![
    Literal::positive(0),
    Literal::positive(1),
]));
solver.add_clause(Clause::from_literals(vec![
    Literal::negative(0),
    Literal::positive(2),
]));

let result = solver.solve();
assert!(result.is_sat());

if let Some(model) = result.model() {
    println!("x0 = {}, x1 = {}, x2 = {}", model[0], model[1], model[2]);
}
```

### DIMACS-style input

```rust
use sat_solver::{Clause, Solver};

let mut solver = Solver::new();
solver.add_clause(Clause::from_ints(&[1, 2, 3]));    // (x0 OR x1 OR x2)
solver.add_clause(Clause::from_ints(&[-1, -2]));      // (NOT x0 OR NOT x1)
```

## Architecture

| Module | Description |
|--------|-------------|
| `literal` | Boolean literal representation with polarity |
| `clause` | CNF clause with evaluation and resolution |
| `solver` | DPLL solver with unit propagation and pure literal elimination |
| `conflict` | Conflict analysis and learned clause derivation |
| `backtrack` | Decision-level backtracking with trail management |

## License

MIT OR Apache-2.0
