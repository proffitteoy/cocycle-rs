//! Lower-bounded circulation for mandatory points and exact multiplicities.

#[cfg(any(test, cocycle_distance_bench))]
use super::bytes;
use super::{Diagnostics, NONE, Pair, filled, reserve, size_overflow};
use crate::Result;

#[derive(Clone, Copy)]
struct Edge {
    target: usize,
    reverse: usize,
    capacity: usize,
}

struct Dinic {
    edges: Vec<Vec<Edge>>,
    levels: Vec<usize>,
    cursors: Vec<usize>,
    queue: Vec<usize>,
    path: Vec<(usize, usize)>,
}

impl Dinic {
    fn new(size: usize) -> Result<Self> {
        let mut queue = Vec::new();
        let mut path = Vec::new();
        reserve(&mut queue, size)?;
        reserve(&mut path, size)?;
        Ok(Self {
            edges: filled(size, Vec::new())?,
            levels: filled(size, NONE)?,
            cursors: filled(size, 0)?,
            queue,
            path,
        })
    }

    fn edge(&mut self, source: usize, target: usize, capacity: usize) -> Result<()> {
        let reverse = self.edges[target].len();
        let forward = self.edges[source].len();
        // Callers never insert self-edges; all circulation nodes are distinct.
        reserve(&mut self.edges[source], 1)?;
        reserve(&mut self.edges[target], 1)?;
        self.edges[source].push(Edge {
            target,
            reverse,
            capacity,
        });
        self.edges[target].push(Edge {
            target: source,
            reverse: forward,
            capacity: 0,
        });
        Ok(())
    }

    fn max_flow(
        &mut self,
        source: usize,
        sink: usize,
        required: usize,
        _stats: &mut Diagnostics,
    ) -> Result<usize> {
        let mut total = 0_usize;
        while total < required {
            self.levels.fill(NONE);
            self.queue.clear();
            self.levels[source] = 0;
            self.queue.push(source);
            for head in 0..self.edges.len() {
                if head == self.queue.len() {
                    break;
                }
                let node = self.queue[head];
                for edge in &self.edges[node] {
                    if edge.capacity > 0 && self.levels[edge.target] == NONE {
                        self.levels[edge.target] = self.levels[node] + 1;
                        self.queue.push(edge.target);
                    }
                }
            }
            if self.levels[sink] == NONE {
                break;
            }
            self.cursors.fill(0);
            loop {
                // Heap-backed DFS avoids recursion proportional to an augmenting path.
                self.path.clear();
                let mut node = source;
                loop {
                    if node == sink {
                        break;
                    }
                    let mut found = false;
                    while self.cursors[node] < self.edges[node].len() {
                        let edge = self.edges[node][self.cursors[node]];
                        record! { _stats.adjacency_checks += 1; }
                        if edge.capacity > 0 && self.levels[edge.target] == self.levels[node] + 1 {
                            self.path.push((node, self.cursors[node]));
                            node = edge.target;
                            found = true;
                            break;
                        }
                        self.cursors[node] += 1;
                    }
                    if found {
                        continue;
                    }
                    self.levels[node] = NONE;
                    match self.path.pop() {
                        Some((parent, _)) => {
                            node = parent;
                            self.cursors[node] += 1;
                        }
                        None => break,
                    }
                }
                if node != sink {
                    break;
                }
                let mut amount = required - total;
                for &(from, index) in &self.path {
                    amount = amount.min(self.edges[from][index].capacity);
                }
                for &(from, index) in &self.path {
                    let edge = self.edges[from][index];
                    self.edges[from][index].capacity -= amount;
                    self.edges[edge.target][edge.reverse].capacity = self.edges[edge.target]
                        [edge.reverse]
                        .capacity
                        .checked_add(amount)
                        .ok_or_else(size_overflow)?;
                }
                total = total.checked_add(amount).ok_or_else(size_overflow)?;
                record! { _stats.augment_searches += 1; }
                if total == required {
                    break;
                }
            }
        }
        Ok(total)
    }

    #[cfg(any(test, cocycle_distance_bench))]
    fn bytes(&self) -> usize {
        self.edges
            .iter()
            .fold(bytes(&self.edges), |sum, row| {
                sum.saturating_add(bytes(row))
            })
            .saturating_add(bytes(&self.levels))
            .saturating_add(bytes(&self.cursors))
            .saturating_add(bytes(&self.queue))
            .saturating_add(bytes(&self.path))
    }
}

pub(super) fn within(
    pair: &Pair<'_, '_>,
    radius: f64,
    grouped: bool,
    _stats: &mut Diagnostics,
    _outer_bytes: usize,
) -> Result<bool> {
    let first_indices = if grouped {
        &pair.first.representatives
    } else {
        &pair.first.order
    };
    let second_indices = if grouped {
        &pair.second.representatives
    } else {
        &pair.second.order
    };
    let n = first_indices.len();
    let m = second_indices.len();
    let source = n.checked_add(m).ok_or_else(size_overflow)?;
    let count = source.checked_add(4).ok_or_else(size_overflow)?;
    let sink = source + 1;
    let super_source = source + 2;
    let super_sink = source + 3;
    let mut flow = Dinic::new(count)?;
    let mut incoming = filled(count, 0_usize)?;
    let mut outgoing = filled(count, 0_usize)?;
    let mut first_required = filled(n, false)?;
    let mut second_required = filled(m, false)?;
    for (left, &index) in first_indices.iter().enumerate() {
        let capacity = if grouped {
            pair.first.multiplicities[left]
        } else {
            1
        };
        first_required[left] = pair.first.diagonals[index] > radius;
        let lower = if first_required[left] { capacity } else { 0 };
        flow.edge(source, left, capacity - lower)?;
        outgoing[source] = outgoing[source]
            .checked_add(lower)
            .ok_or_else(size_overflow)?;
        incoming[left] = lower;
    }
    for (right, &index) in second_indices.iter().enumerate() {
        let capacity = if grouped {
            pair.second.multiplicities[right]
        } else {
            1
        };
        second_required[right] = pair.second.diagonals[index] > radius;
        let lower = if second_required[right] { capacity } else { 0 };
        flow.edge(n + right, sink, capacity - lower)?;
        outgoing[n + right] = lower;
        incoming[sink] = incoming[sink]
            .checked_add(lower)
            .ok_or_else(size_overflow)?;
    }
    if outgoing[source] == 0 && incoming[sink] == 0 {
        return Ok(true);
    }
    for (left, &index) in first_indices.iter().enumerate() {
        let window = if grouped {
            0..m
        } else {
            pair.second.window(pair.first.points[index][0], radius)
        };
        for right in window {
            if !grouped && !first_required[left] && !second_required[right] {
                continue;
            }
            record! { _stats.adjacency_checks += 1; }
            if pair.cross(index, second_indices[right]) <= radius {
                let capacity = if grouped {
                    pair.first.multiplicities[left].min(pair.second.multiplicities[right])
                } else {
                    1
                };
                flow.edge(left, n + right, capacity)?;
                record! { _stats.capacity_edges += 1; }
            }
        }
    }
    flow.edge(sink, source, pair.size)?;
    let mut required = 0_usize;
    for node in 0..=sink {
        if incoming[node] > outgoing[node] {
            let demand = incoming[node] - outgoing[node];
            flow.edge(super_source, node, demand)?;
            required = required.checked_add(demand).ok_or_else(size_overflow)?;
        } else if outgoing[node] > incoming[node] {
            flow.edge(node, super_sink, outgoing[node] - incoming[node])?;
        }
    }
    record! { _stats.workspace(
        _outer_bytes
            .max(pair.bytes())
            .saturating_add(flow.bytes())
            .saturating_add(bytes(&incoming))
            .saturating_add(bytes(&outgoing))
            .saturating_add(bytes(&first_required))
            .saturating_add(bytes(&second_required)),
    ); }
    Ok(flow.max_flow(super_source, super_sink, required, _stats)? == required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_flow_path_uses_bounded_heap_storage() {
        let count = 8192;
        let mut flow = Dinic::new(count).unwrap();
        for node in 1..count {
            flow.edge(node - 1, node, 3).unwrap();
        }
        assert_eq!(
            flow.max_flow(0, count - 1, 3, &mut Diagnostics::default())
                .unwrap(),
            3
        );
    }
}
