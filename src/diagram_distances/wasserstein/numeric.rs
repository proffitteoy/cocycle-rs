//! Checked scaling and reconstruction from original matching costs.
//! Adapted from Topp; the parent module retains the MIT attribution.

use super::{Metric, allocation, buffer, sum_size};
use crate::{Error, Result};

pub(super) fn numerical() -> Error {
    Error::NumericalFailure {
        context: "Wasserstein distance",
    }
}

pub(super) fn finite(value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numerical())
    }
}

pub(super) fn restore_scale(value: f64, scale: f64) -> Result<f64> {
    let restored = finite(value * scale)?;
    if value > 0.0 && restored == 0.0 {
        return Err(numerical());
    }
    Ok(restored)
}

#[derive(Clone, Copy)]
pub(super) struct Point {
    pub(super) coordinates: [f64; 2],
    pub(super) midpoint: f64,
    pub(super) half: f64,
}

pub(super) fn prepare(input: &[[f64; 2]], scale: f64) -> Result<Vec<Point>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(input.len())
        .map_err(|_| allocation())?;
    for &[birth, death] in input {
        let b = birth / scale;
        let d = death / scale;
        // Scaling by a power of two must be reversible: no implicit quantization
        // or collapsed intervals are accepted when the exponent range is wide.
        if !b.is_finite() || !d.is_finite() || b * scale != birth || d * scale != death || b >= d {
            return Err(numerical());
        }
        let half = (d - b) * 0.5;
        if !half.is_finite() || half <= 0.0 {
            return Err(numerical());
        }
        result.push(Point {
            coordinates: [b, d],
            midpoint: b * 0.5 + d * 0.5,
            half,
        });
    }
    Ok(result)
}

pub(super) fn power_scale(first: &[[f64; 2]], second: &[[f64; 2]]) -> f64 {
    let largest = first
        .iter()
        .chain(second)
        .flatten()
        .fold(0.0_f64, |a, b| a.max(b.abs()));
    if largest == 0.0 || (2.0_f64.powi(-200)..=2.0_f64.powi(200)).contains(&largest) {
        return 1.0;
    }
    let exponent = ((largest.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    // A normal scaling factor also scales the smallest subnormal up to 2^-52.
    2.0_f64.powi(exponent.max(-1022))
}

pub(super) fn square(value: f64) -> Result<f64> {
    let squared = finite(value * value)?;
    if value != 0.0 && squared == 0.0 {
        return Err(numerical());
    }
    Ok(squared)
}

pub(super) fn diagonal_power(point: Point, metric: Metric) -> Result<f64> {
    match metric {
        Metric::W1 => Ok(point.half),
        Metric::W2 => finite(2.0 * square(point.half)?),
    }
}

pub(super) fn cross_power(first: Point, second: Point, metric: Metric) -> Result<f64> {
    let b = finite(first.coordinates[0] - second.coordinates[0])?.abs();
    let d = finite(first.coordinates[1] - second.coordinates[1])?.abs();
    match metric {
        Metric::W1 => Ok(b.max(d)),
        Metric::W2 => finite(square(b)? + square(d)?),
    }
}

pub(super) fn saving(first: Point, second: Point, metric: Metric) -> Result<(f64, bool)> {
    // Rounded midpoints are only a search index, never the cost authority:
    // adjacent endpoint values need not have a representable midpoint.
    let baseline = finite(diagonal_power(first, metric)? + diagonal_power(second, metric)?)?;
    let cross = cross_power(first, second, metric)?;
    // In this regime, subtracting a small cross cost from the diagonal baseline
    // cannot retain enough significant bits to rank almost equal matchings.
    // Reconstructing the final cost is insufficient: choose the matching itself
    // using original costs. This is a numerical guard, not a changed tolerance
    // or a heuristic substitute for the normal saving-graph solver.
    let direct_cost_required = cross > 0.0 && cross <= 64.0 * f64::EPSILON * baseline;
    Ok((finite(baseline - cross)?, direct_cost_required))
}

pub(super) fn accumulate(sum: &mut f64, correction: &mut f64, cost: f64) -> Result<()> {
    // Kahan accumulation of original nonnegative costs avoids baseline-minus-
    // savings cancellation, especially for almost equal diagrams in W2.
    let adjusted = cost - *correction;
    let next = finite(*sum + adjusted)?;
    *correction = (next - *sum) - adjusted;
    *sum = next;
    Ok(())
}

pub(super) fn from_flows(
    first: &[Point],
    second: &[Point],
    row_capacity: &[usize],
    column_capacity: &[usize],
    flows: &[(usize, usize, usize)],
    metric: Metric,
) -> Result<f64> {
    let mut row_used = buffer(first.len(), 0)?;
    let mut col_used = buffer(second.len(), 0)?;
    let mut total = 0.0;
    let mut correction = 0.0;
    for &(row, column, count) in flows {
        row_used[row] = sum_size(row_used[row], count)?;
        col_used[column] = sum_size(col_used[column], count)?;
        accumulate(
            &mut total,
            &mut correction,
            finite(cross_power(first[row], second[column], metric)? * count as f64)?,
        )?;
    }
    for (row, &point) in first.iter().enumerate() {
        let remaining =
            row_capacity[row]
                .checked_sub(row_used[row])
                .ok_or(Error::InternalInvariant {
                    reason: "Wasserstein row capacity exceeded",
                })?;
        if remaining > 0 {
            accumulate(
                &mut total,
                &mut correction,
                finite(diagonal_power(point, metric)? * remaining as f64)?,
            )?;
        }
    }
    for (column, &point) in second.iter().enumerate() {
        let remaining = column_capacity[column]
            .checked_sub(col_used[column])
            .ok_or(Error::InternalInvariant {
                reason: "Wasserstein column capacity exceeded",
            })?;
        if remaining > 0 {
            accumulate(
                &mut total,
                &mut correction,
                finite(diagonal_power(point, metric)? * remaining as f64)?,
            )?;
        }
    }
    Ok(if metric == Metric::W2 {
        total.sqrt()
    } else {
        total
    })
}

pub(super) fn from_matching(
    first: &[Point],
    second: &[Point],
    matching: Vec<Option<usize>>,
    metric: Metric,
) -> Result<f64> {
    let mut flows = Vec::new();
    flows
        .try_reserve_exact(first.len().min(second.len()))
        .map_err(|_| allocation())?;
    for (row, column) in matching.into_iter().enumerate() {
        if let Some(column) = column {
            flows.push((row, column, 1));
        }
    }
    from_flows(
        first,
        second,
        &buffer(first.len(), 1)?,
        &buffer(second.len(), 1)?,
        &flows,
        metric,
    )
}
