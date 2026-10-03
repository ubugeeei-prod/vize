use super::*;
use vize_l2::lang::js::SetupPrimitiveType;

#[test]
fn original_keyword_annotations_keep_same_file_program_colon_spans_and_order() -> Result<(), String>
{
    let arena = Allocator::default();
    let source = "<script setup lang=ts>/* 雪 */const count: /* 🌸 */ number=1, text:string=': number';let yes: boolean=true, huge:bigint=2n;var nil:null=null, absent:undefined=null, symbol:symbol=1;const plain=2;</script>";
    let original = Observed::new(&arena, source)?;
    let file = original.file(&arena)?;
    let setup = checked(&original, &file)?.map_err(|_| "original annotated TS")?;
    require!(file.is_complete());
    require!(core::ptr::eq(
        setup.exposure().program().program(),
        original.syntax.program().ok_or("original Program")?
    ));
    require!(core::ptr::eq(setup.source().root_source(), source));
    equal!(original.syntax.diagnostics().count(), 0);
    equal!(original.syntax.comments().count(), 2);
    equal!(setup.bindings().count(), 8);
    equal!(setup.type_annotations().count(), 7);
    let mut end = setup.source().start();
    for (annotation, (kind, expected)) in setup.type_annotations().zip([
        (SetupPrimitiveType::Number, ": /* 🌸 */ number"),
        (SetupPrimitiveType::String, ":string"),
        (SetupPrimitiveType::Boolean, ": boolean"),
        (SetupPrimitiveType::BigInt, ":bigint"),
        (SetupPrimitiveType::Null, ":null"),
        (SetupPrimitiveType::Undefined, ":undefined"),
        (SetupPrimitiveType::Symbol, ":symbol"),
    ]) {
        require!(core::ptr::eq(annotation.file(), &file));
        equal!(annotation.kind(), kind);
        let span = annotation.span();
        require!(setup.source().contains_block_span(span));
        require!(span.start >= end && span.start < span.end);
        equal!(
            source.get(span.start as usize..span.end as usize),
            Some(expected)
        );
        end = span.end;
    }
    Ok(())
}

#[test]
fn nonkeyword_nested_exported_definite_and_nonprimitive_annotations_keep_file_issues()
-> Result<(), String> {
    let arena = Allocator::default();
    for body in [
        "const value: number | string = 1;",
        "const value: number & string = 1;",
        "const value: (number) = 1;",
        "const value: 1 = 1;",
        "const value: 'text' = 1;",
        "const value: true = 1;",
        "const value: Name = 1;",
        "const value: any = 1;",
        "const value: unknown = 1;",
        "const value: never = 1;",
        "const value: void = 1;",
        "const value: object = 1;",
        "const value: number[] = 1;",
        "const value: [number] = 1;",
        "const value: readonly number[] = 1;",
        "const value: typeof other = 1;",
        "const value: keyof any = 1;",
        "const value: {name:number} = 1;",
        "const value: {[K in string]:number} = 1;",
        "const value: () => number = 1;",
        "const value: new () => number = 1;",
        "const value: Promise<number> = 1;",
        "const value: number extends string ? number : string = 1;",
        "declare const value:number;",
        "let value!:number;",
        "export const value:number=1;",
        "function hidden(){const value:number=1;}const visible=1;",
        "const value:number=other;",
        "const value:number=call();",
        "import {counter} from 'x';let value:number=counter;",
        "const value:number={} ;",
        "const value:number=1 as number;",
    ] {
        let source = cstr!("<script setup lang=ts>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        require!(
            original.syntax.admitted_program().is_some(),
            "genuine parse: {body}"
        );
        let file = original.file(&arena)?;
        require!(!file.is_complete(), "no false whole File: {body}");
        require!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax),
            "actual unsupported File issue: {body}"
        );
        require!(checked(&original, &file)?.is_err(), "no capability: {body}");
        require!(core::ptr::eq(file.artifact().source(), source.as_str()));
        equal!(original.syntax.diagnostics().count(), 0);
    }
    Ok(())
}

#[test]
fn keyword_annotation_rows_never_bypass_original_strict_or_source_authority() -> Result<(), String>
{
    let arena = Allocator::default();
    for body in [
        "const value:number=010;",
        "let value:number=08;",
        r"var value:string='\1';",
        "const eval:number=1;",
        "let arguments:number=1;",
        r"const \u0065val:number=1;",
        "'use strict';const value:number=1;",
    ] {
        let source = cstr!("<script setup lang=ts>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        require!(
            checked(&original, &file)?.is_err(),
            "strict original body: {body}"
        );
        equal!(original.syntax.diagnostics().count(), 0);
    }
    let source = String::from("<script setup lang=ts>const value:number=1;</script>");
    let copied = source.clone();
    let original = Observed::new(&arena, &source)?;
    let foreign = Observed::new(&arena, &copied)?;
    let file = original.file(&arena)?;
    require!(
        matches!(checked(&foreign,&file)?,Err(issue) if issue.kind==SetupIssueKind::Exposure(ExposureIssueKind::Source))
    );
    let script = original.script()?;
    let second = Parser::new(
        &arena,
        script.block().source(),
        SourceType::ts().with_module(true),
    )
    .parse_observed();
    require!(
        matches!(VueSetup::checked(&file,script,second.admitted().ok_or("second parse")?),Err(issue) if issue.kind==SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin))
    );
    equal!(
        checked(&original, &file)?
            .map_err(|_| "actual original")?
            .type_annotations()
            .count(),
        1
    );
    Ok(())
}

#[test]
fn actual_annotation_event_interruption_cannot_complete_the_file_or_expose_receipts()
-> Result<(), String> {
    let arena = Allocator::default();
    let original = Observed::new(
        &arena,
        "<script setup lang=ts>const first:number=1;let after:string='x';</script>",
    )?;
    let script = original.script()?;
    for point in [Point::Unit, Point::Statement, Point::Declared] {
        let mut producer =
            FileProducer::new(&arena, original.descriptor.source()).map_err(|_| "file")?;
        let interruption = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer
                .program_observed(
                    ProgramInput::checked(
                        original.syntax.admitted_program().ok_or("Program")?,
                        script.block(),
                        script.container_index(),
                    )
                    .map_err(|_| "input")?,
                    ProgramScope::Nested,
                    &mut Interrupt(point),
                )
                .map_err(|_| "unit")
        }));
        require!(
            interruption.is_err(),
            "actual callback interruption: {point:?}"
        );
        let file = producer.finish().map_err(|_| "artifact")?;
        require!(!file.is_complete());
        equal!(file.interrupted_programs().count(), 1);
        equal!(
            file.bindings().count(),
            usize::from(point == Point::Declared)
        );
        require!(
            matches!(checked(&original,&file)?,Err(issue) if issue.kind==SetupIssueKind::Exposure(ExposureIssueKind::IncompleteFile))
        );
        require!(core::ptr::eq(
            file.artifact().source(),
            original.descriptor.source()
        ));
    }
    equal!(
        checked(&original, &original.file(&arena)?)?
            .map_err(|_| "complete setup")?
            .type_annotations()
            .count(),
        2
    );
    Ok(())
}

#[test]
fn unannotated_original_js_and_ts_keep_empty_annotation_receipts() -> Result<(), String> {
    let arena = Allocator::default();
    for lang in ["", " lang=ts"] {
        let source = cstr!("<script setup{lang}>const value=1;let text=':number';</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        let setup = checked(&original, &file)?.map_err(|_| "unannotated original")?;
        equal!(setup.type_annotations().count(), 0);
        equal!(setup.bindings().count(), 2);
        require!(file.is_complete());
    }
    Ok(())
}
