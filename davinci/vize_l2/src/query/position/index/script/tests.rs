//! The selector must preserve each original row and reject every collision.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects the symbolic harness with cfg(kani)"
)]

use super::{NameSite, PositionQueryError};

fn check(declaration: Option<u32>, reference: Option<u32>) {
    let site = NameSite::select(declaration, reference);
    assert_eq!(site.is_err(), declaration.is_some() && reference.is_some());
    match site {
        Ok(NameSite::Declaration(row)) => {
            assert_eq!(declaration, Some(row));
            assert_eq!(reference, None);
        }
        Ok(NameSite::Reference(row)) => {
            assert_eq!(reference, Some(row));
            assert_eq!(declaration, None);
        }
        Ok(NameSite::Vacant) => assert_eq!((declaration, reference), (None, None)),
        Err(error) => assert_eq!(error, PositionQueryError::AmbiguousSite),
    }
}

#[test]
fn equal_row_numbers_in_different_tables_still_collide() {
    for declaration in [None, Some(0), Some(7), Some(u32::MAX)] {
        for reference in [None, Some(0), Some(7), Some(u32::MAX)] {
            check(declaration, reference);
        }
    }
}

#[cfg(kani)]
#[kani::proof]
fn position_queries_compose_distinct_tables_without_losing_the_original_row() {
    let declaration = kani::any();
    let reference = kani::any();
    check(declaration, reference);
    kani::cover!(
        declaration.is_none() && reference.is_none(),
        "unoccupied byte"
    );
    kani::cover!(declaration.is_some() && reference.is_none(), "declaration");
    kani::cover!(declaration.is_none() && reference.is_some(), "reference");
    kani::cover!(
        declaration.is_some() && declaration == reference,
        "equal IDs from different tables"
    );
}
