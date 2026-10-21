use std::any::TypeId;

use vize_armature::ErrorCode as ArmatureErrorCode;
use vize_l0::ErrorCode as L0ErrorCode;
use vize_relief::ErrorCode as ReliefErrorCode;

fn through_relief(code: ReliefErrorCode) -> L0ErrorCode {
    code
}

fn through_armature(code: ArmatureErrorCode) -> L0ErrorCode {
    code
}

#[test]
fn compiler_error_paths_are_the_exact_l0_type() {
    assert_eq!(TypeId::of::<L0ErrorCode>(), TypeId::of::<ReliefErrorCode>());
    assert_eq!(
        TypeId::of::<L0ErrorCode>(),
        TypeId::of::<ArmatureErrorCode>()
    );

    for code in [
        L0ErrorCode::EofInTag,
        L0ErrorCode::InvalidExpression,
        L0ErrorCode::ExtendPoint,
    ] {
        let relief = through_relief(code);
        let armature = through_armature(code);

        assert_eq!(relief as u16, code as u16);
        assert_eq!(armature as u16, code as u16);
        assert_eq!(relief.message(), code.message());
        assert_eq!(armature.message(), code.message());
        assert_eq!(relief.code(), code.code());
        assert_eq!(armature.code(), code.code());
    }
}
