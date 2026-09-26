//! Hand-derived columns exercise the algorithm without a Builder or source adapter.
use super::{BoundaryInput, diagram};
use crate::algebra::{PrimeField, column::Column, reduction};
use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use crate::execution::{Execution, WorkBudget};
use crate::{Error, Result};

fn triangle(field: PrimeField, filled: bool) -> Result<BoundaryInput<&'static str>> {
    let mut input = BoundaryInput::new();
    for (id, value) in [("a", -4.), ("b", -3.), ("c", -2.)] {
        input.push(id, 0, value, Column::new())?;
    }
    for (id, value, a, b) in [("ab", 0., 0, 1), ("ac", 1., 0, 2), ("bc", 2., 1, 2)] {
        let mut boundary = Column::new();
        boundary.add_term(a, field.characteristic() - 1, field);
        boundary.add_term(b, 1, field);
        input.push(id, 1, value, boundary)?;
    }
    if filled {
        let mut boundary = Column::new();
        boundary.add_term(3, 1, field);
        boundary.add_term(4, field.characteristic() - 1, field);
        boundary.add_term(5, 1, field);
        input.push("abc", 2, 3., boundary)?;
    }
    Ok(input)
}

#[test]
fn signed_births_and_delayed_filling_have_hand_derived_pairs() -> Result<()> {
    for prime in [2, 3, 65537] {
        let field = PrimeField::new(prime)?;
        for filled in [false, true] {
            let coverage = if filled {
                Coverage::Complete
            } else {
                Coverage::Through(2.)
            };
            let survivor = if filled {
                IntervalEnd::Essential
            } else {
                IntervalEnd::RightCensored { through: 2. }
            };
            let h1 = if filled {
                IntervalEnd::Finite(3.)
            } else {
                survivor
            };
            let expected = PersistenceDiagram::new(
                1,
                coverage,
                vec![
                    PersistenceInterval::new(0, -4., survivor)?,
                    PersistenceInterval::new(0, -3., IntervalEnd::Finite(0.))?,
                    PersistenceInterval::new(0, -2., IntervalEnd::Finite(1.))?,
                    PersistenceInterval::new(1, 2., h1)?,
                ],
            )?;
            let actual = diagram(
                triangle(field, filled)?,
                1,
                field,
                coverage,
                &mut WorkBudget::new(&Execution::default())?,
            )?;
            assert_eq!(actual, expected);
        }
    }
    Ok(())
}

#[test]
fn cellular_integer_incidence_can_change_homology_with_the_field() -> Result<()> {
    for prime in [2, 3, 65537] {
        let field = PrimeField::new(prime)?;
        let mut input = BoundaryInput::new();
        input.push("point", 0, -4., Column::new())?;
        input.push("loop", 1, -2., Column::new())?;
        let mut disk = Column::new();
        disk.add_term(1, 2 % prime, field);
        input.push("disk", 2, 1., disk)?;
        let actual = diagram(
            input,
            2,
            field,
            Coverage::Complete,
            &mut WorkBudget::new(&Execution::default())?,
        )?;
        let h1 = actual.intervals_in_dimension(1)?.next().unwrap();
        assert_eq!(h1.birth(), -2.);
        assert_eq!(
            h1.end(),
            if prime == 2 {
                IntervalEnd::Essential
            } else {
                IntervalEnd::Finite(1.)
            }
        );
        assert_eq!(
            actual.intervals_in_dimension(2)?.count(),
            usize::from(prime == 2)
        );
    }
    Ok(())
}

#[test]
fn transformations_satisfy_dv_equals_r_and_unique_normalized_pivots() -> Result<()> {
    for prime in [2, 3, 251] {
        let field = PrimeField::new(prime)?;
        let columns = triangle(field, true)?.columns;
        let original = columns.clone();
        let reduced = reduction::reduce(columns, field, &mut || Ok(()))?;
        let mut pivots = std::collections::BTreeSet::new();
        for (j, column) in reduced.reduced.iter().enumerate() {
            // Independent dense multiplication using integer arithmetic modulo p.
            for row in 0..original.len() {
                let product: u64 = reduced.transforms[j]
                    .entries()
                    .map(|(&k, &c)| u64::from(original[k].get(&row)) * u64::from(c))
                    .sum();
                assert_eq!(product % u64::from(prime), u64::from(column.get(&row)));
            }
            assert!(reduced.transforms[j].entries().all(|(&k, _)| k <= j));
            if let Some((&row, &coefficient)) = column.last() {
                assert!(row < j && pivots.insert(row));
                assert_eq!(coefficient, 1);
                assert_eq!(reduced.deaths[row], Some(j));
            }
        }
        let pairs = reduction::reduce_pairs(original, field, &mut || Ok(()))?;
        assert!(pairs.transforms.is_empty());
        assert_eq!(pairs.deaths, reduced.deaths);
    }
    Ok(())
}

#[test]
fn interruption_does_not_poison_a_fresh_computation() -> Result<()> {
    let field = PrimeField::new(3)?;
    let expected = diagram(
        triangle(field, true)?,
        1,
        field,
        Coverage::Complete,
        &mut WorkBudget::new(&Execution::default())?,
    )?;
    let mut failures = 0;
    for limit in 0..200 {
        match diagram(
            triangle(field, true)?,
            1,
            field,
            Coverage::Complete,
            &mut WorkBudget::new(&Execution::default().max_work(limit))?,
        ) {
            Err(Error::WorkLimitExceeded { .. }) => failures += 1,
            Ok(actual) => {
                assert_eq!(actual, expected);
                assert!(failures > 1);
                return Ok(());
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }
    panic!("small boundary computation never completed")
}
