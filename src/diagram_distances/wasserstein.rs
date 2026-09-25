//! Scalar diagonal-saving matching, adapted from Topp 1.0.1.
//!
//! Reference: Topp `ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234`,
//! `src/wasserstein.cpp`. Candidate, storage and matcher routing retain the
//! reference thresholds. SIMD and parallel candidate loops use scalar loops.
//!
//! Copyright (c) 2026 Topp contributors
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use crate::{Error, Result};

#[cfg(any(test, cocycle_distance_bench))]
#[path = "wasserstein/instrumentation.rs"]
mod instrumentation;
#[cfg(any(test, cocycle_distance_bench))]
pub(crate) use instrumentation::{Options, Stats};
#[cfg(not(any(test, cocycle_distance_bench)))]
#[derive(Clone, Copy, Default)]
struct Options {}
#[cfg(not(any(test, cocycle_distance_bench)))]
#[derive(Default)]
struct Stats {}

mod dense;
mod direct;
mod graph;
mod numeric;

use dense::{certified_greedy, dense_sap, tiny};
use graph::{Edge, Graph, components, generate, groups};
use numeric::{
    Point, cross_power, diagonal_power, finite, from_flows, from_matching, numerical, power_scale,
    prepare, restore_scale, saving,
};
mod sparse;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Metric {
    W1,
    W2,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SparseLayout {
    #[default]
    Vectors,
    #[cfg(any(test, cocycle_distance_bench))]
    Arena,
}

#[cfg(any(test, cocycle_distance_bench))]
fn capacity_bytes<T>(values: &Vec<T>) -> usize {
    values.capacity().saturating_mul(std::mem::size_of::<T>())
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "Wasserstein workspace",
    }
}

fn sum_size(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or(Error::SizeOverflow {
        operation: "Wasserstein workspace size",
    })
}

fn product(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or(Error::SizeOverflow {
        operation: "Wasserstein pair count",
    })
}

fn buffer<T: Clone>(length: usize, value: T) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(length).map_err(|_| allocation())?;
    result.resize(length, value);
    Ok(result)
}

fn push<T>(target: &mut Vec<T>, value: T) -> Result<()> {
    target.try_reserve(1).map_err(|_| allocation())?;
    target.push(value);
    Ok(())
}

fn solve_graph(
    graph: &Graph,
    dense: Option<bool>,
    greedy: bool,
    metric: Metric,
    _options: Options,
    _stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    if greedy
        && (graph.rows.max(graph.columns) <= 32 || graph.density() <= 0.05)
        && let Some(matching) = certified_greedy(graph, _stats)?
    {
        return Ok(matching);
    }
    let use_dense = dense.unwrap_or(graph.rows.max(graph.columns) <= 24 || graph.density() >= 0.15);
    if use_dense {
        dense_sap(
            graph,
            dense.is_none() && metric == Metric::W1 && graph.rows.max(graph.columns) >= 512,
            _stats,
        )
    } else {
        let flows = sparse::solve(
            graph,
            None,
            experiment!(_options.sparse, SparseLayout::Vectors),
            _stats,
        )?;
        let mut matching = buffer(graph.rows, None)?;
        for (row, column, amount) in flows {
            if amount != 1 {
                return Err(Error::InternalInvariant {
                    reason: "unit Wasserstein matching has non-unit flow",
                });
            }
            matching[row] = Some(column);
        }
        Ok(matching)
    }
}

fn solve_components(
    graph: &Graph,
    metric: Metric,
    _options: Options,
    _stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    let mut matching = buffer(graph.rows, None)?;
    let mut column_map = buffer(graph.columns, 0)?;
    for component in components(graph)? {
        record! { _stats.components += 1; }
        if component.rows.len() == 1 || component.columns.len() == 1 {
            let mut best = 0.0;
            let mut pair = None;
            for &row in &component.rows {
                for edge in graph.edges(row) {
                    if edge.saving > best {
                        best = edge.saving;
                        pair = Some((row, edge.column));
                    }
                }
            }
            if let Some((row, column)) = pair {
                matching[row] = Some(column);
            }
            record! { _stats.tiny_components += 1; }
            continue;
        }
        for (local, &column) in component.columns.iter().enumerate() {
            column_map[column] = local;
        }
        let mut candidates = buffer(component.rows.len(), Vec::new())?;
        for (local, &row) in component.rows.iter().enumerate() {
            for edge in graph.edges(row) {
                push(
                    &mut candidates[local],
                    Edge {
                        column: column_map[edge.column],
                        saving: edge.saving,
                    },
                )?;
            }
        }
        let local = Graph::from_candidates(candidates, component.columns.len(), true)?;
        record! { local.record_capacity(_stats); }
        let local_matching = if sum_size(local.rows, local.columns)? <= 8 {
            record! { _stats.tiny_components += 1; }
            tiny(&local)?
        } else {
            let dense = local.rows.max(local.columns) <= 24 || local.density() >= 0.20;
            solve_graph(&local, Some(dense), !dense, metric, _options, _stats)?
        };
        for (row, column) in local_matching.into_iter().enumerate() {
            if let Some(column) = column {
                matching[component.rows[row]] = Some(component.columns[column]);
            }
        }
    }
    Ok(matching)
}

pub(crate) fn distance(first: &[[f64; 2]], second: &[[f64; 2]], metric: Metric) -> Result<f64> {
    solve(
        first,
        second,
        metric,
        Options::default(),
        &mut Stats::default(),
    )
}

#[cfg(any(test, cocycle_distance_bench))]
pub(crate) fn distance_with_options(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    metric: Metric,
    _options: Options,
    _stats: &mut Stats,
) -> Result<f64> {
    solve(first, second, metric, _options, _stats)
}

fn solve(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    metric: Metric,
    _options: Options,
    _stats: &mut Stats,
) -> Result<f64> {
    let scale = power_scale(first, second);
    let first = prepare(first, scale)?;
    let second = prepare(second, scale)?;
    let original_pairs = product(first.len(), second.len())?;
    if !experiment!(_options.force_sparse, false) {
        let (first_groups, rows) = groups(&first)?;
        let (second_groups, columns) = groups(&second)?;
        record! { _stats.duplicate_groups = sum_size(first_groups.len(), second_groups.len())?; }
        let removed = first_groups.len() != first.len() || second_groups.len() != second.len();
        if removed && product(first_groups.len(), second_groups.len())? <= original_pairs / 16 {
            let mut candidates = buffer(first_groups.len(), Vec::new())?;
            let mut direct_cost_required = false;
            for (row, &point) in first_groups.iter().enumerate() {
                for (column, &other) in second_groups.iter().enumerate() {
                    record! { _stats.candidate_pairs += 1; }
                    let (value, direct) = saving(point, other, metric)?;
                    direct_cost_required |= direct;
                    if value > 0.0 {
                        push(
                            &mut candidates[row],
                            Edge {
                                column,
                                saving: value,
                            },
                        )?;
                        record! { _stats.positive_edges += 1; }
                    }
                }
            }
            if direct_cost_required {
                let matching = direct::matching(&first, &second, metric, _stats)?;
                return restore_scale(from_matching(&first, &second, matching, metric)?, scale);
            }
            let graph = Graph::from_candidates(candidates, second_groups.len(), true)?;
            record! { graph.record_capacity(_stats); }
            let flows = sparse::solve(
                &graph,
                Some((&rows, &columns)),
                experiment!(_options.sparse, SparseLayout::Vectors),
                _stats,
            )?;
            return restore_scale(
                from_flows(
                    &first_groups,
                    &second_groups,
                    &rows,
                    &columns,
                    &flows,
                    metric,
                )?,
                scale,
            );
        }
    }
    let graph = generate(&first, &second, metric, _stats)?;
    record! { graph.record_capacity(_stats); }
    let matching = if graph.direct_cost_required {
        direct::matching(&first, &second, metric, _stats)?
    } else if experiment!(_options.force_sparse, false) {
        solve_graph(&graph, Some(false), false, metric, _options, _stats)?
    } else if graph.density() >= 0.15 {
        solve_graph(&graph, None, true, metric, _options, _stats)?
    } else {
        solve_components(&graph, metric, _options, _stats)?
    };
    restore_scale(from_matching(&first, &second, matching, metric)?, scale)
}
