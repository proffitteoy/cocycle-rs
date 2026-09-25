//! Positive-saving graph storage, candidate generation and components.
//! Adapted from Topp; the parent module retains the MIT attribution.

#[cfg(any(test, cocycle_distance_bench))]
use super::capacity_bytes;
use super::{Metric, Point, Stats, allocation, buffer, product, push, saving, sum_size};
use crate::Result;

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

    pub(super) fn from_candidates(
        rows: Vec<Vec<Edge>>,
        columns: usize,
        force_csr: bool,
    ) -> Result<Self> {
        let row_count = rows.len();
        let pairs = product(row_count, columns)?;
        let edge_count = rows.iter().try_fold(0, |n, row| sum_size(n, row.len()))?;
        let density = if pairs == 0 {
            0.0
        } else {
            edge_count as f64 / pairs as f64
        };
        let storage = if !force_csr && (pairs <= 1024 || density >= 0.15) {
            let mut values = buffer(pairs, 0.0)?;
            for (row, edges) in rows.iter().enumerate() {
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

    pub(super) fn active(&self) -> Result<(Vec<usize>, Vec<usize>)> {
        let mut rows = Vec::new();
        let mut used = buffer(self.columns, false)?;
        for row in 0..self.rows {
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
            if present {
                push(&mut columns, column)?;
            }
        }
        Ok((rows, columns))
    }
}

pub(super) fn generate(
    first: &[Point],
    second: &[Point],
    metric: Metric,
    _stats: &mut Stats,
) -> Result<Graph> {
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
        if sweep {
            for &column in &order[window(point)] {
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
    let mut graph = Graph::from_candidates(candidates, second.len(), false)?;
    graph.direct_cost_required = direct_cost_required;
    Ok(graph)
}

pub(super) struct Component {
    pub(super) rows: Vec<usize>,
    pub(super) columns: Vec<usize>,
}

pub(super) fn components(graph: &Graph) -> Result<Vec<Component>> {
    let mut reverse = buffer(graph.columns, Vec::new())?;
    for row in 0..graph.rows {
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
                push(&mut component.rows, index)?;
                for edge in graph.edges(index) {
                    if !col_seen[edge.column] {
                        col_seen[edge.column] = true;
                        queue.push((false, edge.column));
                    }
                }
            } else {
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

pub(super) fn groups(points: &[Point]) -> Result<(Vec<Point>, Vec<usize>)> {
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
    for index in order {
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
