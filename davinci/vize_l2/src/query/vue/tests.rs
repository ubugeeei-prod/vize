use super::{Family, VueQueryError};
use crate::file::FileArtifact;
use crate::framework::vue::{
    MacroKind, OptionMemberKind, OptionsFacts, OptionsObserver, ReactiveSource, SetupBindingKind,
    SetupFacts, SetupObserver,
};
use crate::lang::js::{FileObserver, FileProducer, ProgramInput, ProgramScope};
use crate::query::Model;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};

fn observe<'a, O: FileObserver<'a>>(
    arena: &'a Allocator,
    source: &'a str,
    observer: &mut O,
) -> FileArtifact<'a> {
    let syntax = Parser::new(arena, source, SourceType::ts().with_module(true)).parse_observed();
    assert!(syntax.diagnostics().is_empty());
    let block = SourceRoot::new(source).unwrap().whole_block();
    let input = ProgramInput::checked(syntax.admitted().unwrap(), block, 0).unwrap();
    let mut producer = FileProducer::new(arena, source).unwrap();
    producer
        .program_observed(input, ProgramScope::Nested, observer)
        .unwrap();
    producer.finish().unwrap()
}

fn setup<'a>(arena: &'a Allocator, source: &'a str) -> (FileArtifact<'a>, SetupFacts) {
    let mut observer = SetupObserver::new();
    let file = observe(arena, source, &mut observer);
    (file, observer.finish().unwrap())
}

fn options<'a>(arena: &'a Allocator, source: &'a str) -> (FileArtifact<'a>, OptionsFacts) {
    let mut observer = OptionsObserver::new();
    let file = observe(arena, source, &mut observer);
    (file, observer.finish().unwrap())
}

const SETUP: &str = "import { ref, computed } from 'vue';
import Child from './Child.vue';
const props = defineProps<{ label: string }>();
const emit = defineEmits(['change']);
const count = ref(0);
const double = computed(() => count.value * 2);
let draft = '';
const limit = 3;
function bump() { const local = count.value; count.value = local + 1; }
";

#[test]
fn every_setup_row_belongs_to_exactly_one_file_binding_by_declaration_site() {
    let arena = Allocator::default();
    let (file, facts) = setup(&arena, SETUP);
    let model = Model::new(&file).with_setup(&facts);
    assert!(!facts.bindings().is_empty());
    for row in facts.bindings() {
        let owners = file
            .bindings()
            .filter(|binding| model.setup_binding(*binding) == Ok(Some(row)))
            .count();
        assert_eq!(owners, 1, "{row:?}");
    }
    let kind = |name: &str| {
        let binding = file
            .bindings()
            .find(|binding| {
                binding.declaration().is_some_and(|row| {
                    SETUP.get(row.span.start as usize..row.span.end as usize) == Some(name)
                })
            })
            .unwrap();
        model
            .setup_binding(binding)
            .unwrap()
            .map(|row| (row.kind, row.source))
    };
    assert_eq!(
        kind("count"),
        Some((SetupBindingKind::SetupRef, Some(ReactiveSource::Ref)))
    );
    assert_eq!(
        kind("double"),
        Some((SetupBindingKind::SetupRef, Some(ReactiveSource::Computed)))
    );
    assert_eq!(kind("draft"), Some((SetupBindingKind::SetupLet, None)));
    assert_eq!(kind("bump"), Some((SetupBindingKind::SetupConst, None)));
    // A nested declaration is a File binding without a setup row.
    assert_eq!(kind("local"), None);
}

#[test]
fn macro_calls_and_contract_are_the_attached_facts() {
    let arena = Allocator::default();
    let (file, facts) = setup(&arena, SETUP);
    let model = Model::new(&file).with_setup(&facts);
    let calls = model.macro_calls().unwrap();
    assert!(core::ptr::eq(calls, facts.macro_calls()));
    assert!(
        calls
            .iter()
            .map(|call| call.kind)
            .eq([MacroKind::DefineProps, MacroKind::DefineEmits])
    );
    assert!(core::ptr::eq(model.contract().unwrap(), facts.contract()));
    assert_eq!(
        model
            .contract()
            .unwrap()
            .props
            .as_ref()
            .unwrap()
            .props
            .len(),
        1
    );
}

#[test]
fn missing_families_and_foreign_bindings_are_refused() {
    let arena = Allocator::default();
    let (file, facts) = setup(&arena, SETUP);
    let (other, _) = setup(&arena, SETUP);
    let binding = file.bindings().next().unwrap();
    let bare = Model::new(&file);
    let missing = Err(VueQueryError::NotObserved(Family::Setup));
    assert_eq!(bare.setup_binding(binding).map(|_| ()), missing);
    assert_eq!(bare.macro_calls().map(|_| ()), missing);
    assert_eq!(bare.contract().map(|_| ()), missing);
    assert_eq!(
        bare.component_options().map(|_| ()),
        Err(VueQueryError::NotObserved(Family::Options))
    );
    let model = Model::new(&file).with_setup(&facts);
    let foreign = other.bindings().next().unwrap();
    assert_eq!(foreign.id(), binding.id());
    assert_eq!(
        model.setup_binding(foreign),
        Err(VueQueryError::ForeignBinding)
    );
}

#[test]
fn unclassified_destructure_declarations_never_get_a_setup_row() {
    let arena = Allocator::default();
    let source = "const { a } = defineProps(['a']);\nconst { b } = defineProps(['b']);\n";
    let (file, facts) = setup(&arena, source);
    let model = Model::new(&file).with_setup(&facts);
    let [pattern] = facts.unclassified() else {
        panic!("{:?}", facts.unclassified());
    };
    assert_eq!(
        source.get(pattern.start as usize..pattern.end as usize),
        Some("{ b }")
    );
    // A binding declared inside the pattern is refused, never matched by name;
    // every other binding answers from its own declaration site.
    for binding in file.bindings() {
        let site = binding.declaration().unwrap().span;
        let inside = pattern.start <= site.start && site.end <= pattern.end;
        match model.setup_binding(binding) {
            Err(VueQueryError::Unclassified(refused)) => assert!(inside && refused == *pattern),
            Ok(row) => assert!(!inside && row.is_none_or(|row| row.name == site)),
            Err(error) => panic!("{error:?}"),
        }
    }
}

#[test]
fn options_component_is_the_attached_options_facts() {
    let arena = Allocator::default();
    let source = "export default { props: ['size'], data() { return { open: false }; } };\n";
    let (file, facts) = options(&arena, source);
    let model = Model::new(&file).with_options(&facts);
    let component = model.component_options().unwrap().unwrap();
    assert!(core::ptr::eq(component, facts.component().unwrap()));
    assert!(
        component
            .members
            .iter()
            .any(|member| member.kind == OptionMemberKind::Prop)
    );
    assert_eq!(
        model.setup_facts().map(|_| ()),
        Err(VueQueryError::NotObserved(Family::Setup))
    );
}
