//! Cooperative controls shared by construction and analysis operations.
use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

/// Optional cooperative controls for one construction or analysis operation.
///
/// Builders share one budget across preparation, expansion, reduction and requested
/// representatives. Counted units include distance reads, metric inequalities,
/// sampling updates, simplex/cofacet candidates, stored incidence terms and
/// reduction steps. Repeated visits count again; counts are not timings or bytes.
/// Diagram-distance `_with` functions share one budget across validation,
/// extraction, preparation, candidate generation, matching and accumulation.
/// Distance work is charged by interval, candidate/graph row, search visit or
/// batch. Counts depend on the selected algorithm and are not comparable across
/// metrics or versions. These controls impose neither a memory quota nor a
/// thread limit.
/// Cancellation is checked around sorting, allocation, heap construction and user
/// callbacks, which cannot be interrupted internally. Previously validated input is outside the
/// operation budget. Each terminal starts fresh; no partial result is returned.
/// Legacy persistence functions retain their documented persistence-only scope.
#[derive(Clone, Copy, Debug, Default)]
pub struct Execution<'a> {
    max_work: Option<u64>,
    cancellation: Option<&'a AtomicBool>,
}
impl<'a> Execution<'a> {
    /// Create cooperative controls. A zero limit permits no counted work units.
    /// Set the optional flag to true to request cancellation; the flag is never reset.
    pub fn new(max_work: Option<u64>, cancellation: Option<&'a AtomicBool>) -> Self {
        Self {
            max_work,
            cancellation,
        }
    }
    /// Limit counted work; zero permits no counted units.
    pub fn max_work(mut self, limit: u64) -> Self {
        self.max_work = Some(limit);
        self
    }
    /// Borrow a cancellation flag without resetting it.
    pub fn cancellation(mut self, flag: &'a AtomicBool) -> Self {
        self.cancellation = Some(flag);
        self
    }

    pub(crate) fn is_unlimited(&self) -> bool {
        self.max_work.is_none() && self.cancellation.is_none()
    }
}

// Distance kernels specialize the unlimited case so their inner loops carry no
// runtime polling branches. Construction keeps the existing controlled default.
// Callers also guard count-only lookups: an unused charge argument can otherwise
// retain indexing checks or duplicate a search even when step_by is a no-op.
pub(crate) struct WorkBudget<'a, const CONTROLLED: bool = true> {
    limits: Execution<'a>,
    used: u64,
    #[cfg(test)]
    cancel_at: Option<u64>,
}
impl<'a> WorkBudget<'a> {
    pub(crate) fn new(limits: &Execution<'a>) -> Result<Self> {
        let result = Self {
            limits: *limits,
            used: 0,
            #[cfg(test)]
            cancel_at: None,
        };
        result.check()?;
        Ok(result)
    }
}

impl WorkBudget<'static, false> {
    pub(crate) fn unlimited() -> Self {
        Self {
            limits: Execution::default(),
            used: 0,
            #[cfg(test)]
            cancel_at: None,
        }
    }
}

impl<const CONTROLLED: bool> WorkBudget<'_, CONTROLLED> {
    #[inline(always)]
    pub(crate) fn check(&self) -> Result<()> {
        if !CONTROLLED {
            return Ok(());
        }
        // Deterministic interruption after real work, without timing-sensitive
        // test threads. This hook and its state do not exist in normal builds.
        #[cfg(test)]
        if self.cancel_at.is_some_and(|at| self.used >= at)
            && let Some(flag) = self.limits.cancellation
        {
            flag.store(true, Ordering::Relaxed);
        }
        if self
            .limits
            .cancellation
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            return Err(Error::Cancelled);
        }
        Ok(())
    }
    #[inline(always)]
    pub(crate) fn step(&mut self) -> Result<()> {
        self.step_by(1)
    }

    /// Charge a batch before doing its work, without overflowing the counter.
    #[inline(always)]
    pub(crate) fn step_by(&mut self, count: usize) -> Result<()> {
        if !CONTROLLED {
            return Ok(());
        }
        self.check()?;
        if let Some(limit) = self.limits.max_work {
            let count = u64::try_from(count).map_err(|_| Error::WorkLimitExceeded { limit })?;
            if count > limit - self.used {
                return Err(Error::WorkLimitExceeded { limit });
            }
            self.used += count;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn used(&self) -> u64 {
        self.used
    }

    #[cfg(test)]
    pub(crate) fn cancel_at_work(&mut self, work: u64) {
        self.cancel_at = Some(work);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_charging_is_atomic_and_does_not_wrap() {
        let mut budget = WorkBudget::new(&Execution::default().max_work(3)).unwrap();
        budget.step_by(2).unwrap();
        assert_eq!(
            budget.step_by(2),
            Err(Error::WorkLimitExceeded { limit: 3 })
        );
        assert_eq!(budget.used, 2);
        budget.step().unwrap();
        budget.step_by(0).unwrap();
        assert_eq!(budget.step(), Err(Error::WorkLimitExceeded { limit: 3 }));
        let mut budget = WorkBudget::new(&Execution::default().max_work(u64::MAX)).unwrap();
        budget.used = u64::MAX - 1;
        budget.step().unwrap();
        assert_eq!(
            budget.step(),
            Err(Error::WorkLimitExceeded { limit: u64::MAX })
        );
    }
}
