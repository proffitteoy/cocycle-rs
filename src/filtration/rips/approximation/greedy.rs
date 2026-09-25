//! Deterministic exhaustive farthest-point sampling; no triangle pruning.
use crate::{Error, Result};

pub(super) struct Greedy {
    pub(super) order: Vec<usize>,
    pub(super) radii: Vec<Option<f64>>,
}
pub(super) fn permutation(
    n: usize,
    start: Option<usize>,
    distance: &mut impl FnMut(usize, usize) -> Result<f64>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<Greedy> {
    if start.is_some_and(|v| v >= n) {
        return Err(Error::InvalidParameter {
            parameter: "start_vertex",
            reason: "outside input vertices",
        });
    }
    let mut order = Vec::new();
    let mut radii = Vec::new();
    let mut nearest = Vec::new();
    let mut selected = Vec::new();
    order
        .try_reserve_exact(n)
        .map_err(|_| super::allocation())?;
    radii
        .try_reserve_exact(n)
        .map_err(|_| super::allocation())?;
    nearest
        .try_reserve_exact(n)
        .map_err(|_| super::allocation())?;
    selected
        .try_reserve_exact(n)
        .map_err(|_| super::allocation())?;
    nearest.resize(n, f64::INFINITY);
    selected.resize(n, false);
    if n == 0 {
        return Ok(Greedy { order, radii });
    }
    let mut next = start.unwrap_or(0);
    for i in 0..n {
        checkpoint()?;
        order.push(next);
        radii.push(if i == 0 { None } else { Some(nearest[next]) });
        selected[next] = true;
        let mut farthest = None;
        for v in 0..n {
            checkpoint()?;
            if selected[v] {
                continue;
            }
            nearest[v] = nearest[v].min(distance(next, v)?);
            if farthest.is_none_or(|old| nearest[v] > nearest[old]) {
                farthest = Some(v);
            }
        }
        if let Some(v) = farthest {
            next = v;
        }
    }
    Ok(Greedy { order, radii })
}
