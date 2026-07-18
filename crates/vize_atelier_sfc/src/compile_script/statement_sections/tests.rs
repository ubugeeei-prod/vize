use super::extract_script_sections;

#[test]
fn skips_next_line_macro_assignment() {
    let content = r#"const props =
  defineProps<{
    name: string
  }>()

const count = 1
"#;
    let (_, setup, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(setup.as_slice(), ["const count = 1"]);
    assert!(module.is_empty());
}

#[test]
fn trims_leading_blank_lines_from_type_declarations() {
    let content = r#"import { useStore } from 'vuex'

interface RootState {
  count: number
}

const store = useStore<RootState>()
"#;
    let (_, _, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(
        module.as_slice(),
        ["interface RootState {\n  count: number\n}"]
    );
}

#[test]
fn hoists_only_static_runtime_enums() {
    let content = r#"const seed = runtime()

enum StaticStep { Name, General = 1 + 1 }
enum DynamicStep { Name = seed }
"#;
    let (_, setup, module) = extract_script_sections(content, false).unwrap();

    assert_eq!(
        module.as_slice(),
        ["enum StaticStep { Name, General = 1 + 1 }"]
    );
    assert_eq!(
        setup.as_slice(),
        ["const seed = runtime()", "enum DynamicStep { Name = seed }"]
    );
}

#[test]
fn skips_ecosystem_compile_time_macro() {
    let content = r#"definePage({
  name: 'home',
  meta: {
    requiresAuth: true,
  },
})

const msg = 'ready'
"#;
    let (_, setup, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(setup.as_slice(), ["const msg = 'ready'"]);
    assert!(module.is_empty());
}

#[test]
fn preserves_imported_define_page() {
    let content = r#"import { definePage } from '@/page.js'

definePage(() => ({
  title: 'runtime page',
}))

const msg = 'ready'
"#;
    let (imports, setup, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(imports.len(), 1);
    assert!(setup.iter().any(|line| line.contains("definePage")));
    assert!(setup.iter().any(|line| line.contains("const msg")));
    assert!(module.is_empty());
}

#[test]
fn skips_define_page_meta() {
    let content = r#"definePageMeta({
  name: 'docs',
  meta: {
    scrollMargin: 180,
  },
})

const msg = 'ready'
"#;
    let (_, setup, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(setup.as_slice(), ["const msg = 'ready'"]);
    assert!(module.is_empty());
}

#[test]
fn skips_define_route_rules() {
    let content = r#"defineRouteRules({
  prerender: true,
  cache: {
    maxAge: 60,
  },
})

const msg = 'ready'
"#;
    let (_, setup, module) = extract_script_sections(content, true).unwrap();

    assert_eq!(setup.as_slice(), ["const msg = 'ready'"]);
    assert!(module.is_empty());
}
