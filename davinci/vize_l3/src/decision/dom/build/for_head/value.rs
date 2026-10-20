//! Current original callback value, without script provenance or another walk.

use super::super::{DomBuilder, DomExpressionFacts};
use crate::decision::dom::vue::{VueReadKind, policy::FileReads};
use vize_l2::{
    expr::JsExpr,
    file::{BindingRef, FileResolution},
    op::Op,
    resolution::{ForAliasRole, Occurrence, Usage},
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(in crate::decision::dom::build) fn for_value(
        &self,
        resolution: FileResolution<'owner, 'arena>,
        expression: &JsExpr<'arena>,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        if !self.in_original_for_body() || occurrence.usage != Usage::Read {
            return None;
        }
        let [(origin, frame), (_, body)] = self.frames.as_slice() else {
            return None;
        };
        let Op::OriginalFor(original) = frame.node.op else {
            return None;
        };
        if !matches!(body.node.op, Op::Element(element) if element.attributes.is_empty()) {
            return None;
        }
        let file = self.file?;
        let [original_occurrence] = resolution.table()?.occurrences() else {
            return None;
        };
        if !core::ptr::eq(original_occurrence, occurrence)
            || !crate::decision::dom::file::matches_expression(resolution, expression)
        {
            return None;
        }
        let row = self.facts.file_for_heads.get(*origin)?;
        let expected = row.resolution().value_declaration();
        let head = row.head();
        let alias = head.value()?;
        let scope = head.scope()?;
        let identifier = expression.ast.get_identifier_reference()?;
        if !core::ptr::eq(resolution.file(), file)
            || !core::ptr::eq(binding.file(), file)
            || !row.accepts_original(original)
            || resolution.scope() != Some(scope)
            || expected.fact().role() != ForAliasRole::Value
            || binding.id() != alias.id()
            || !binding.same_owner(alias)
            || occurrence.binding != binding.id()
            || occurrence.name != expected.fact().name()
            || identifier.name.as_str() != occurrence.name
            || expression.source != occurrence.name
            || expression.ast_span_to_source(identifier.span) != Some(occurrence.span)
            || expression
                .source
                .get(occurrence.span.start as usize..occurrence.span.end as usize)
                != Some(occurrence.name)
            || !super::alias(Some(binding), &expected, file, head.id(), scope)
        {
            return None;
        }
        Some(VueReadKind::ForValue)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::decision::dom::{DomChild, DomChildren, DomUnsupported, LiteralExpressions};
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
    };
    use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
    use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
    use vize_l2::{file::DeclarationKind, lang::js::NativeSelectedSetup};

    struct Reads<'v, 'o, 'a>(&'v NativeSelectedSetup<'o, 'a>);
    impl<'o, 'a> FileReads<'o, 'a> for Reads<'_, 'o, 'a> {
        const RECORD: bool = true;
        const NATIVE_SETUP: bool = true;
        const ORIGINAL_FOR: bool = true;
        fn classify(
            &self,
            occurrence: &Occurrence<'a>,
            binding: BindingRef<'o, 'a>,
        ) -> Option<VueReadKind> {
            let declaration = self.0.binding(binding).ok()?.declaration()?;
            (occurrence.usage == Usage::Read
                && matches!(
                    declaration.kind,
                    DeclarationKind::Let | DeclarationKind::Var
                ))
            .then_some(VueReadKind::SetupLet)
        }
    }

    #[test]
    fn actual_callback_rejects_foreign_binding_resolution_neutral_expression_and_missing_read_row()
    {
        let arena = Allocator::default();
        let source = "<script setup>let count=2</script><template><i v-for='item in count'>{{item}}</i></template>";
        let options = DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        };
        let observation = lower_selected_setup_sfc_native(&arena, source, options);
        let other = lower_selected_setup_sfc_native(&arena, source, options);
        let view = observation.admitted().unwrap();
        let foreign_view = other.admitted().unwrap();
        let setup = view.setup();
        let file = setup.file();
        let [root @ Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("For");
        };
        let [body @ Op::Element(element)] = original.region.ops.as_slice() else {
            panic!("body");
        };
        let [interpolation @ Op::Interpolation(_)] = element.children.ops.as_slice() else {
            panic!("value");
        };
        let analysis =
            crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
        let DomChildren::Array(children) = &analysis
            .dom()
            .unwrap()
            .node(original.id().node())
            .unwrap()
            .children
        else {
            panic!("carrier");
        };
        let [DomChild::Node(body_node)] = children.as_slice() else {
            panic!("body node");
        };
        let [record] = file.native_interpolations() else {
            panic!("original");
        };
        let vize_l2::file::NativeFileInterpolationState::Admitted(node) = record.state() else {
            panic!("attached");
        };
        let resolution = file.expression(node).unwrap();
        let table = resolution.table().unwrap();
        let expression = table.expression();
        let occurrence = &table.occurrences()[0];
        let binding = resolution.binding(occurrence.binding).unwrap();
        let foreign = foreign_view.setup().file();
        let foreign_resolution = foreign.expression(node).unwrap();
        let foreign_binding = foreign_resolution.binding(occurrence.binding).unwrap();
        assert_eq!(binding.id(), foreign_binding.id());
        let reads = Reads(setup);
        let mut builder = DomBuilder::new(&LiteralExpressions, 1, Some(file), &reads);
        builder
            .enter(original.id().node(), root, None, &mut None)
            .unwrap();
        builder.enter(*body_node, body, None, &mut None).unwrap();
        assert_eq!(
            builder.for_value(resolution, expression, occurrence, binding),
            Some(VueReadKind::ForValue)
        );
        assert_eq!(
            builder.for_value(foreign_resolution, expression, occurrence, binding),
            None
        );
        assert_eq!(
            builder.for_value(resolution, expression, occurrence, foreign_binding),
            None
        );
        let copied = *occurrence;
        assert_eq!(
            builder.for_value(resolution, expression, &copied, binding),
            None
        );
        let neutral = JsExpr {
            coordinates: None,
            ..*expression
        };
        assert_eq!(builder.file_value(node, &neutral), None);
        assert_eq!(
            builder.facts.unsupported.last().unwrap().reason,
            DomUnsupported::FileExpression
        );
        assert!(builder.facts.vue_expressions.get(node).is_none());
        builder.enter(node, interpolation, None, &mut None).unwrap();
        builder.leave(node).unwrap();
        builder.leave(*body_node).unwrap();
        assert!(builder.facts.vue_expressions.get(node).is_some());
        // One genuine row is required at the existing root Leave; a missing table
        // cannot turn the dynamic body into an eligible static fallback.
        builder.facts.vue_expressions = vize_l0::side_table::SideTable::new();
        builder.leave(original.id().node()).unwrap();
        assert_eq!(
            builder.facts.unsupported.last().unwrap().reason,
            DomUnsupported::ForBody
        );
        assert!(
            !builder
                .facts
                .node(original.id().node())
                .unwrap()
                .block_eligible
        );
    }

    #[test]
    fn caught_open_callback_unwind_preserves_original_program_params_and_interpolation() {
        let arena = Allocator::default();
        let source = "<script setup>let count=2</script><template><i v-for='item in count'>{{item}}</i></template>";
        let observation = lower_selected_setup_sfc_native(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let view = observation.admitted().unwrap();
        let setup = view.setup();
        let file = setup.file();
        let [root @ Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("For");
        };
        let [body @ Op::Element(element)] = original.region.ops.as_slice() else {
            panic!("body");
        };
        let [interpolation @ Op::Interpolation(_)] = element.children.ops.as_slice() else {
            panic!("value");
        };
        let analysis =
            crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
        let DomChildren::Array(children) = &analysis
            .dom()
            .unwrap()
            .node(original.id().node())
            .unwrap()
            .children
        else {
            panic!("carrier");
        };
        let [DomChild::Node(body_node)] = children.as_slice() else {
            panic!("body node");
        };
        let [record] = file.native_interpolations() else {
            panic!("original");
        };
        let vize_l2::file::NativeFileInterpolationState::Admitted(node) = record.state() else {
            panic!("attached");
        };
        let head = file.for_head_for(original).unwrap();
        let parameter = head.resolution().unwrap().value_declaration().parameter() as *const _;
        let ast = record.input().operand().syntax().expression().unwrap() as *const _;
        let program = setup.program().program().body.as_ptr();
        let reads = Reads(setup);
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut builder = DomBuilder::new(&LiteralExpressions, 1, Some(file), &reads);
            builder
                .enter(original.id().node(), root, None, &mut None)
                .unwrap();
            builder.enter(*body_node, body, None, &mut None).unwrap();
            builder.enter(node, interpolation, None, &mut None).unwrap();
            assert_eq!(
                builder.facts.vue_expressions.get(node).unwrap().reads()[0].kind(),
                VueReadKind::ForValue
            );
            assert_eq!(builder.frames.len(), 3);
            panic!("stop before callback/body/root Leave");
        }));
        assert!(caught.is_err());
        assert!(file.is_complete() && head.accepts(original));
        assert_eq!(
            parameter,
            head.resolution().unwrap().value_declaration().parameter() as *const _
        );
        assert_eq!(
            ast,
            record.input().operand().syntax().expression().unwrap() as *const _
        );
        assert_eq!(program, setup.program().program().body.as_ptr());
        let rebuilt =
            crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
        assert!(rebuilt.dom().unwrap().unsupported().is_empty());
    }
}
