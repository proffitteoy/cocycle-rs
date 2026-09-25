//! Original-cost numerical fallback for unresolved diagonal-saving differences.
//!
//! The ordinary kernels retain the sparse positive-saving graph. When a small
//! cross cost would lose significant bits in a large diagonal baseline, this
//! fallback assigns the augmented diagram directly. It is a square shortest
//! augmenting-path solve, with O((n+m)^3) worst-case work and O(n+m) scratch.
//! Costs are evaluated on demand; no augmented matrix is allocated.

use super::{Metric, Point, Stats, buffer, cross_power, diagonal_power, finite, sum_size};
use crate::Result;

pub(super) fn matching(
    first: &[Point],
    second: &[Point],
    metric: Metric,
    _stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    record! { _stats.direct_cost_fallbacks += 1; }
    record! { _stats.dense_solves += 1; }
    let size = sum_size(first.len(), second.len())?;
    let length = sum_size(size, 1)?;
    let cost = |row: usize, column: usize| -> Result<f64> {
        if row < first.len() {
            if column < second.len() {
                cross_power(first[row], second[column], metric)
            } else {
                diagonal_power(first[row], metric)
            }
        } else if column < second.len() {
            diagonal_power(second[column], metric)
        } else {
            Ok(0.0)
        }
    };
    // Dummy rows and columns have interchangeable diagonal copies. Every real
    // unmatched point is charged once, and dummy-to-dummy matches cost zero.
    let mut row_potential = buffer(length, 0.0)?;
    let mut column_potential = buffer(length, 0.0)?;
    let mut matched_row = buffer(length, 0)?;
    let mut predecessor = buffer(length, 0)?;
    let mut distance = buffer(length, f64::INFINITY)?;
    let mut seen = buffer(length, false)?;
    for row in 1..=size {
        matched_row[0] = row;
        distance.fill(f64::INFINITY);
        seen.fill(false);
        let mut column = 0;
        loop {
            seen[column] = true;
            let current_row = matched_row[column];
            let mut delta = f64::INFINITY;
            let mut next_column = 0;
            for candidate in 1..=size {
                if seen[candidate] {
                    continue;
                }
                let reduced = finite(
                    cost(current_row - 1, candidate - 1)?
                        - row_potential[current_row]
                        - column_potential[candidate],
                )?;
                if reduced < distance[candidate] {
                    distance[candidate] = reduced;
                    predecessor[candidate] = column;
                }
                if distance[candidate] < delta {
                    delta = distance[candidate];
                    next_column = candidate;
                }
            }
            finite(delta)?;
            for candidate in 0..=size {
                if seen[candidate] {
                    row_potential[matched_row[candidate]] =
                        finite(row_potential[matched_row[candidate]] + delta)?;
                    column_potential[candidate] = finite(column_potential[candidate] - delta)?;
                } else {
                    distance[candidate] -= delta;
                }
            }
            column = next_column;
            if matched_row[column] == 0 {
                break;
            }
        }
        loop {
            let previous = predecessor[column];
            matched_row[column] = matched_row[previous];
            column = previous;
            if column == 0 {
                break;
            }
        }
        record! { _stats.augmentations += 1; }
    }
    let mut matching = buffer(first.len(), None)?;
    for (column, &row) in matched_row
        .iter()
        .enumerate()
        .take(second.len() + 1)
        .skip(1)
    {
        if row > 0 && row <= first.len() {
            matching[row - 1] = Some(column - 1);
        }
    }
    Ok(matching)
}
