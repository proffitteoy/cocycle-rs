//! Dense shortest augmenting paths and small matching certificates.
//! Adapted from Topp; the parent module retains the MIT attribution.

use super::graph::Storage;
use super::{Graph, Stats, allocation, buffer, finite, product, sum_size};
use crate::Result;

pub(super) fn certified_greedy(
    graph: &Graph,
    _stats: &mut Stats,
) -> Result<Option<Vec<Option<usize>>>> {
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
        for edge in graph.edges(row) {
            *maximum = maximum.max(edge.saving);
            column_max[edge.column] = column_max[edge.column].max(edge.saving);
            edges.push((row, edge));
        }
    }
    edges.sort_unstable_by(|a, b| {
        b.1.saving
            .total_cmp(&a.1.saving)
            .then(a.0.cmp(&b.0))
            .then(a.1.column.cmp(&b.1.column))
    });
    let mut matching = buffer(graph.rows, None)?;
    let mut column_match = buffer(graph.columns, None)?;
    let mut weight = buffer(graph.rows, 0.0)?;
    for (row, edge) in edges {
        if matching[row].is_none() && column_match[edge.column].is_none() {
            matching[row] = Some(edge.column);
            column_match[edge.column] = Some(row);
            weight[row] = edge.saving;
        }
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

pub(super) fn dense_sap(
    graph: &Graph,
    row_reduction: bool,
    _stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    record! { _stats.dense_solves += 1; }
    let (rows, columns) = graph.active()?;
    let mut matching = buffer(graph.rows, None)?;
    if rows.is_empty() || columns.is_empty() {
        return Ok(matching);
    }
    let mut lookup;
    let values = match &graph.storage {
        Storage::Dense(values) => values,
        Storage::Csr { .. } => {
            lookup = buffer(product(graph.rows, graph.columns)?, 0.0)?;
            for row in 0..graph.rows {
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
        matched[0] = row;
        let mut column = 0;
        minimum.fill(f64::INFINITY);
        used.fill(false);
        loop {
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
            let previous = predecessor[column];
            matched[column] = matched[previous];
            column = previous;
            if column == 0 {
                break;
            }
        }
        record! { _stats.augmentations += 1; }
    }
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

pub(super) fn tiny(graph: &Graph) -> Result<Vec<Option<usize>>> {
    fn visit(
        graph: &Graph,
        row: usize,
        value: f64,
        current: &mut [Option<usize>],
        used: &mut [bool],
        best: &mut (f64, Vec<Option<usize>>),
    ) {
        if row == graph.rows {
            if value > best.0 {
                best.0 = value;
                best.1.copy_from_slice(current);
            }
            return;
        }
        visit(graph, row + 1, value, current, used, best);
        for edge in graph.edges(row) {
            if used[edge.column] {
                continue;
            }
            used[edge.column] = true;
            current[row] = Some(edge.column);
            visit(graph, row + 1, value + edge.saving, current, used, best);
            used[edge.column] = false;
            current[row] = None;
        }
    }
    let mut best = (0.0, buffer(graph.rows, None)?);
    visit(
        graph,
        0,
        0.0,
        &mut buffer(graph.rows, None)?,
        &mut buffer(graph.columns, false)?,
        &mut best,
    );
    Ok(best.1)
}
