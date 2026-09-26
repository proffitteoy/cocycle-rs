//! M1 public contracts. These tests do not compute persistent homology.

use cocycle::Error;
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::geometry::{DissimilarityView, PointCloudView};
use cocycle::persistence::RipsOptions;

fn finite(dimension: usize, birth: f64, death: f64) -> PersistenceInterval {
    PersistenceInterval::new(dimension, birth, IntervalEnd::Finite(death)).unwrap()
}

fn censored(dimension: usize, birth: f64, through: f64) -> PersistenceInterval {
    PersistenceInterval::new(dimension, birth, IntervalEnd::RightCensored { through }).unwrap()
}

fn essential(dimension: usize, birth: f64) -> PersistenceInterval {
    PersistenceInterval::new(dimension, birth, IntervalEnd::Essential).unwrap()
}

#[test]
fn row_major_points_borrow_and_preserve_duplicates() {
    let coordinates = [-0.0, 2.0, -0.0, 2.0];
    let points = PointCloudView::new(&coordinates, 2, 2).unwrap();
    assert_eq!(points.len(), 2);
    assert!(!points.is_empty());
    assert_eq!(points.dimension(), 2);
    assert_eq!(points.coordinates().as_ptr(), coordinates.as_ptr());
    assert_eq!(points.point(0), Some(&coordinates[..2]));
    assert_eq!(points.point(1), Some(&coordinates[2..]));
    assert_eq!(points.point(2), None);
    assert_eq!(points.point(usize::MAX), None);
    assert_eq!(points.coordinates()[0].to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn empty_cloud_still_requires_positive_coordinate_dimension() {
    let points = PointCloudView::new(&[], 0, 3).unwrap();
    assert!(points.is_empty());
    assert_eq!(points.dimension(), 3);
    assert_eq!(points.point(0), None);
    assert!(matches!(
        PointCloudView::new(&[], 0, 0),
        Err(Error::InvalidParameter { .. })
    ));
}

#[test]
fn point_shape_mismatch_is_distinct_from_overflow() {
    assert!(matches!(
        PointCloudView::new(&[1.0], 2, 2),
        Err(Error::ShapeMismatch {
            expected: 4,
            actual: 1,
            ..
        })
    ));
    assert!(matches!(
        PointCloudView::new(&[], usize::MAX, 2),
        Err(Error::SizeOverflow { .. })
    ));
}

#[test]
fn coordinates_reject_nonfinite_values_with_their_position() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            PointCloudView::new(&[0.0, value], 1, 2).unwrap_err(),
            Error::NonFiniteValue {
                field: "coordinates",
                index: Some(1)
            }
        );
    }
    assert!(PointCloudView::new(&[-f64::MAX, f64::MAX], 1, 2).is_ok());
}

#[test]
fn condensed_layout_matches_a_symmetric_matrix() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let distances = DissimilarityView::new(&values, 4).unwrap();
    let expected = [
        [0.0, 1.0, 2.0, 4.0],
        [1.0, 0.0, 3.0, 5.0],
        [2.0, 3.0, 0.0, 6.0],
        [4.0, 5.0, 6.0, 0.0],
    ];
    for (i, row) in expected.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            assert_eq!(distances.get(i, j), Some(value));
        }
    }
    assert_eq!(distances.len(), 4);
    assert!(!distances.is_empty());
    assert_eq!(distances.values().as_ptr(), values.as_ptr());
    assert_eq!(distances.diameter(), 6.0);
    for (i, j) in [(4, 0), (0, 4), (4, 4), (usize::MAX, usize::MAX)] {
        assert_eq!(distances.get(i, j), None);
    }
}

#[test]
fn explicit_vertex_count_distinguishes_empty_and_singleton() {
    let empty = DissimilarityView::new(&[], 0).unwrap();
    let singleton = DissimilarityView::new(&[], 1).unwrap();
    assert!(empty.is_empty());
    assert!(!singleton.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(singleton.len(), 1);
    assert_eq!(empty.get(0, 0), None);
    assert_eq!(singleton.get(0, 0), Some(0.0));
    assert_eq!(empty.diameter().to_bits(), 0.0_f64.to_bits());
    assert_eq!(singleton.diameter().to_bits(), 0.0_f64.to_bits());
}

#[test]
fn dissimilarities_accept_zero_pairs_and_nonmetric_data() {
    // d(2,0) = 4 > d(2,1) + d(1,0); no metric certification is implied.
    assert!(DissimilarityView::new(&[1.0, 4.0, 1.0], 3).is_ok());
    let raw = [-0.0];
    let distances = DissimilarityView::new(&raw, 2).unwrap();
    assert_eq!(distances.len(), 2);
    assert_eq!(distances.values()[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(distances.get(0, 1).unwrap().to_bits(), 0.0_f64.to_bits());
    assert_eq!(distances.diameter().to_bits(), 0.0_f64.to_bits());
}

#[test]
fn dissimilarities_reject_nonfinite_and_negative_entries() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            DissimilarityView::new(&[1.0, value, 2.0], 3),
            Err(Error::NonFiniteValue { index: Some(1), .. })
        ));
    }
    assert!(matches!(
        DissimilarityView::new(&[-f64::MIN_POSITIVE], 2),
        Err(Error::NegativeValue { index: Some(0), .. })
    ));
}

#[test]
fn condensed_shape_validation_divides_before_multiplying() {
    // n*(n-1) overflows usize, but n choose 2 fits, on both 32- and 64-bit targets.
    let n = (1_usize << (usize::BITS / 2)) + 1;
    let expected = ((n as u128) * ((n - 1) as u128) / 2) as usize;
    assert_eq!(
        DissimilarityView::new(&[], n).unwrap_err(),
        Error::ShapeMismatch {
            input: "dissimilarities",
            expected,
            actual: 0
        }
    );
    assert!(matches!(
        DissimilarityView::new(&[], usize::MAX),
        Err(Error::SizeOverflow { .. })
    ));
    assert!(matches!(
        DissimilarityView::new(&[1.0, 2.0], 3),
        Err(Error::ShapeMismatch {
            expected: 3,
            actual: 2,
            ..
        })
    ));
}

#[test]
fn rips_options_limit_computation_dimensions_without_limiting_diagrams() {
    for dimension in [0, 1] {
        let options = RipsOptions::new(dimension, None).unwrap();
        assert_eq!(options.max_dimension(), dimension);
        assert_eq!(options.max_edge(), None);
    }
    for dimension in [2, usize::MAX] {
        assert_eq!(
            RipsOptions::new(dimension, None).unwrap_err(),
            Error::UnsupportedDimension {
                requested: dimension,
                max_supported: 1
            }
        );
    }
    assert_eq!(RipsOptions::default(), RipsOptions::new(1, None).unwrap());
}

#[test]
fn rips_cutoff_must_be_finite_and_nonnegative() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            RipsOptions::new(1, Some(value)),
            Err(Error::NonFiniteValue { .. })
        ));
    }
    assert!(matches!(
        RipsOptions::new(0, Some(-1.0)),
        Err(Error::NegativeValue { .. })
    ));
    let options = RipsOptions::new(1, Some(-0.0)).unwrap();
    assert_eq!(options.max_edge().unwrap().to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        RipsOptions::new(1, Some(f64::MAX)).unwrap().max_edge(),
        Some(f64::MAX)
    );
}

#[test]
fn generic_intervals_allow_signed_scales_and_high_dimensions() {
    let interval = finite(8, -3.0, -1.0);
    assert_eq!(interval.dimension(), 8);
    assert_eq!(interval.birth(), -3.0);
    assert_eq!(interval.end(), IntervalEnd::Finite(-1.0));
    let diagram = PersistenceDiagram::new(
        usize::MAX,
        Coverage::Complete,
        vec![interval, essential(usize::MAX, -2.0)],
    )
    .unwrap();
    assert_eq!(
        diagram.intervals_in_dimension(usize::MAX).unwrap().count(),
        1
    );
}

#[test]
fn finite_intervals_reject_zero_or_negative_lifetime() {
    for (birth, death) in [(1.0, 1.0), (1.0, 0.0), (-0.0, 0.0)] {
        assert!(matches!(
            PersistenceInterval::new(0, birth, IntervalEnd::Finite(death)),
            Err(Error::InvalidInterval { .. })
        ));
    }
    // Adjacent representable values are distinct; no epsilon is used.
    let next = f64::from_bits(1.0_f64.to_bits() + 1);
    assert!(PersistenceInterval::new(0, 1.0, IntervalEnd::Finite(next)).is_ok());
}

#[test]
fn censoring_includes_birth_at_the_cutoff() {
    assert!(PersistenceInterval::new(1, 2.0, IntervalEnd::RightCensored { through: 2.0 }).is_ok());
    assert!(matches!(
        PersistenceInterval::new(1, 2.0, IntervalEnd::RightCensored { through: 1.0 }),
        Err(Error::InvalidInterval { .. })
    ));
}

#[test]
fn all_interval_scale_fields_reject_nonfinite_values() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for result in [
            PersistenceInterval::new(0, value, IntervalEnd::Essential),
            PersistenceInterval::new(0, 0.0, IntervalEnd::Finite(value)),
            PersistenceInterval::new(0, 0.0, IntervalEnd::RightCensored { through: value }),
        ] {
            assert!(matches!(result, Err(Error::NonFiniteValue { .. })));
        }
    }
}

#[test]
fn finite_endpoints_do_not_require_a_representable_lifetime() {
    // A later descriptor must detect overflow; the endpoints themselves are valid.
    let interval = finite(0, -f64::MAX, f64::MAX);
    assert_eq!(interval.birth(), -f64::MAX);
    assert_eq!(interval.end(), IntervalEnd::Finite(f64::MAX));
}

#[test]
fn interval_and_coverage_zeros_are_canonical() {
    assert_eq!(essential(0, -0.0).birth().to_bits(), 0.0_f64.to_bits());
    let IntervalEnd::Finite(death) = finite(0, -1.0, -0.0).end() else {
        panic!()
    };
    assert_eq!(death.to_bits(), 0.0_f64.to_bits());
    let interval = censored(0, -0.0, -0.0);
    let IntervalEnd::RightCensored { through } = interval.end() else {
        panic!()
    };
    assert_eq!(through.to_bits(), 0.0_f64.to_bits());
    let diagram = PersistenceDiagram::new(0, Coverage::Through(-0.0), vec![interval]).unwrap();
    let Coverage::Through(cutoff) = diagram.coverage() else {
        panic!()
    };
    assert_eq!(cutoff.to_bits(), 0.0_f64.to_bits());
}

#[test]
fn empty_diagrams_distinguish_absent_intervals_from_uncomputed_dimensions() {
    let h0 = PersistenceDiagram::new(0, Coverage::Complete, vec![]).unwrap();
    assert_eq!(h0.max_dimension(), 0);
    assert_eq!(h0.intervals_in_dimension(0).unwrap().count(), 0);
    assert!(matches!(
        h0.intervals_in_dimension(1),
        Err(Error::DimensionNotComputed { .. })
    ));
    let h1 = PersistenceDiagram::new(1, Coverage::Through(2.0), vec![]).unwrap();
    assert_eq!(h1.intervals_in_dimension(1).unwrap().count(), 0);
    assert_eq!(h1.coverage(), Coverage::Through(2.0));
}

#[test]
fn computed_dimensions_normalize_sets_and_handle_the_largest_dimension() {
    use cocycle::diagram::ComputedDimensions;
    use std::collections::BTreeSet;
    assert!(ComputedDimensions::new(vec![]).is_err());
    for mask in 1_u32..256 {
        let expected: BTreeSet<_> = (0..8).filter(|d| mask & (1 << d) != 0).collect();
        let input: Vec<_> = expected.iter().rev().flat_map(|&d| [d, d]).collect();
        let dimensions = ComputedDimensions::new(input).unwrap();
        assert_eq!(dimensions.iter().collect::<BTreeSet<_>>(), expected);
        assert_eq!(dimensions.iter().count(), expected.len());
        assert_eq!(
            dimensions.iter().rev().collect::<Vec<_>>(),
            expected.iter().rev().copied().collect::<Vec<_>>()
        );
        assert_eq!(dimensions.max(), *expected.last().unwrap());
        for d in 0..10 {
            assert_eq!(dimensions.contains(d), expected.contains(&d));
        }
    }
    assert_eq!(
        ComputedDimensions::new(vec![2, 0, 1, 2]).unwrap(),
        ComputedDimensions::through(2)
    );
    let largest = ComputedDimensions::new(vec![usize::MAX, 0, usize::MAX - 1, usize::MAX]).unwrap();
    assert_eq!(
        largest.iter().collect::<Vec<_>>(),
        [0, usize::MAX - 1, usize::MAX]
    );
    let all = ComputedDimensions::through(usize::MAX);
    assert!(all.contains(usize::MAX));
    assert_eq!(all.iter().next_back(), Some(usize::MAX));
    assert_eq!(all.iter().take(2).collect::<Vec<_>>(), [0, 1]);
}

#[test]
fn a_gap_is_uncomputed_even_when_below_the_maximum() {
    use cocycle::diagram::ComputedDimensions;
    let dimensions = ComputedDimensions::new(vec![1, 3]).unwrap();
    let intervals = vec![essential(1, 0.)];
    let buffer = intervals.as_ptr();
    let diagram =
        PersistenceDiagram::with_dimensions(dimensions.clone(), Coverage::Complete, intervals)
            .unwrap();
    assert_eq!(diagram.intervals().as_ptr(), buffer);
    assert_eq!(diagram.max_dimension(), 3);
    assert_eq!(diagram.computed_dimensions(), &dimensions);
    assert_eq!(diagram.intervals_in_dimension(3).unwrap().count(), 0);
    for missing in [0, 2, 4] {
        assert!(
            matches!(diagram.intervals_in_dimension(missing), Err(Error::DimensionNotComputed { requested, .. }) if requested == missing)
        );
        assert!(
            matches!(PersistenceDiagram::with_dimensions(dimensions.clone(), Coverage::Complete, vec![essential(missing, 0.)]), Err(Error::DimensionNotComputed { requested, .. }) if requested == missing)
        );
    }
    let contiguous =
        PersistenceDiagram::new(3, Coverage::Complete, diagram.intervals().to_vec()).unwrap();
    assert_ne!(diagram, contiguous);
    let empty =
        PersistenceDiagram::with_dimensions(dimensions, Coverage::Complete, vec![]).unwrap();
    assert_eq!(empty.intervals_in_dimension(1).unwrap().count(), 0);
    assert!(empty.intervals_in_dimension(0).is_err());
}

#[test]
fn generic_diagrams_do_not_assume_rips_connectivity_or_births() {
    let diagram = PersistenceDiagram::new(
        1,
        Coverage::Complete,
        vec![essential(0, -1.0), essential(0, 3.0), essential(1, 4.0)],
    )
    .unwrap();
    assert_eq!(diagram.intervals_in_dimension(0).unwrap().count(), 2);
    assert_eq!(diagram.intervals_in_dimension(1).unwrap().count(), 1);
}

#[test]
fn diagrams_reject_intervals_outside_recorded_dimensions() {
    assert_eq!(
        PersistenceDiagram::new(0, Coverage::Complete, vec![essential(1, 0.0)]).unwrap_err(),
        Error::DimensionNotComputed {
            requested: 1,
            computed_max: 0
        }
    );
}

#[test]
fn complete_diagrams_cannot_contain_censored_intervals() {
    let result = PersistenceDiagram::new(
        1,
        Coverage::Complete,
        vec![finite(1, 3.0, 4.0), censored(0, 0.0, 1.0)],
    );
    // Diagnostic positions refer to the original input, before dimension sorting.
    assert!(matches!(
        result,
        Err(Error::InconsistentDiagram { interval: 1, .. })
    ));
}

#[test]
fn truncated_diagrams_check_every_endpoint_against_the_cutoff() {
    let invalid = [
        essential(0, 0.0),
        finite(0, 0.0, 3.0),
        finite(0, 3.0, 4.0),
        censored(0, 0.0, 1.0),
        censored(0, 0.0, 3.0),
    ];
    for interval in invalid {
        assert!(matches!(
            PersistenceDiagram::new(0, Coverage::Through(2.0), vec![interval]),
            Err(Error::InconsistentDiagram { .. })
        ));
    }
    let valid = PersistenceDiagram::new(
        1,
        Coverage::Through(2.0),
        vec![finite(0, 0.0, 2.0), censored(1, 2.0, 2.0)],
    )
    .unwrap();
    assert_eq!(valid.intervals().len(), 2);
}

#[test]
fn generic_coverage_accepts_negative_cutoffs_but_rejects_nonfinite_ones() {
    assert!(
        PersistenceDiagram::new(0, Coverage::Through(-1.0), vec![censored(0, -2.0, -1.0)]).is_ok()
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            PersistenceDiagram::new(0, Coverage::Through(value), vec![]),
            Err(Error::NonFiniteValue { .. })
        ));
    }
}

#[test]
fn canonical_order_retains_multiplicity_and_ignores_input_permutation() {
    let expected = vec![
        finite(0, 0.0, 1.0),
        finite(0, 0.0, 1.0),
        finite(0, 0.0, 2.0),
        essential(0, 0.0),
        finite(0, 1.0, 2.0),
        finite(1, -2.0, -1.0),
    ];
    let mut input = expected.clone();
    for _ in 0..input.len() {
        input.rotate_left(1);
        let diagram = PersistenceDiagram::new(1, Coverage::Complete, input.clone()).unwrap();
        assert_eq!(diagram.intervals(), expected.as_slice());
        assert_eq!(diagram.intervals_in_dimension(0).unwrap().count(), 5);
        assert_eq!(diagram.intervals_in_dimension(1).unwrap().count(), 1);
    }
    input.reverse();
    assert_eq!(
        PersistenceDiagram::new(1, Coverage::Complete, input)
            .unwrap()
            .intervals(),
        expected
    );
}

#[test]
fn construction_takes_ownership_of_the_interval_buffer() {
    let diagram = {
        let values = vec![finite(0, 0.0, 2.0), essential(0, 0.0)];
        let original_buffer = values.as_ptr();
        let diagram = PersistenceDiagram::new(0, Coverage::Complete, values).unwrap();
        assert_eq!(diagram.intervals().as_ptr(), original_buffer);
        diagram
    };
    assert_eq!(diagram.intervals().len(), 2);
}

#[test]
fn errors_integrate_with_standard_error_handling() {
    let error = DissimilarityView::new(&[-1.0], 2).unwrap_err();
    let standard: &dyn std::error::Error = &error;
    assert!(!standard.to_string().is_empty());
    assert!(standard.source().is_none());
    assert!(standard.to_string().contains("dissimilarities[0]"));
}
