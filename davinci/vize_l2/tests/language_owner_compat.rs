use core::any::type_name;

#[test]
fn language_owned_payloads_keep_their_existing_public_type_identity() {
    assert_eq!(
        type_name::<vize_l2::expr::ForeignExpr<'static>>(),
        "vize_l2::expr::foreign::ForeignExpr<'_>"
    );
    assert_eq!(
        type_name::<vize_l2::dump::Expr>(),
        "vize_l2::dump::owned::expr::Expr"
    );
}
