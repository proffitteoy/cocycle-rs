//! Lower-star filtration on supplied simplicial topology, using only public APIs.

use cocycle::complex::{Simplex, SimplicialComplex};
use cocycle::persistence::PersistenceExt;
use cocycle::{Error, Result};

/// Assign each simplex the maximum of its vertex values.
///
/// Vertex IDs are indices into `vertex_values`; all vertices, including isolates,
/// are inserted. Supply each nonvertex simplex exactly once, including every
/// nonvertex face, with strictly increasing IDs. No additional faces are inferred.
/// This example does not expose execution controls or a new library constructor.
fn lower_star_complex(
    vertex_values: &[f64],
    nonvertex_simplices: &[&[usize]],
) -> Result<SimplicialComplex> {
    let capacity = vertex_values
        .len()
        .checked_add(nonvertex_simplices.len())
        .ok_or(Error::SizeOverflow {
            operation: "lower-star simplex count",
        })?;
    let mut simplices = Vec::new();
    simplices
        .try_reserve_exact(capacity)
        .map_err(|_| Error::AllocationFailed {
            context: "lower-star simplices",
        })?;
    // Simplex::new also rejects nonfinite values, including isolated vertices.
    for (vertex, &value) in vertex_values.iter().enumerate() {
        simplices.push(Simplex::new(vec![vertex], value)?);
    }
    for &vertices in nonvertex_simplices {
        if vertices.len() < 2 {
            return Err(Error::InvalidParameter {
                parameter: "nonvertex_simplices",
                reason: "supply only edges and higher-dimensional simplices",
            });
        }
        let mut value = f64::NEG_INFINITY;
        for &vertex in vertices {
            let &birth = vertex_values.get(vertex).ok_or(Error::InvalidParameter {
                parameter: "vertex ID",
                reason: "no value supplied for this vertex",
            })?;
            value = value.max(birth);
        }
        simplices.push(Simplex::new(vertices.to_vec(), value)?);
    }
    SimplicialComplex::new(simplices)
}

fn main() -> Result<()> {
    // A circle with two local minima. These four edges are the entire topology.
    let complex = lower_star_complex(
        &[-2.0, 1.0, -1.0, 0.0],
        &[&[0, 1], &[1, 2], &[2, 3], &[0, 3]],
    )?;
    for simplex in complex.simplices() {
        println!("{:?} enters at {}", simplex.vertices(), simplex.value());
    }
    let edge = complex.find(&[0, 3]).expect("the supplied edge exists");
    for term in complex.boundary(edge).expect("valid simplex ID") {
        let face = complex.simplex(term.face).expect("a boundary face exists");
        println!(
            "Boundary term: {} * {:?}",
            term.coefficient,
            face.vertices()
        );
    }
    // The engine reads actual vertex births; no Rips-specific assumptions apply.
    let result = complex.persistence().max_homology_dimension(1).compute()?;
    println!("Intervals: {:?}", result.diagram().intervals());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cocycle::algebra::PrimeField;
    use cocycle::diagram::{Coverage, IntervalEnd};

    #[test]
    fn circle_has_hand_derived_pairs_over_several_fields() -> Result<()> {
        let complex = lower_star_complex(
            &[-2.0, 1.0, -1.0, 0.0],
            &[&[0, 1], &[1, 2], &[2, 3], &[0, 3]],
        )?;
        assert_eq!(complex.len(), 8);
        assert_eq!(complex.vertex_count(), 4);
        // Check the construction independently of any persistence diagram.
        let expected: &[(&[usize], f64)] = &[
            (&[0], -2.0),
            (&[1], 1.0),
            (&[2], -1.0),
            (&[3], 0.0),
            (&[0, 1], 1.0),
            (&[1, 2], 1.0),
            (&[2, 3], 0.0),
            (&[0, 3], 0.0),
        ];
        for &(vertices, value) in expected {
            let id = complex.find(vertices).expect("expected simplex exists");
            assert_eq!(complex.simplex(id).unwrap().value(), value);
        }
        for prime in [2, 3, 251] {
            let result = complex
                .persistence()
                .field(PrimeField::new(prime)?)
                .compute()?;
            let pairs: Vec<_> = result
                .diagram()
                .intervals()
                .iter()
                .map(|i| (i.dimension(), i.birth(), i.end()))
                .collect();
            // Minima -2 and -1 merge at 0. At 1 the final two edges close a loop.
            assert_eq!(
                pairs,
                [
                    (0, -2.0, IntervalEnd::Essential),
                    (0, -1.0, IntervalEnd::Finite(0.0)),
                    (1, 1.0, IntervalEnd::Essential),
                ]
            );
        }
        let result = complex.persistence().max_filtration_value(-0.5).compute()?;
        assert_eq!(result.diagram().coverage(), Coverage::Through(-0.5));
        assert_eq!(result.diagram().intervals().len(), 2);
        for interval in result.diagram().intervals() {
            assert_eq!(interval.dimension(), 0);
            assert_eq!(interval.end(), IntervalEnd::RightCensored { through: -0.5 });
        }
        Ok(())
    }

    #[test]
    fn tied_triangle_has_expected_oriented_boundary_and_no_persistent_loop() -> Result<()> {
        // Deliberately supply the coface first. The constructor sorts by value
        // and dimension, so all faces still precede it when every value is tied.
        let complex = lower_star_complex(&[2.0; 3], &[&[0, 1, 2], &[1, 2], &[0, 2], &[0, 1]])?;
        assert_eq!(complex.len(), 7);
        let triangle = complex.find(&[0, 1, 2]).unwrap();
        let terms: Vec<_> = complex
            .boundary(triangle)
            .unwrap()
            .iter()
            .map(|t| {
                (
                    complex.simplex(t.face).unwrap().vertices().to_vec(),
                    t.coefficient,
                )
            })
            .collect();
        assert_eq!(terms, [(vec![1, 2], 1), (vec![0, 2], -1), (vec![0, 1], 1)]);
        for edge in [&[0, 1][..], &[0, 2], &[1, 2]] {
            assert_eq!(
                complex.cofacets(complex.find(edge).unwrap()).unwrap(),
                &[triangle]
            );
        }
        // Sum boundary-of-boundary over the integers, using stored incidences.
        let mut coefficients = [0_i32; 3];
        for edge in complex.boundary(triangle).unwrap() {
            for vertex in complex.boundary(edge.face).unwrap() {
                let label = complex.simplex(vertex.face).unwrap().vertices()[0];
                coefficients[label] += i32::from(edge.coefficient) * i32::from(vertex.coefficient);
            }
        }
        assert_eq!(coefficients, [0; 3]);
        let result = complex.persistence().field(PrimeField::new(3)?).compute()?;
        assert_eq!(result.diagram().intervals().len(), 1);
        let interval = &result.diagram().intervals()[0];
        assert_eq!(
            (interval.dimension(), interval.birth(), interval.end()),
            (0, 2.0, IntervalEnd::Essential)
        );
        Ok(())
    }

    #[test]
    fn empty_inputs_and_isolates_keep_their_meaning() -> Result<()> {
        assert!(lower_star_complex(&[], &[])?.is_empty());
        let complex = lower_star_complex(&[-3.0, 4.0], &[])?;
        assert_eq!(complex.dimension(), Some(0));
        assert_eq!(complex.vertex_count(), 2);
        let result = complex.persistence().compute()?;
        let births: Vec<_> = result
            .diagram()
            .intervals()
            .iter()
            .map(|i| i.birth())
            .collect();
        assert_eq!(births, [-3.0, 4.0]);
        assert!(
            result
                .diagram()
                .intervals()
                .iter()
                .all(|i| i.end() == IntervalEnd::Essential)
        );
        Ok(())
    }

    #[test]
    fn malformed_topology_and_nonfinite_values_are_rejected() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                lower_star_complex(&[value], &[]),
                Err(Error::NonFiniteValue { .. })
            ));
        }
        for cells in [&[&[][..]][..], &[&[0]], &[&[0, 2]]] {
            assert!(matches!(
                lower_star_complex(&[0.0; 2], cells),
                Err(Error::InvalidParameter { .. })
            ));
        }
        for cells in [
            &[&[1, 0][..]][..],
            &[&[0, 0]],
            &[&[0, 1], &[0, 1]],
            &[&[0, 1, 2], &[0, 1], &[1, 2]], // Missing edge [0, 2].
        ] {
            assert!(matches!(
                lower_star_complex(&[0.0; 3], cells),
                Err(Error::InvalidComplex { .. })
            ));
        }
    }
}
