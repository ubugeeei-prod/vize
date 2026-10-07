use super::{Control, checkpoint};

#[test]
fn lexical_control_restores_the_previous_owner_on_return_and_unwind() {
    assert!(checkpoint().is_ok());
    let outer = Control::new();
    let _outer = outer.enter();
    let inner = Control::new();
    let result = std::panic::catch_unwind(|| {
        let _inner = inner.enter();
        inner.retire();
        assert_eq!(
            checkpoint().unwrap_err(),
            "Native caller retired before completing its operation"
        );
        panic!("exercise lexical restoration");
    });
    assert!(result.is_err());
    assert!(checkpoint().is_ok());
    outer.retire();
    assert!(checkpoint().is_err());
    drop(_outer);
    assert!(checkpoint().is_ok());
}
