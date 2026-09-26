//! Selected, validated boundary columns and their local filtration order.
use crate::algebra::column::Column;
use crate::{Error, Result};

/// Working input for boundary-based consumers, not a universal source contract.
/// Readers establish finite ordered values, unique IDs, face-before-coface rows,
/// codimension-one incidence and the chain condition in the chosen field.
/// Source certificates and query settings stay with the caller.
pub(in crate::persistence) struct BoundaryInput<I> {
    pub(in crate::persistence) cells: Vec<I>,
    pub(in crate::persistence) dimensions: Vec<usize>,
    pub(in crate::persistence) values: Vec<f64>,
    pub(in crate::persistence) columns: Vec<Column<usize>>,
}

impl<I> BoundaryInput<I> {
    pub(in crate::persistence) fn new() -> Self {
        Self {
            cells: Vec::new(),
            dimensions: Vec::new(),
            values: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Reserve before appending, keeping cell metadata and columns in lockstep.
    pub(in crate::persistence) fn push(
        &mut self,
        cell: I,
        dimension: usize,
        value: f64,
        column: Column<usize>,
    ) -> Result<()> {
        self.cells.try_reserve(1).map_err(|_| allocation())?;
        self.dimensions.try_reserve(1).map_err(|_| allocation())?;
        self.values.try_reserve(1).map_err(|_| allocation())?;
        self.columns.try_reserve(1).map_err(|_| allocation())?;
        self.cells.push(cell);
        self.dimensions.push(dimension);
        self.values.push(value);
        self.columns.push(column);
        Ok(())
    }
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "filtered boundary input",
    }
}
