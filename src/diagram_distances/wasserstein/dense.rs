//! Dense shortest augmenting paths and small matching certificates.
//! Adapted from Topp; the parent module retains the MIT attribution.

use super::graph::Storage;
use super::{Graph, Stats, allocation, buffer, finite, product, sum_size};
use crate::Result;
use crate::execution::WorkBudget;

pub(super) fn certified_greedy<const CONTROLLED: bool>(
    graph: &Graph,
    _stats: &mut Stats,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Option<Vec<Option<usize>>>> {
    budget.check()?;
    if graph.edge_count == 0 {
        return Ok(None);
    }
    let mut edges = Vec::new();
    edges
        .try_reserve_exact(graph.edge_count)
        .map_err(|_| allocation())?;
    let mut row_max = buffer(graph.rows, 0.0_f64)?;
    let mut column_max = buffer(graph.columns, 0.0_f64)?;
    for (row, maximum) in row_max.iter_mut().enumerate() {
        if CONTROLLED {
            budget.step_by(graph.row_work(row))?;
        }
        for edge in graph.edges(row) {
            *maximum = maximum.max(edge.saving);
            column_max[edge.column] = column_max[edge.column].max(edge.saving);
            edges.push((row, edge));
        }
    }
    budget.step_by(edges.len())?;
    edges.sort_unstable_by(|a, b| {
        b.1.saving
            .total_cmp(&a.1.saving)
            .then(a.0.cmp(&b.0))
            .then(a.1.column.cmp(&b.1.column))
    });
    let mut matching = buffer(graph.rows, None)?;
    let mut column_match = buffer(graph.columns, None)?;
    let mut weight = buffer(graph.rows, 0.0)?;
    for (position, (row, edge)) in edges.into_iter().enumerate() {
        if CONTROLLED && position % 256 == 0 {
            budget.step_by((graph.edge_count - position).min(256))?;
        }
        if matching[row].is_none() && column_match[edge.column].is_none() {
            matching[row] = Some(edge.column);
            column_match[edge.column] = Some(row);
            weight[row] = edge.saving;
        }
    }
    if CONTROLLED {
        budget.step_by(sum_size(graph.rows, graph.columns)?)?;
    }
    let row_bound = row_max.iter().zip(&weight).all(|(a, b)| a == b);
    let column_bound = column_max
        .iter()
        .zip(&column_match)
        .all(|(&a, &row)| a == 0.0 || row.is_some_and(|r| weight[r] == a));
    if row_bound || column_bound {
        record! { _stats.greedy_certificates += 1; }
        Ok(Some(matching))
    } else {
        Ok(None)
    }
}

pub(super) fn dense_sap<const CONTROLLED: bool>(
    graph: &Graph,
    row_reduction: bool,
    _stats: &mut Stats,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Vec<Option<usize>>> {
    record! { _stats.dense_solves += 1; }
    let (rows, columns) = graph.active(budget)?;
    let mut matching = buffer(graph.rows, None)?;
    if rows.is_empty() || columns.is_empty() {
        return Ok(matching);
    }
    let mut lookup;
    let values = match &graph.storage {
        Storage::Dense(values) => values,
        Storage::Csr { .. } => {
            if CONTROLLED {
                budget.step_by(product(graph.rows, graph.columns)?)?;
            }
            lookup = buffer(product(graph.rows, graph.columns)?, 0.0)?;
            for row in 0..graph.rows {
                if CONTROLLED {
                    budget.step_by(graph.row_work(row))?;
                }
                for edge in graph.edges(row) {
                    lookup[row * graph.columns + edge.column] = edge.saving;
                }
            }
            &lookup
        }
    };
    let transposed = rows.len() > columns.len();
    let short = rows.len().min(columns.len());
    let long = rows.len().max(columns.len());
    let cost = |r: usize, c: usize| {
        let (row, column) = if transposed {
            (rows[c], columns[r])
        } else {
            (rows[r], columns[c])
        };
        -values[row * graph.columns + column]
    };
    let short_len = sum_size(short, 1)?;
    let long_len = sum_size(long, 1)?;
    let mut u = buffer(short_len, 0.0)?;
    let mut v = buffer(long_len, 0.0)?;
    let mut minimum = buffer(long_len, 0.0)?;
    let mut matched = buffer(long_len, 0)?;
    let mut predecessor = buffer(long_len, 0)?;
    let mut used = buffer(long_len, false)?;
    let mut initially_matched = buffer(short_len, false)?;
    if row_reduction {
        for row in 1..=short {
            budget.step_by(long)?;
            let mut best = f64::INFINITY;
            let mut column = 0;
            for c in 1..=long {
                let value = cost(row - 1, c - 1);
                if value < best {
                    best = value;
                    column = c;
                }
            }
            u[row] = best;
            if matched[column] == 0 {
                matched[column] = row;
                initially_matched[row] = true;
            }
        }
    }
    for (row, &already_matched) in initially_matched.iter().enumerate().skip(1) {
        if already_matched {
            continue;
        }
        budget.step_by(long_len)?;
        matched[0] = row;
        let mut column = 0;
        minimum.fill(f64::INFINITY);
        used.fill(false);
        loop {
            budget.step_by(long_len)?;
            used[column] = true;
            let current = matched[column];
            let mut delta = f64::INFINITY;
            let mut next = 0;
            for c in 1..=long {
                if used[c] {
                    continue;
                }
                let reduced = finite(cost(current - 1, c - 1) - u[current] - v[c])?;
                if reduced < minimum[c] {
                    minimum[c] = reduced;
                    predecessor[c] = column;
                }
                if minimum[c] < delta {
                    delta = minimum[c];
                    next = c;
                }
            }
            finite(delta)?;
            budget.step_by(long_len)?;
            for c in 0..=long {
                if used[c] {
                    u[matched[c]] = finite(u[matched[c]] + delta)?;
                    v[c] = finite(v[c] - delta)?;
                } else {
                    minimum[c] -= delta;
                }
            }
            column = next;
            if matched[column] == 0 {
                break;
            }
        }
        loop {
            budget.step()?;
            let previous = predecessor[column];
            matched[column] = matched[previous];
            column = previous;
            if column == 0 {
                break;
            }
        }
        record! { _stats.augmentations += 1; }
    }
    budget.step_by(long)?;
    for column in 1..=long {
        let row = matched[column];
        if row == 0 {
            continue;
        }
        let (r, c) = if transposed {
            (rows[column - 1], columns[row - 1])
        } else {
            (rows[row - 1], columns[column - 1])
        };
        if values[r * graph.columns + c] > 0.0 {
            matching[r] = Some(c);
        }
    }
    Ok(matching)
}

pub(super) fn tiny<const CONTROLLED: bool>(
    graph: &Graph,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Vec<Option<usize>>> {
    fn visit<const CONTROLLED: bool>(
        graph: &Graph,
        row: usize,
        value: f64,
        current: &mut [Option<usize>],
        used: &mut [bool],
        best: &mut (f64, Vec<Option<usize>>),
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<()> {
        budget.step()?;
        if row == graph.rows {
            if value > best.0 {
                best.0 = value;
                best.1.copy_from_slice(current);
            }
            return Ok(());
        }
        visit(graph, row + 1, value, current, used, best, budget)?;
        for edge in graph.edges(row) {
            if used[edge.column] {
                continue;
            }
            used[edge.column] = true;
            current[row] = Some(edge.column);
            visit(
                graph,
                row + 1,
                value + edge.saving,
                current,
                used,
                best,
                budget,
            )?;
            used[edge.column] = false;
            current[row] = None;
        }
        Ok(())
    }
    let mut best = (0.0, buffer(graph.rows, None)?);
    visit(
        graph,
        0,
        0.0,
        &mut buffer(graph.rows, None)?,
        &mut buffer(graph.columns, false)?,
        &mut best,
        budget,
    )?;
    Ok(best.1)
}
