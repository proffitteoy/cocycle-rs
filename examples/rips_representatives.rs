//! Compute an F3 cycle and its dual cocycle at a specified Rips scale.
use cocycle::algebra::PrimeField;
use cocycle::diagram::RepresentativeKind;
use cocycle::filtration::RipsBuilder;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{PersistenceExt, RepresentativeRequest, RepresentativeSelection};

fn main() -> cocycle::Result<()> {
    // Four-cycle edges enter at 1; diagonals enter at 2 and fill the cycle.
    let values = [1., 2., 1., 1., 2., 1.];
    let matrix = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle)?;
    let requests = [RepresentativeRequest::new(
        1,
        1.,
        RepresentativeSelection::Both,
    )?];
    let result = RipsBuilder::from_distance_matrix(matrix)
        .persistence()
        .field(PrimeField::new(3)?)
        .representatives(&requests)
        .compute()?;
    let representatives = result.representatives().unwrap();
    assert_eq!(representatives.len(), 2);
    assert_eq!(result.context().characteristic(), 3);
    for representative in representatives {
        let interval = result.diagram().intervals()[representative.interval_index()];
        let kind = match representative.kind() {
            RepresentativeKind::Cycle => "cycle",
            RepresentativeKind::Cocycle => "cocycle",
        };
        println!(
            "{kind} at {} for interval {:?} over F{}:",
            representative.scale(),
            interval,
            representative.characteristic()
        );
        for term in representative.terms() {
            println!("  {:?}: {}", term.vertices(), term.coefficient());
        }
    }
    Ok(())
}
