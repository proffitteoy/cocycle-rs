//! Exact KD range queries and implicit-diagonal Hopcroft--Karp refinement.

#[cfg(any(test, cocycle_distance_bench))]
use super::bytes;
use super::{Diagnostics, NONE, Options, Pair, Route, candidates, cross, filled, reserve};
use crate::execution::WorkBudget;
use crate::{Error, Result};

#[derive(Clone, Copy)]
struct Node {
    point: usize,
    parent: usize,
    end: usize,
    low: [f64; 2],
    high: [f64; 2],
}

struct KdIndex {
    nodes: Vec<Node>,
}

impl KdIndex {
    fn new<const CONTROLLED: bool>(
        points: &[[f64; 2]],
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<Self> {
        budget.step_by(points.len())?;
        let mut indices = filled(points.len(), 0)?;
        for (index, value) in indices.iter_mut().enumerate() {
            *value = index;
        }
        let mut result = Self { nodes: Vec::new() };
        reserve(&mut result.nodes, points.len())?;
        result.build(points, &mut indices, 0, NONE, budget)?;
        Ok(result)
    }

    fn build<const CONTROLLED: bool>(
        &mut self,
        points: &[[f64; 2]],
        indices: &mut [usize],
        depth: usize,
        parent: usize,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<()> {
        budget.step_by(indices.len())?;
        if indices.is_empty() {
            return Ok(());
        }
        let middle = indices.len() / 2;
        let axis = depth % 2;
        indices.select_nth_unstable_by(middle, |&a, &b| {
            points[a][axis].total_cmp(&points[b][axis]).then(a.cmp(&b))
        });
        let point = indices[middle];
        let node = self.nodes.len();
        self.nodes.push(Node {
            point,
            parent,
            end: 0,
            low: points[point],
            high: points[point],
        });
        // Median splits bound recursion by the bit width of usize, independently
        // of input geometry. Query/augment paths below are fully iterative.
        let (left, rest) = indices.split_at_mut(middle);
        self.build(points, left, depth + 1, node, budget)?;
        self.build(points, &mut rest[1..], depth + 1, node, budget)?;
        let end = self.nodes.len();
        self.nodes[node].end = end;
        let mut child = node + 1;
        while child < end {
            for axis in 0..2 {
                self.nodes[node].low[axis] =
                    self.nodes[node].low[axis].min(self.nodes[child].low[axis]);
                self.nodes[node].high[axis] =
                    self.nodes[node].high[axis].max(self.nodes[child].high[axis]);
            }
            child = self.nodes[child].end;
        }
        Ok(())
    }

    // Keep the existing query operands separate from diagnostics and execution.
    #[allow(clippy::too_many_arguments)]
    fn next<const CONTROLLED: bool>(
        &self,
        points: &[[f64; 2]],
        query: [f64; 2],
        radius: f64,
        cursor: &mut usize,
        remaining: Option<&[usize]>,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<Option<(usize, usize)>> {
        let low = [
            (query[0] - radius).next_down(),
            (query[1] - radius).next_down(),
        ];
        let high = [(query[0] + radius).next_up(), (query[1] + radius).next_up()];
        while *cursor < self.nodes.len() {
            budget.step()?;
            let index = *cursor;
            let node = self.nodes[index];
            if remaining.is_some_and(|counts| counts[index] == 0)
                || (0..2).any(|axis| node.high[axis] < low[axis] || node.low[axis] > high[axis])
            {
                *cursor = node.end;
                continue;
            }
            *cursor += 1;
            record! { _stats.kd_nodes_visited += 1; }
            record! { _stats.adjacency_checks += 1; }
            // Outward-rounded bounding boxes only discover candidates. This
            // scalar comparison is the exact threshold membership certificate.
            if cross(query, points[node.point]) <= radius {
                return Ok(Some((node.point, index)));
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy, Default)]
struct Cursor {
    node: usize,
    diagonal: usize,
}

struct Frame {
    left: usize,
    cursor: Cursor,
    via: usize,
}

struct Oracle<'a, 'p, 'q> {
    pair: &'q Pair<'a, 'p>,
    tree: KdIndex,
    left: Vec<usize>,
    right: Vec<usize>,
    levels: Vec<usize>,
    queue: Vec<usize>,
    stack: Vec<Frame>,
    active: Vec<bool>,
    remaining: Vec<usize>,
    free_diagonal: Vec<usize>,
    radius: f64,
    used: bool,
}

impl<'a, 'p, 'q> Oracle<'a, 'p, 'q> {
    fn new<const CONTROLLED: bool>(
        pair: &'q Pair<'a, 'p>,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<Self> {
        let mut result = Self {
            pair,
            tree: KdIndex::new(pair.second.points, budget)?,
            left: filled(pair.size, NONE)?,
            right: filled(pair.size, NONE)?,
            levels: Vec::new(),
            queue: Vec::new(),
            stack: Vec::new(),
            active: Vec::new(),
            remaining: Vec::new(),
            free_diagonal: Vec::new(),
            radius: f64::INFINITY,
            used: false,
        };
        result.allocate_scratch(budget)?;
        Ok(result)
    }

    fn allocate_scratch<const CONTROLLED: bool>(
        &mut self,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<()> {
        budget.step_by(self.pair.size)?;
        self.levels = filled(self.pair.size, NONE)?;
        self.active = filled(self.pair.second.points.len(), false)?;
        self.remaining = filled(self.tree.nodes.len(), 0)?;
        self.queue = Vec::new();
        self.stack = Vec::new();
        self.free_diagonal = Vec::new();
        reserve(&mut self.queue, self.pair.size)?;
        reserve(&mut self.stack, self.pair.size)?;
        reserve(&mut self.free_diagonal, self.pair.first.points.len())?;
        Ok(())
    }

    fn next_neighbor<const CONTROLLED: bool>(
        &self,
        left: usize,
        radius: f64,
        cursor: &mut Cursor,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<Option<usize>> {
        budget.step()?;
        let n = self.pair.first.points.len();
        let m = self.pair.second.points.len();
        if left < n {
            if let Some((point, _)) = self.tree.next(
                self.pair.second.points,
                self.pair.first.points[left],
                radius,
                &mut cursor.node,
                None,
                _stats,
                budget,
            )? {
                return Ok(Some(point));
            }
            if cursor.diagonal == 0 {
                cursor.diagonal = 1;
                if self.pair.first.diagonals[left] <= radius {
                    return Ok(Some(m + left));
                }
            }
            Ok(None)
        } else {
            if cursor.node == 0 {
                cursor.node = 1;
                if self.pair.second.diagonals[left - n] <= radius {
                    return Ok(Some(left - n));
                }
            }
            if cursor.diagonal < n {
                let right = m + cursor.diagonal;
                cursor.diagonal += 1;
                Ok(Some(right))
            } else {
                Ok(None)
            }
        }
    }

    fn greedy<const CONTROLLED: bool>(
        &mut self,
        radius: f64,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<()> {
        budget.step_by(self.pair.size)?;
        for (point, active) in self.active.iter_mut().enumerate() {
            *active = self.right[point] == NONE;
        }
        self.remaining.fill(0);
        for index in (0..self.tree.nodes.len()).rev() {
            let node = self.tree.nodes[index];
            self.remaining[index] += usize::from(self.active[node.point]);
            if node.parent != NONE {
                self.remaining[node.parent] += self.remaining[index];
            }
        }
        let n = self.pair.first.points.len();
        let m = self.pair.second.points.len();
        for left in 0..n {
            budget.step()?;
            if self.left[left] != NONE {
                continue;
            }
            let mut cursor = 0;
            let mut found = NONE;
            while let Some((point, mut node)) = self.tree.next(
                self.pair.second.points,
                self.pair.first.points[left],
                radius,
                &mut cursor,
                Some(&self.remaining),
                _stats,
                budget,
            )? {
                if !self.active[point] {
                    continue;
                }
                self.active[point] = false;
                loop {
                    budget.step()?;
                    self.remaining[node] -= 1;
                    node = self.tree.nodes[node].parent;
                    if node == NONE {
                        break;
                    }
                }
                found = point;
                break;
            }
            if found == NONE
                && self.pair.first.diagonals[left] <= radius
                && self.right[m + left] == NONE
            {
                found = m + left;
            }
            if found != NONE {
                self.left[left] = found;
                self.right[found] = left;
            }
        }
        budget.step_by(self.pair.size)?;
        self.free_diagonal.clear();
        for right in m..self.pair.size {
            if self.right[right] == NONE {
                self.free_diagonal.push(right);
            }
        }
        let mut next = 0;
        for point in 0..m {
            let left = n + point;
            if self.left[left] != NONE {
                continue;
            }
            let mut right = NONE;
            if self.pair.second.diagonals[point] <= radius && self.right[point] == NONE {
                right = point;
            } else if next < self.free_diagonal.len() {
                right = self.free_diagonal[next];
                next += 1;
            }
            if right != NONE {
                self.left[left] = right;
                self.right[right] = left;
            }
        }
        Ok(())
    }

    fn bfs<const CONTROLLED: bool>(
        &mut self,
        radius: f64,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<bool> {
        budget.step_by(self.pair.size)?;
        self.queue.clear();
        self.levels.fill(NONE);
        for left in 0..self.pair.size {
            if self.left[left] == NONE {
                self.levels[left] = 0;
                self.queue.push(left);
            }
        }
        let mut found = false;
        let mut head = 0;
        while head < self.queue.len() {
            budget.step()?;
            let left = self.queue[head];
            head += 1;
            let mut cursor = Cursor::default();
            while let Some(right) = self.next_neighbor(left, radius, &mut cursor, _stats, budget)? {
                let next = self.right[right];
                if next == NONE {
                    found = true;
                } else if self.levels[next] == NONE {
                    self.levels[next] = self.levels[left] + 1;
                    self.queue.push(next);
                }
            }
        }
        Ok(found)
    }

    fn augment<const CONTROLLED: bool>(
        &mut self,
        left: usize,
        radius: f64,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<bool> {
        self.stack.clear();
        self.stack.push(Frame {
            left,
            cursor: Cursor::default(),
            via: NONE,
        });
        while !self.stack.is_empty() {
            budget.step()?;
            let last = self.stack.len() - 1;
            let left = self.stack[last].left;
            let mut cursor = self.stack[last].cursor;
            let next = self.next_neighbor(left, radius, &mut cursor, _stats, budget)?;
            self.stack[last].cursor = cursor;
            let Some(right) = next else {
                self.levels[left] = NONE;
                self.stack.pop();
                continue;
            };
            let previous = self.right[right];
            if previous == NONE {
                let mut assign = right;
                budget.step_by(self.stack.len())?;
                for frame in self.stack.iter().rev() {
                    self.left[frame.left] = assign;
                    self.right[assign] = frame.left;
                    assign = frame.via;
                }
                return Ok(true);
            }
            if self.levels[left] != NONE && self.levels[previous] == self.levels[left] + 1 {
                self.stack.push(Frame {
                    left: previous,
                    cursor: Cursor::default(),
                    via: right,
                });
            }
        }
        Ok(false)
    }

    fn within<const CONTROLLED: bool>(
        &mut self,
        radius: f64,
        _options: Options,
        _stats: &mut Diagnostics,
        budget: &mut WorkBudget<'_, CONTROLLED>,
    ) -> Result<bool> {
        budget.step_by(self.pair.size)?;
        record! { _stats.threshold_decisions += 1; }
        if radius < self.pair.lower() {
            return Ok(false);
        }
        if self.used && !experiment!(_options.reuse_scratch, true) {
            self.allocate_scratch(budget)?;
        } else if self.used {
            record! { _stats.scratch_reuses += 1; }
        }
        let reuse_matching = experiment!(_options.reuse_matching, true);
        if !reuse_matching {
            self.left.fill(NONE);
            self.right.fill(NONE);
        } else if self.used && radius < self.radius {
            for left in 0..self.pair.size {
                if CONTROLLED && left % 256 == 0 {
                    budget.step_by((self.pair.size - left).min(256))?;
                }
                let right = self.left[left];
                if right != NONE && !self.pair.allowed(left, right, radius) {
                    self.left[left] = NONE;
                    self.right[right] = NONE;
                }
            }
        }
        record! {
            if reuse_matching && self.used && self.left.iter().any(|&right| right != NONE) {
                _stats.matching_reuses += 1;
            }
        }
        self.radius = radius;
        self.used = true;
        self.greedy(radius, _stats, budget)?;
        budget.step_by(self.pair.size)?;
        let mut size = self.left.iter().filter(|&&right| right != NONE).count();
        while size < self.pair.size && self.bfs(radius, _stats, budget)? {
            let before = size;
            for left in 0..self.pair.size {
                if CONTROLLED && left % 256 == 0 {
                    budget.step_by((self.pair.size - left).min(256))?;
                }
                if self.left[left] == NONE {
                    record! { _stats.augment_searches += 1; }
                    size += usize::from(self.augment(left, radius, _stats, budget)?);
                }
            }
            if size == before {
                break;
            }
        }
        record! { _stats.workspace(self.pair.bytes().saturating_add(self.bytes())); }
        Ok(size == self.pair.size)
    }

    #[cfg(any(test, cocycle_distance_bench))]
    fn bytes(&self) -> usize {
        bytes(&self.tree.nodes)
            .saturating_add(bytes(&self.left))
            .saturating_add(bytes(&self.right))
            .saturating_add(bytes(&self.levels))
            .saturating_add(bytes(&self.queue))
            .saturating_add(bytes(&self.stack))
            .saturating_add(bytes(&self.active))
            .saturating_add(bytes(&self.remaining))
            .saturating_add(bytes(&self.free_diagonal))
    }
}

pub(super) fn distance<const CONTROLLED: bool>(
    pair: &Pair<'_, '_>,
    _options: Options,
    _stats: &mut Diagnostics,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<f64> {
    let mut oracle = Oracle::new(pair, budget)?;
    let mut lower = pair.lower();
    let mut upper = pair.upper();
    if oracle.within(lower, _options, _stats, budget)? {
        return Ok(lower);
    }
    if lower == 0.0 {
        let mut probe = upper * 0.5;
        while probe > 0.0 && oracle.within(probe, _options, _stats, budget)? {
            upper = probe;
            probe *= 0.5;
        }
        lower = probe;
    }
    while upper > lower && upper - lower > upper * 0.01 {
        let middle = lower + (upper - lower) * 0.5;
        if middle == lower || middle == upper {
            break;
        }
        if oracle.within(middle, _options, _stats, budget)? {
            upper = middle;
        } else {
            lower = middle;
        }
    }
    let mut radii = candidates(
        pair,
        Route::Refinement,
        experiment!(_options.clip_candidates, true),
        lower,
        upper,
        _stats,
        budget,
    )?;
    if experiment!(_options.clip_candidates, true) {
        radii.retain(|&value| value > lower && value <= upper);
    }
    budget.step_by(radii.len())?;
    radii.sort_unstable_by(f64::total_cmp);
    radii.dedup();
    let mut begin = 0;
    let mut end = radii.len();
    while begin < end {
        budget.step()?;
        let middle = begin + (end - begin) / 2;
        if oracle.within(radii[middle], _options, _stats, budget)? {
            end = middle;
        } else {
            begin = middle + 1;
        }
    }
    record! { _stats.workspace(
        pair.bytes()
            .saturating_add(oracle.bytes())
            .saturating_add(bytes(&radii)),
    ); }
    if let Some(&result) = radii.get(begin) {
        return Ok(result);
    }
    // A future range-index regression must never return an approximate bound.
    radii = candidates(
        pair,
        Route::Refinement,
        false,
        0.0,
        pair.upper(),
        _stats,
        budget,
    )?;
    budget.step_by(radii.len())?;
    radii.sort_unstable_by(f64::total_cmp);
    radii.dedup();
    begin = 0;
    end = radii.len();
    while begin < end {
        budget.step()?;
        let middle = begin + (end - begin) / 2;
        if oracle.within(radii[middle], _options, _stats, budget)? {
            end = middle;
        } else {
            begin = middle + 1;
        }
    }
    record! { _stats.workspace(
        pair.bytes()
            .saturating_add(oracle.bytes())
            .saturating_add(bytes(&radii)),
    ); }
    radii.get(begin).copied().ok_or(Error::InternalInvariant {
        reason: "exact bottleneck refinement has no feasible candidate",
    })
}
