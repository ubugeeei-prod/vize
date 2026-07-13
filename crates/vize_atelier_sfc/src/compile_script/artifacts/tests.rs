use super::{erase_artifact_macro_statements, extract_macro_artifacts};

#[test]
fn extracts_define_page_artifact_module() {
    let content = r#"import { routeMeta } from './route'

definePage({
  name: 'home',
  meta: routeMeta,
})

const msg = 'ready'
"#;

    let artifacts = extract_macro_artifacts(content, 10);

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind.as_str(), "vue-router.definePage");
    assert_eq!(artifacts[0].name.as_str(), "definePage");
    assert!(artifacts[0].source.contains("definePage"));
    assert!(artifacts[0].content.contains("routeMeta"));
    assert_eq!(artifacts[0].start, 10 + content.find("definePage").unwrap());
    assert!(
        artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("import { routeMeta } from './route'\nexport default {")
    );
}

#[test]
fn extracts_define_page_meta_artifact_module() {
    let content = r#"import { pageAlias } from './route'

definePageMeta({
  name: 'docs',
  alias: pageAlias,
  meta: {
    scrollMargin: 180,
  },
})

const msg = 'ready'
"#;

    let artifacts = extract_macro_artifacts(content, 4);

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind.as_str(), "nuxt.definePageMeta");
    assert_eq!(artifacts[0].name.as_str(), "definePageMeta");
    assert!(artifacts[0].source.contains("definePageMeta"));
    assert!(artifacts[0].content.contains("scrollMargin"));
    assert_eq!(
        artifacts[0].start,
        4 + content.find("definePageMeta").unwrap()
    );
    assert!(
        artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("import { pageAlias } from './route'\nconst __nuxt_page_meta = {")
    );
    assert!(
        artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("export default __nuxt_page_meta")
    );
}

#[test]
fn extracts_define_page_meta_imported_from_typed_router() {
    let content = r#"import { definePageMeta } from '@typed-router'
import { pageAlias } from './route'

definePageMeta({
  name: 'docs',
  alias: pageAlias,
})

const msg = 'ready'
"#;

    let artifacts = extract_macro_artifacts(content, 0);

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind.as_str(), "nuxt.definePageMeta");
    assert_eq!(artifacts[0].name.as_str(), "definePageMeta");
    assert!(
        artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("import { pageAlias } from './route'\nconst __nuxt_page_meta = {")
    );
    assert!(
        !artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("@typed-router")
    );
}

#[test]
fn extracts_define_route_rules_artifact_module() {
    let content = r#"defineRouteRules({
  prerender: true,
  cache: {
    maxAge: 60,
  },
})

const msg = 'ready'
"#;

    let artifacts = extract_macro_artifacts(content, 2);

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind.as_str(), "nuxt.defineRouteRules");
    assert_eq!(artifacts[0].name.as_str(), "defineRouteRules");
    assert!(artifacts[0].source.contains("defineRouteRules"));
    assert!(artifacts[0].content.contains("prerender"));
    assert_eq!(
        artifacts[0].start,
        2 + content.find("defineRouteRules").unwrap()
    );
    assert!(
        artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .starts_with("export default {")
    );
}

#[test]
fn ignores_content_without_artifact_macro_candidates() {
    let content = r#"const msg = 'ready'
const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'visible',
  () => import('./components/MyComponent.vue'),
)
"#;

    assert!(extract_macro_artifacts(content, 0).is_empty());
    assert!(erase_artifact_macro_statements(content).is_none());
}

#[test]
fn preserves_imported_define_page_runtime_call() {
    let content = r#"import { definePage } from '@/page.js'

definePage(() => ({
  title: 'runtime page',
}))

const msg = 'ready'
"#;

    assert!(extract_macro_artifacts(content, 0).is_empty());
    assert!(erase_artifact_macro_statements(content).is_none());
}

#[test]
fn erases_define_page_top_level_statement() {
    let content = r#"definePage({ name: 'home' })
const msg = 'ready'
"#;

    let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

    assert!(!erased.contains("definePage"));
    assert!(erased.contains("const msg = 'ready'"));
}

#[test]
fn erases_define_page_meta_top_level_statement() {
    let content = r#"definePageMeta({ name: 'docs' })
const msg = 'ready'
"#;

    let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

    assert!(!erased.contains("definePageMeta"));
    assert!(erased.contains("const msg = 'ready'"));
}

#[test]
fn erases_typed_router_macro_import_and_call() {
    let content = r#"import { definePageMeta } from '@typed-router'

definePageMeta({ name: 'docs' })
const msg = 'ready'
"#;

    let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

    assert!(!erased.contains("definePageMeta"));
    assert!(!erased.contains("@typed-router"));
    assert!(erased.contains("const msg = 'ready'"));
}

#[test]
fn erases_define_route_rules_top_level_statement() {
    let content = r#"defineRouteRules({ prerender: true })
const msg = 'ready'
"#;

    let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

    assert!(!erased.contains("defineRouteRules"));
    assert!(erased.contains("const msg = 'ready'"));
}
