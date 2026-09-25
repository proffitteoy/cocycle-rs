//! Cooperative controls shared by construction and persistence.
use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

/// Optional cooperative controls for one construction or analysis operation.
///
/// Builders share one budget across preparation, expansion, reduction and requested
/// representatives. Counted units include distance reads, metric inequalities,
/// sampling updates, simplex/cofacet candidates, stored incidence terms and
/// reduction steps. Repeated visits count again; counts are not timings or bytes.
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
}

pub(crate) struct WorkBudget<'a> {
    limits: Execution<'a>,
    used: u64,
}
impl<'a> WorkBudget<'a> {
    pub(crate) fn new(limits: &Execution<'a>) -> Result<Self> {
        let result = Self {
            limits: *limits,
            used: 0,
        };
        result.check()?;
        Ok(result)
    }
    pub(crate) fn check(&self) -> Result<()> {
        if self
            .limits
            .cancellation
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            return Err(Error::Cancelled);
        }
        Ok(())
    }
    pub(crate) fn step(&mut self) -> Result<()> {
        self.check()?;
        if let Some(limit) = self.limits.max_work {
            if self.used >= limit {
                return Err(Error::WorkLimitExceeded { limit });
            }
            self.used += 1;
        }
        Ok(())
    }
}
