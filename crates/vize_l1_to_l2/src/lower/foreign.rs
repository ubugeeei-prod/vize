//! A first-party expression dialect selected once for an authored block.

use vize_l0::{Allocator, SourceBlock};
use vize_l1::{SurfaceError, SurfaceTree};
use vize_l2::expr::{ExprDialect, ExprRef};

use super::{LegacyCaps, Lowered, cx::Cx, finish::lower_in_context};

/// A registry name and its binding capability implementation. The core never
/// parses this dialect's expressions as JavaScript or interprets opaque facts.
#[derive(Clone, Copy)]
pub struct ForeignDialect<'a> {
    pub(super) name: &'a str,
    identifier: for<'e> fn(ExprRef<'e>) -> Option<&'e str>,
}

impl<'a> ForeignDialect<'a> {
    /// Registry ids are nonempty lowercase ASCII words.
    #[must_use]
    pub fn new<D: ExprDialect + Default>(name: &'a str) -> Option<Self> {
        (!name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase())).then_some(Self {
            name,
            identifier: foreign_identifier::<D>,
        })
    }
}

/// Lower Vue structure with every authored expression admitted as the selected
/// foreign payload. Source spans remain file-absolute; missing structural
/// positions and compound display runs retain their existing typed escapes.
#[must_use]
pub fn lower_source_block_with_foreign_expressions<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
    caps: LegacyCaps,
    dialect: ForeignDialect<'a>,
) -> Lowered<'a> {
    debug_assert!(
        tree.source.as_ptr() == block.source().as_ptr()
            && tree.source.len() == block.source().len(),
        "L1 and L0 must describe the same block"
    );
    let mut cx = Cx::with_source_block_and_comment_policy(allocator, block, caps, false, &[], None);
    cx.foreign_dialect = Some(dialect);
    lower_in_context(cx, tree, errors, block)
}

/// Binding positions use the dialect's one capability seam. A whole source
/// that exactly enumerates itself as its only binding is a simple identifier;
/// patterns keep their scope tag without guessing introduced names.
pub(super) fn simple_identifier<'a>(
    expr: &ExprRef<'a>,
    dialect: Option<ForeignDialect<'_>>,
) -> Option<&'a str> {
    if let (ExprRef::Foreign(foreign), Some(dialect)) = (expr, dialect)
        && foreign.dialect == dialect.name
    {
        return (dialect.identifier)(*expr);
    }
    super::expr::simple_identifier(expr)
}

/// The stateless first-party capability implementation is selected once at
/// entry through this monomorphized adapter, without a trait object or a
/// registry lookup inside the structural walk.
fn foreign_identifier<D: ExprDialect + Default>(expr: ExprRef<'_>) -> Option<&str> {
    let dialect = D::default();
    if !dialect.bindings_are_exact(expr) {
        return None;
    }
    let mut count = 0;
    let mut same = true;
    dialect.enumerate_bindings(expr, &mut |name| {
        count += 1;
        same &= name == expr.source();
    });
    (count == 1 && same).then_some(expr.source())
}
