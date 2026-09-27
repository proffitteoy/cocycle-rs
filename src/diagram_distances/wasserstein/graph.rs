//! Positive-saving graph storage, candidate generation and components.
//! Adapted from Topp; the parent module retains the MIT attribution.

#[cfg(any(test, cocycle_distance_bench))]
use super::capacity_bytes;
use super::{Metric, Point, Stats, allocation, buffer, product, push, saving, sum_size};
use crate::Result;
use crate::execution::WorkBudget;

#[derive(Clone, Copy, Debug)]
pub(super) struct Edge {
    pub(super) column: usize,
    pub(super) saving: f64,
}

pub(super) enum Storage {
    Dense(Vec<f64>),
    Csr {
        offsets: Vec<usize>,
        edges: Vec<Edge>,
    },
}

pub(super) struct Graph {
    pub(super) rows: usize,
    pub(super) columns: usize,
    pub(super) edge_count: usize,
    pub(super) storage: Storage,
    pub(super) direct_cost_required: bool,
}

pub(super) enum Edges<'a> {
    Dense(std::iter::Enumerate<std::slice::Iter<'a, f64>>),
    Csr(std::iter::Copied<std::slice::Iter<'a, Edge>>),
}

impl Iterator for Edges<'_> {
    type Item = Edge;
    fn next(&mut self) -> Option<Edge> {
        match self {
            Self::Dense(iter) => {
                iter.find_map(|(column, &saving)| (saving > 0.0).then_some(Edge { column, saving }))
            }
            Self::Csr(iter) => iter.next(),
        }
    }
}

impl Graph {
    #[cfg(any(test, cocycle_distance_bench))]
    pub(super) fn record_capacity(&self, _stats: &mut Stats) {
        let bytes = match &self.storage {
            Storage::Dense(values) => capacity_bytes(values),
            Storage::Csr { offsets, edges } => {
                capacity_bytes(offsets).saturating_add(capacity_bytes(edges))
            }
        };
        record! { _stats.peak_graph_storage_bytes = _stats.peak_graph_storage_bytes.max(bytes); }
    }

    // Dense iterators inspect absent edges too; CSR visits only stored edges.
    pub(super) fn row_work(&self, row: usize) -> usize {
        match &self.storage {
            Storage::Dense(_) => self.columns.max(1),
            Storage::Csr { offsets, .. } => (offsets[row + 1] - offsets[row]).max(1),
        }
    }

    pub(super) fn edges(&self, row: usize) -> Edges<'_> {
        match &self.storage {
            Storage::Dense(values) => Edges::Dense(
                values[row * self.columns..(row + 1) * self.columns]
                    .iter()
                    .enumerate(),
            ),
            Storage::Csr { offsets, edges } => {
                Edges::Csr(edges[offsets[row]..offsets[row + 1]].iter().copied())
            }
        }
    }

    pub(super) fn from_candidates<const CONTROLLED: bool>(
        rows: Vec<Vec<Edge>>,
        columns: usize,
        force_csr: bool,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<Self> {
        budget.step_by(rows.len())?;
        let row_count = rows.len();
        let pairs = product(row_count, columns)?;
        let edge_count = rows.iter().try_fold(0, |n, row| sum_size(n, row.len()))?;
        let density = if pairs == 0 {
            0.0
        } else {
            edge_count as f64 / pairs as f64
        };
        let storage = if !force_csr && (pairs <= 1024 || density >= 0.15) {
            budget.step_by(pairs)?;
            let mut values = buffer(pairs, 0.0)?;
            for (row, edges) in rows.iter().enumerate() {
                budget.step_by(edges.len())?;
                for edge in edges {
                    values[row * columns + edge.column] = edge.saving;
                }
            }
            Storage::Dense(values)
        } else {
            let mut offsets = Vec::new();
            offsets
                .try_reserve_exact(sum_size(row_count, 1)?)
                .map_err(|_| allocation())?;
            let mut edges = Vec::new();
            edges
                .try_reserve_exact(edge_count)
                .map_err(|_| allocation())?;
            offsets.push(0);
            for row in rows {
                budget.step_by(row.len().max(1))?;
                edges.extend(row);
                offsets.push(edges.len());
            }
            Storage::Csr { offsets, edges }
        };
        Ok(Self {
            rows: row_count,
            columns,
            edge_count,
            storage,
            direct_cost_required: false,
        })
    }

    pub(super) fn density(&self) -> f64 {
        if self.rows == 0 || self.columns == 0 {
            0.0
        } else {
            self.edge_count as f64 / (self.rows * self.columns) as f64
        }
    }

    pub(super) fn active<const CONTROLLED: bool>(
        &self,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<(Vec<usize>, Vec<usize>)> {
        let mut rows = Vec::new();
        let mut used = buffer(self.columns, false)?;
        for row in 0..self.rows {
            if CONTROLLED {
                budget.step_by(self.row_work(row))?;
            }
            let mut active = false;
            for edge in self.edges(row) {
                active = true;
                used[edge.column] = true;
            }
            if active {
                push(&mut rows, row)?;
            }
        }
        let mut columns = Vec::new();
        for (column, present) in used.into_iter().enumerate() {
            if CONTROLLED && column % 256 == 0 {
                budget.step_by((self.columns - column).min(256))?;
            }
            if present {
                push(&mut columns, column)?;
            }
        }
        Ok((rows, columns))
    }
}

pub(super) fn generate<const CONTROLLED: bool>(
    first: &[Point],
    second: &[Point],
    metric: Metric,
    _stats: &mut Stats,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Graph> {
    budget.step_by(second.len())?;
    let pairs = product(first.len(), second.len())?;
    let mut candidates = buffer(first.len(), Vec::new())?;
    let mut direct_cost_required = false;
    let mut order = Vec::new();
    order
        .try_reserve_exact(second.len())
        .map_err(|_| allocation())?;
    order.extend(0..second.len());
    order.sort_unstable_by(|&a, &b| {
        second[a]
            .midpoint
            .total_cmp(&second[b].midpoint)
            .then(a.cmp(&b))
    });
    budget.step_by(second.len())?;
    let maximum_half = second.iter().fold(0.0_f64, |a, b| a.max(b.half));
    let midpoint_error = |point: Point| {
        (point.midpoint.next_up() - point.midpoint).max(point.midpoint - point.midpoint.next_down())
    };
    let second_error = second
        .iter()
        .copied()
        .map(midpoint_error)
        .fold(0.0, f64::max);
    let radius = |point: Point| match metric {
        Metric::W1 => (2.0 * point.half).next_up(),
        Metric::W2 => ((2.0 * point.half).next_up() * maximum_half)
            .next_up()
            .sqrt()
            .next_up(),
    };
    let window = |point: Point| {
        let r = (radius(point) + midpoint_error(point)).next_up() + second_error;
        let lower = (point.midpoint - r).next_down();
        let upper = (point.midpoint + r).next_up();
        let start = order.partition_point(|&i| second[i].midpoint < lower);
        let stop = order.partition_point(|&i| second[i].midpoint <= upper);
        start..stop
    };
    let sampled = first.len().min(4);
    let mut window_count = 0;
    for sample in 0..sampled {
        window_count = sum_size(
            window_count,
            window(first[sample * first.len() / sampled]).len(),
        )?;
    }
    let window_density = if sampled == 0 || second.is_empty() {
        0.0
    } else {
        window_count as f64 / product(sampled, second.len())? as f64
    };
    // SIMD/parallel reference routes intentionally use the scalar equivalent.
    let sweep = pairs > 1024 && window_density < 0.75;
    for (row, &point) in first.iter().enumerate() {
        budget.step()?;
        if sweep {
            let range = window(point);
            budget.step_by(range.len())?;
            for &column in &order[range] {
                record! { _stats.candidate_pairs += 1; }
                let (value, direct) = saving(point, second[column], metric)?;
                direct_cost_required |= direct;
                if value > 0.0 {
                    push(
                        &mut candidates[row],
                        Edge {
                            column,
                            saving: value,
                        },
                    )?;
                }
            }
        } else {
            budget.step_by(second.len())?;
            for (column, &other) in second.iter().enumerate() {
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
                }
            }
        }
        record! { _stats.positive_edges += candidates[row].len(); }
    }
    let mut graph = Graph::from_candidates(candidates, second.len(), false, budget)?;
    graph.direct_cost_required = direct_cost_required;
    Ok(graph)
}

pub(super) struct Component {
    pub(super) rows: Vec<usize>,
    pub(super) columns: Vec<usize>,
}

pub(super) fn components<const CONTROLLED: bool>(
    graph: &Graph,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Vec<Component>> {
    let mut reverse = buffer(graph.columns, Vec::new())?;
    for row in 0..graph.rows {
        if CONTROLLED {
            budget.step_by(graph.row_work(row))?;
        }
        for edge in graph.edges(row) {
            push(&mut reverse[edge.column], row)?;
        }
    }
    let mut row_seen = buffer(graph.rows, false)?;
    let mut col_seen = buffer(graph.columns, false)?;
    let mut result = Vec::new();
    let mut queue = Vec::new();
    queue
        .try_reserve_exact(sum_size(graph.rows, graph.columns)?)
        .map_err(|_| allocation())?;
    for start in 0..graph.rows {
        if CONTROLLED {
            budget.step_by(graph.row_work(start))?;
        }
        if row_seen[start] || graph.edges(start).next().is_none() {
            continue;
        }
        let mut component = Component {
            rows: Vec::new(),
            columns: Vec::new(),
        };
        queue.clear();
        queue.push((true, start));
        row_seen[start] = true;
        let mut next = 0;
        while next < queue.len() {
            let (is_row, index) = queue[next];
            next += 1;
            if is_row {
                if CONTROLLED {
                    budget.step_by(graph.row_work(index))?;
                }
                push(&mut component.rows, index)?;
                for edge in graph.edges(index) {
                    if !col_seen[edge.column] {
                        col_seen[edge.column] = true;
                        queue.push((false, edge.column));
                    }
                }
            } else {
                if CONTROLLED {
                    budget.step_by(reverse[index].len().max(1))?;
                }
                push(&mut component.columns, index)?;
                for &row in &reverse[index] {
                    if !row_seen[row] {
                        row_seen[row] = true;
                        queue.push((true, row));
                    }
                }
            }
        }
        push(&mut result, component)?;
    }
    Ok(result)
}

pub(super) fn groups<const CONTROLLED: bool>(
    points: &[Point],
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<(Vec<Point>, Vec<usize>)> {
    budget.step_by(points.len())?;
    let mut order = Vec::new();
    order
        .try_reserve_exact(points.len())
        .map_err(|_| allocation())?;
    order.extend(0..points.len());
    order.sort_unstable_by(|&a, &b| {
        points[a].coordinates[0]
            .total_cmp(&points[b].coordinates[0])
            .then(points[a].coordinates[1].total_cmp(&points[b].coordinates[1]))
    });
    let mut unique: Vec<Point> = Vec::new();
    let mut multiplicity: Vec<usize> = Vec::new();
    for (position, index) in order.into_iter().enumerate() {
        if CONTROLLED && position % 256 == 0 {
            budget.step_by((points.len() - position).min(256))?;
        }
        if unique
            .last()
            .is_some_and(|p| p.coordinates == points[index].coordinates)
        {
            let last = multiplicity.len() - 1;
            multiplicity[last] += 1;
        } else {
            push(&mut unique, points[index])?;
            push(&mut multiplicity, 1)?;
        }
    }
    Ok((unique, multiplicity))
}
