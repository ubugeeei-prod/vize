use vize_armature::parse;
use vize_s0::{Allocator, profiler::global_profiler};

#[test]
fn nested_directive_arguments_keep_the_parse_once_contract() {
    let source = r#"<Child :[keys[index]]="value" @[events[index]]="handler" v-model:[models[index]]="value"/>"#;
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, source);
    profiler.disable();
    assert!(errors.is_empty(), "{errors:?}");
    let summary = profiler.counter_summary();
    let counter = summary
        .entries
        .iter()
        .find(|entry| entry.name == "davinci.expr.parses")
        .unwrap();
    // Three arguments and three values, with no parser invocation for finding
    // the lexical boundary of any dynamic argument.
    assert_eq!(counter.samples, 6);
    assert_eq!(counter.total, 6);
}
