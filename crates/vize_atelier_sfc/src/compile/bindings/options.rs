//! The Options API members a normal `<script>` exposes to its template, read
//! from the native L2 Options API facts (#6844, S7).
//!
//! The prepared script content is parsed once by L1 with the block's `lang`
//! and the L2 Program walk runs `OptionsObserver` over it. Nothing falls back
//! to another analysis: a script the levels refuse forwards no members, and
//! its template reads them through the render context, as it does for a
//! component that declares none.

use vize_l0::{Allocator, SourceRoot, Span, String, ToCompactString, camelize, profile};
use vize_l1::embed::{EmbedSource, syntax::parse_program_once};
use vize_l2::framework::vue::{OptionMemberKind, OptionsObserver};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

use crate::compile::setup_facts::program_options;
use crate::types::{BindingMetadata, BindingType};

/// The binding of each member the template reads through the instance:
/// `props` as props, `data` and Nuxt `asyncData` as data, `computed`,
/// `methods` and `inject` as options. A later member of the same name
/// replaces an earlier one; `setup()` returns stay setup state, so a
/// returned name drops the option binding it replaces, and Vue 3 has no
/// `filters`.
pub(in crate::compile) fn options_api(
    content: &str,
    lang: Option<&str>,
) -> Option<BindingMetadata> {
    profile!(
        "atelier.sfc.normal_script.options_api_bindings",
        observe(content, lang)
    )
}

fn observe(content: &str, lang: Option<&str>) -> Option<BindingMetadata> {
    let arena = Allocator::default();
    let span = Span::new(0, u32::try_from(content.len()).ok()?);
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(content, span).ok()?,
        program_options(lang),
    );
    let block = SourceRoot::new(content).ok()?.whole_block();
    let input = ProgramInput::checked(syntax.admitted_program()?, block, 0).ok()?;
    let mut observer = OptionsObserver::new();
    FileProducer::new(&arena, content)
        .ok()?
        .program_observed(input, ProgramScope::Nested, &mut observer)
        .ok()?;
    let facts = observer.finish().ok()?;
    let mut bindings = BindingMetadata::default();
    for member in &facts.component()?.members {
        let binding = match member.kind {
            OptionMemberKind::Prop => Some(BindingType::Props),
            OptionMemberKind::Data | OptionMemberKind::AsyncData => Some(BindingType::Data),
            OptionMemberKind::Computed | OptionMemberKind::Method | OptionMemberKind::Inject => {
                Some(BindingType::Options)
            }
            OptionMemberKind::Setup => None,
            OptionMemberKind::Filter => continue,
        };
        let name = content.get(member.name.start as usize..member.name.end as usize)?;
        let Some(name) = template_name(name) else {
            continue;
        };
        match binding {
            Some(binding) => bindings.bindings.insert(name, binding),
            None => bindings.bindings.remove(&name),
        };
    }
    (!bindings.bindings.is_empty()).then_some(bindings)
}

/// The template name of a member: an ASCII identifier as written, or the
/// camel case of a kebab-case name when that is one.
fn template_name(name: &str) -> Option<String> {
    let identifier = |name: &str| {
        let mut chars = name.chars();
        chars
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || matches!(first, '_' | '$'))
            && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
    };
    if identifier(name) {
        return Some(name.to_compact_string());
    }
    let camel = camelize(name);
    (name.contains('-') && identifier(&camel)).then_some(camel)
}

#[cfg(test)]
mod tests {
    use super::options_api;
    use crate::types::BindingType;

    #[test]
    fn members_bind_by_option_group() {
        let script = "export default {
  props: ['p', 'kebab-prop'],
  inject: { i: 'key' },
  computed: { c() { return 1 } },
  methods: { m() {} },
  data() { return { d: 1, shadow: 2 } },
  asyncData() { return { n: 1 } },
  setup() { return { shadow: 3 } },
  filters: { f() {} },
}";
        let bindings = options_api(script, Some("ts")).expect("bindings");
        let read = |name: &str| bindings.bindings.get(name).copied();
        assert_eq!(read("p"), Some(BindingType::Props));
        assert_eq!(read("kebabProp"), Some(BindingType::Props));
        assert_eq!(read("i"), Some(BindingType::Options));
        assert_eq!(read("c"), Some(BindingType::Options));
        assert_eq!(read("m"), Some(BindingType::Options));
        assert_eq!(read("d"), Some(BindingType::Data));
        assert_eq!(read("n"), Some(BindingType::Data));
        assert_eq!(read("shadow"), None);
        assert_eq!(read("f"), None);
        assert_eq!(bindings.bindings.len(), 7);
    }

    #[test]
    fn a_member_hides_no_module_name_and_refusals_forward_nothing() {
        // The template reads the instance, so a module constant spelled like
        // a data member leaves the member bound, as in Vue.
        let script = "const filters = []; export default { data: () => ({ filters }) }";
        let bindings = options_api(script, None).expect("bindings");
        assert_eq!(
            bindings.bindings.get("filters").copied(),
            Some(BindingType::Data)
        );
        assert!(options_api("export default { props: ['p'", None).is_none());
        assert!(options_api("export default {}", None).is_none());
        assert!(options_api("export default factory({ props: ['p'] })", None).is_none());
    }
}
