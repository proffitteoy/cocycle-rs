//! Native exact bottleneck kernel, adapted from Topp 1.0.1 (MIT).
//!
//! The adaptive gates match Topp commit ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234.
//! Inputs here contain validated finite, strictly off-diagonal points only.
//! The facade owns essential points and input/provenance validation.
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

#[path = "bottleneck/flow.rs"]
mod flow;
#[path = "bottleneck/geometry.rs"]
mod geometry;
#[path = "bottleneck/matching.rs"]
mod matching;

use crate::{Error, Result};

#[cfg(any(test, cocycle_distance_bench))]
#[path = "bottleneck/instrumentation.rs"]
mod instrumentation;
#[cfg(any(test, cocycle_distance_bench))]
pub(crate) use instrumentation::{Diagnostics, Options, Search};
#[cfg(not(any(test, cocycle_distance_bench)))]
#[derive(Clone, Copy, Default)]
struct Options {}
#[cfg(not(any(test, cocycle_distance_bench)))]
#[derive(Default)]
struct Diagnostics {}

const NONE: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(any(test, cocycle_distance_bench), derive(Default))]
pub(crate) enum Route {
    #[default]
    #[cfg(any(test, cocycle_distance_bench))]
    Empty,
    #[cfg(any(test, cocycle_distance_bench))]
    Identity,
    Multiplicity,
    #[cfg(any(test, cocycle_distance_bench))]
    NoCross,
    MandatorySparse,
    Quickselect,
    Refinement,
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "bottleneck workspace",
    }
}

fn size_overflow() -> Error {
    Error::SizeOverflow {
        operation: "bottleneck workspace shape",
    }
}

fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<()> {
    values.try_reserve(additional).map_err(|_| allocation())
}

fn filled<T: Clone>(len: usize, value: T) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(len).map_err(|_| allocation())?;
    result.resize(len, value);
    Ok(result)
}

fn push<T>(values: &mut Vec<T>, value: T) -> Result<()> {
    reserve(values, 1)?;
    values.push(value);
    Ok(())
}

#[cfg(any(test, cocycle_distance_bench))]
fn bytes<T>(values: &Vec<T>) -> usize {
    values.capacity().saturating_mul(std::mem::size_of::<T>())
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).abs().max((a[1] - b[1]).abs())
}

fn diagonal(point: [f64; 2]) -> Result<f64> {
    // The diagonal is a real-valued locus, not a rounded representable midpoint.
    // In particular an adjacent-float lifetime has half an ULP of diagonal cost.
    // This intentionally fixes Topp C0's midpoint-rounding boundary discrepancy.
    let lifetime = point[1] - point[0];
    let result = if lifetime.is_finite() {
        lifetime * 0.5
    } else {
        point[1] * 0.5 - point[0] * 0.5
    };
    if result == 0.0 {
        return Err(Error::NumericalFailure {
            context: "bottleneck positive diagonal cost underflow",
        });
    }
    Ok(result)
}

struct Prepared<'a> {
    points: &'a [[f64; 2]],
    diagonals: Vec<f64>,
    order: Vec<usize>,
    representatives: Vec<usize>,
    multiplicities: Vec<usize>,
    max_diagonal: f64,
}

impl<'a> Prepared<'a> {
    fn new(points: &'a [[f64; 2]]) -> Result<Self> {
        let mut diagonals = filled(points.len(), 0.0)?;
        let mut order = filled(points.len(), 0)?;
        let mut max_diagonal: f64 = 0.0;
        for (index, &point) in points.iter().enumerate() {
            diagonals[index] = diagonal(point)?;
            max_diagonal = max_diagonal.max(diagonals[index]);
            order[index] = index;
        }
        // Equal-coordinate groups remain contiguous; the final index tie break
        // makes ordering deterministic without an allocating stable sort.
        order.sort_unstable_by(|&a, &b| {
            points[a][0]
                .partial_cmp(&points[b][0])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| points[a][1].total_cmp(&points[b][1]))
                .then(a.cmp(&b))
        });
        let mut representatives = Vec::new();
        let mut multiplicities = Vec::new();
        reserve(&mut representatives, points.len())?;
        reserve(&mut multiplicities, points.len())?;
        for &index in &order {
            if representatives
                .last()
                .is_some_and(|&old| points[old] == points[index])
            {
                // A group cannot exceed the borrowed slice's length.
                if let Some(count) = multiplicities.last_mut() {
                    *count += 1;
                }
            } else {
                representatives.push(index);
                multiplicities.push(1);
            }
        }
        Ok(Self {
            points,
            diagonals,
            order,
            representatives,
            multiplicities,
            max_diagonal,
        })
    }

    fn window(&self, birth: f64, radius: f64) -> std::ops::Range<usize> {
        let lower = (birth - radius).next_down();
        let upper = (birth + radius).next_up();
        let begin = self
            .order
            .partition_point(|&index| self.points[index][0] < lower);
        let end = self
            .order
            .partition_point(|&index| self.points[index][0] <= upper);
        begin..end
    }

    #[cfg(any(test, cocycle_distance_bench))]
    fn bytes(&self) -> usize {
        bytes(&self.diagonals)
            .saturating_add(bytes(&self.order))
            .saturating_add(bytes(&self.representatives))
            .saturating_add(bytes(&self.multiplicities))
    }
}

struct Pair<'a, 'p> {
    first: &'p Prepared<'a>,
    second: &'p Prepared<'a>,
    size: usize,
    dense: Vec<f64>,
}

impl<'a, 'p> Pair<'a, 'p> {
    fn new(first: &'p Prepared<'a>, second: &'p Prepared<'a>, dense: bool) -> Result<Self> {
        let size = first
            .points
            .len()
            .checked_add(second.points.len())
            .ok_or_else(size_overflow)?;
        let mut result = Self {
            first,
            second,
            size,
            dense: Vec::new(),
        };
        if dense {
            let count = first
                .points
                .len()
                .checked_mul(second.points.len())
                .ok_or_else(size_overflow)?;
            reserve(&mut result.dense, count)?;
            for &a in first.points {
                for &b in second.points {
                    result.dense.push(cross(a, b));
                }
            }
        }
        Ok(result)
    }

    fn cross(&self, left: usize, right: usize) -> f64 {
        if self.dense.is_empty() {
            cross(self.first.points[left], self.second.points[right])
        } else {
            self.dense[left * self.second.points.len() + right]
        }
    }

    fn upper(&self) -> f64 {
        self.first.max_diagonal.max(self.second.max_diagonal)
    }

    fn lower(&self) -> f64 {
        (self.first.max_diagonal - self.second.max_diagonal).abs()
    }

    fn allowed(&self, left: usize, right: usize, radius: f64) -> bool {
        let n = self.first.points.len();
        let m = self.second.points.len();
        if left < n {
            if right < m {
                self.cross(left, right) <= radius
            } else {
                right == m + left && self.first.diagonals[left] <= radius
            }
        } else if right < m {
            right == left - n && self.second.diagonals[right] <= radius
        } else {
            true
        }
    }

    #[cfg(any(test, cocycle_distance_bench))]
    fn bytes(&self) -> usize {
        self.first
            .bytes()
            .saturating_add(self.second.bytes())
            .saturating_add(bytes(&self.dense))
    }
}

fn prefer_multiplicity(first: &Prepared<'_>, second: &Prepared<'_>, total: usize) -> bool {
    if total < 128 {
        return false;
    }
    let divisor = if total >= 1024 {
        4
    } else if total >= 512 {
        5
    } else {
        6
    };
    first.representatives.len() + second.representatives.len() <= total.div_ceil(divisor)
}

fn identical(first: &Prepared<'_>, second: &Prepared<'_>) -> bool {
    first.multiplicities == second.multiplicities
        && first
            .representatives
            .iter()
            .zip(&second.representatives)
            .all(|(&a, &b)| first.points[a] == second.points[b])
}

fn prefer_mandatory(first: &Prepared<'_>, second: &Prepared<'_>, total: usize, upper: f64) -> bool {
    if total < 256 || upper <= 0.0 {
        return false;
    }
    let small = first
        .diagonals
        .iter()
        .chain(&second.diagonals)
        .filter(|&&value| value <= upper * 0.05)
        .count();
    if small >= total.div_ceil(2) {
        return true;
    }
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for prepared in [first, second] {
        if let (Some(&a), Some(&b)) = (prepared.order.first(), prepared.order.last()) {
            low = low.min(prepared.points[a][0]);
            high = high.max(prepared.points[b][0]);
        }
    }
    let span = high - low;
    let cutoff = if span.is_finite() {
        span * 0.02
    } else {
        high * 0.02 - low * 0.02
    };
    span > 0.0 && upper <= cutoff
}

pub(crate) fn distance(first: &[[f64; 2]], second: &[[f64; 2]]) -> Result<f64> {
    solve(
        first,
        second,
        Options::default(),
        &mut Diagnostics::default(),
    )
}

#[cfg(any(test, cocycle_distance_bench))]
pub(crate) fn distance_with_options(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    _options: Options,
    _stats: &mut Diagnostics,
) -> Result<f64> {
    solve(first, second, _options, _stats)
}

fn solve(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    _options: Options,
    _stats: &mut Diagnostics,
) -> Result<f64> {
    record! { *_stats = Diagnostics::default(); }
    let first = Prepared::new(first)?;
    let second = Prepared::new(second)?;
    let total = first
        .points
        .len()
        .checked_add(second.points.len())
        .ok_or_else(size_overflow)?;
    record! { _stats.workspace(first.bytes().saturating_add(second.bytes())); }
    if first.points.is_empty() || second.points.is_empty() {
        return Ok(first.max_diagonal.max(second.max_diagonal));
    }
    let duplicates = prefer_multiplicity(&first, &second, total);
    if duplicates && identical(&first, &second) {
        record! { _stats.route = Route::Identity; }
        return Ok(0.0);
    }
    let (first, second) = if total >= 128 && first.points.len() > second.points.len() {
        (&second, &first)
    } else {
        (&first, &second)
    };
    let upper = first.max_diagonal.max(second.max_diagonal);
    let mut route = Route::Quickselect;
    if experiment!(_options.search == Search::Adaptive, true) {
        if duplicates {
            route = Route::Multiplicity;
        } else if total >= 128 {
            let mut window_pairs = 0_usize;
            for &index in &first.order {
                window_pairs = window_pairs
                    .checked_add(second.window(first.points[index][0], upper).len())
                    .ok_or_else(size_overflow)?;
            }
            if window_pairs == 0 {
                record! { _stats.route = Route::NoCross; }
                return Ok(upper);
            }
            if prefer_mandatory(first, second, total, upper) {
                route = Route::MandatorySparse;
            } else if first.points.len().min(second.points.len()) >= 32
                && window_pairs as f64 / (first.points.len() as f64 * second.points.len() as f64)
                    >= 0.01
            {
                route = Route::Refinement;
            }
        }
    } else if experiment!(_options.search == Search::Refinement, false) {
        route = Route::Refinement;
    }
    record! { _stats.route = route; }
    let pair = Pair::new(first, second, route == Route::Quickselect)?;
    if route == Route::Refinement {
        return geometry::distance(&pair, _options, _stats);
    }
    let mut radii = candidates(
        &pair,
        route,
        experiment!(_options.clip_candidates, true),
        0.0,
        pair.upper(),
        _stats,
    )?;
    let extra_bytes = experiment!(pair.bytes().saturating_add(bytes(&radii)), 0);
    record! { _stats.workspace(extra_bytes); }
    let mut workspace = if route == Route::Quickselect {
        Some(matching::Workspace::new(pair.size)?)
    } else {
        None
    };
    let mut decide = |radius| -> Result<bool> {
        record! { _stats.threshold_decisions += 1; }
        if radius < pair.lower() {
            return Ok(false);
        }
        match route {
            Route::Multiplicity => flow::within(&pair, radius, true, _stats, extra_bytes),
            Route::MandatorySparse => flow::within(&pair, radius, false, _stats, extra_bytes),
            _ => {
                if pair.size >= 384 {
                    let optional_first = pair
                        .first
                        .diagonals
                        .iter()
                        .filter(|&&d| d <= radius)
                        .count();
                    let optional_second = pair
                        .second
                        .diagonals
                        .iter()
                        .filter(|&&d| d <= radius)
                        .count();
                    let fraction = optional_first as f64 * optional_second as f64
                        / (pair.first.points.len() as f64 * pair.second.points.len() as f64);
                    let count = pair
                        .first
                        .points
                        .len()
                        .checked_mul(pair.second.points.len())
                        .ok_or_else(size_overflow)?;
                    let samples = count.min(64);
                    let mut allowed = 0;
                    for sample in 0..samples {
                        // Quotient/remainder form avoids overflowing sample * count.
                        let flat =
                            sample * (count / samples) + sample * (count % samples) / samples;
                        record! { _stats.adjacency_checks += 1; }
                        allowed += usize::from(
                            pair.cross(
                                flat / pair.second.points.len(),
                                flat % pair.second.points.len(),
                            ) <= radius,
                        );
                    }
                    if allowed as f64 / samples as f64 <= 0.03 || fraction >= 0.75 {
                        let allocated = experiment!(
                            extra_bytes.saturating_add(
                                workspace.as_ref().map_or(0, matching::Workspace::bytes),
                            ),
                            0
                        );
                        return flow::within(&pair, radius, false, _stats, allocated);
                    }
                }
                if !experiment!(_options.reuse_scratch, true) {
                    workspace = Some(matching::Workspace::new(pair.size)?);
                }
                workspace
                    .as_mut()
                    .ok_or(Error::InternalInvariant {
                        reason: "quickselect matcher workspace is missing",
                    })?
                    .within(&pair, radius, _stats, extra_bytes)
            }
        }
    };
    #[cfg(any(test, cocycle_distance_bench))]
    if _options.search == Search::Binary {
        radii.sort_unstable_by(f64::total_cmp);
        radii.dedup();
        let mut lower = radii.partition_point(|&value| value < pair.lower());
        let mut end = radii.len();
        while lower < end {
            let middle = lower + (end - lower) / 2;
            if decide(radii[middle])? {
                end = middle;
            } else {
                lower = middle + 1;
            }
        }
        return radii.get(lower).copied().ok_or(Error::InternalInvariant {
            reason: "bottleneck has no feasible diagonal bound",
        });
    }
    radii.retain(|&value| value >= pair.lower());
    let mut remaining = radii.as_mut_slice();
    let mut best = pair.upper();
    while !remaining.is_empty() {
        let middle = remaining.len() / 2;
        let (_, pivot, _) = remaining.select_nth_unstable_by(middle, f64::total_cmp);
        let pivot = *pivot;
        if pivot >= best || decide(pivot)? {
            best = best.min(pivot);
            let mut count = 0;
            for index in 0..remaining.len() {
                if remaining[index] < pivot {
                    remaining.swap(index, count);
                    count += 1;
                }
            }
            remaining = &mut remaining[..count];
        } else {
            let mut count = 0;
            for index in 0..remaining.len() {
                if remaining[index] > pivot {
                    remaining.swap(index, count);
                    count += 1;
                }
            }
            remaining = &mut remaining[..count];
        }
    }
    Ok(best)
}

fn candidates(
    pair: &Pair<'_, '_>,
    route: Route,
    clip: bool,
    lower: f64,
    upper: f64,
    _stats: &mut Diagnostics,
) -> Result<Vec<f64>> {
    let grouped = route == Route::Multiplicity;
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
    let mut values = Vec::new();
    push(&mut values, 0.0)?;
    for (prepared, indices) in [(pair.first, first_indices), (pair.second, second_indices)] {
        for &index in indices {
            let value = prepared.diagonals[index];
            if !clip || (value >= lower && value <= upper) {
                push(&mut values, value)?;
            }
        }
    }
    for &left in first_indices {
        if clip && matches!(route, Route::MandatorySparse | Route::Refinement) {
            for position in pair.second.window(pair.first.points[left][0], upper) {
                let right = pair.second.order[position];
                record! { _stats.adjacency_checks += 1; }
                let value = pair.cross(left, right);
                if value > lower && value <= upper {
                    push(&mut values, value)?;
                }
            }
        } else {
            for &right in second_indices {
                let value = pair.cross(left, right);
                if !clip || (value >= lower && value <= upper) {
                    push(&mut values, value)?;
                }
            }
        }
    }
    record! { _stats.candidate_count = _stats.candidate_count.saturating_add(values.len()); }
    Ok(values)
}

#[cfg(test)]
#[path = "bottleneck/tests.rs"]
mod tests;
