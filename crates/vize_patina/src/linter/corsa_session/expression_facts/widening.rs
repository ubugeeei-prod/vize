//! Which queried ranges are typed as their outermost node with the same span.
use super::ExpressionQuery;

impl ExpressionQuery {
    /// Resolve this range to the outermost node with the same span. A type
    /// reference and its name share one, and only the reference is typed: the
    /// native runtime types a bare type name's identifier as the error type,
    /// where `getTypeAtLocation` on the type node answers the type it names.
    /// That covers union-part and empty-object queries, an `implements` clause
    /// (its `ExpressionWithTypeArguments`, as `getHeritageTypes` reads it) and
    /// type nodes a shape is asked for (`${T}` in a template literal type).
    pub(super) fn widens(&self) -> bool {
        self.parts
            || self.empty_object
            || self
                .heritage
                .as_ref()
                .is_some_and(|heritage| !heritage.base_types)
            || self.shape.as_ref().is_some_and(|shape| shape.type_node)
    }
}
