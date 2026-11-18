use crate::linter::{LintResult, Linter};

fn lint_tsrx(linter: &Linter, source: &str, filename: &str) -> LintResult {
    linter.lint_tsrx(source, filename)
}

fn rows(result: &LintResult) -> Vec<(&str, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn regions_merge_in_source_order_with_one_report_per_rule_and_range() {
    // The `<li>` region is the `.map()` loop's region in the `<ul>` region,
    // so the loop rules see it.
    let source = "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    {items.map((item) => <li  title={item}><img src={item} /></li>)}
  </ul>
}

export function Footer() @{
  <footer  class=\"footer\">done</footer>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    assert_eq!(
        rows(&result),
        [
            ("vue/no-multi-spaces", 61, 63),
            ("vue/require-v-for-key", 81, 145),
            ("vue/no-multi-spaces", 105, 107),
            ("a11y/alt-text", 120, 138),
            ("vue/no-multi-spaces", 194, 196),
        ]
    );
    assert_eq!((result.error_count, result.warning_count), (1, 4));
}

#[test]
fn conditional_arms_are_linted_as_branches_of_the_enclosing_region() {
    let source = "export function Avatar({ ok, src }: { ok: boolean; src: string }) @{
  <p>{ok ? <img  src={src} /> : null}</p>
}
";
    let result = lint_tsrx(&Linter::new(), source, "Avatar.tsrx");
    assert_eq!(
        rows(&result),
        [("a11y/alt-text", 80, 98), ("vue/no-multi-spaces", 84, 86)]
    );
}

#[test]
fn value_and_regions_are_linted_inside_the_enclosing_region() {
    // `{n && …}` with a non-boolean `n` is a `ui.and` in the `<ul>` region.
    let source = "export function Avatar({ n, src }: { n: number; src: string }) @{
  <ul>{n && <li><img  src={src} /></li>}{n && <li>{n}</li>}</ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "Avatar.tsrx");
    assert_eq!(
        rows(&result),
        [("a11y/alt-text", 82, 100), ("vue/no-multi-spaces", 86, 88)]
    );
}

#[test]
fn a_module_with_an_unsupported_construct_reports_every_one_and_lints_no_region() {
    let source = "export function A() @{
  <div  class=\"a\">
    @try {
      <p>a</p>
    } @catch (error) {
      <p>b</p>
    }
  </div>
}
";
    let result = lint_tsrx(&Linter::new(), source, "A.tsrx");
    assert_eq!(rows(&result), [("parser/tsrx", 46, 111)]);
    assert_eq!(
        result.diagnostics[0].message,
        "Try is not supported in TSRX modules yet, so this module's markup is not linted"
    );
    assert_eq!((result.error_count, result.warning_count), (1, 0));
}

#[test]
fn a_keyed_host_loop_lints_its_items_and_its_empty_region() {
    // The head's `key` keys every item; `@empty` renders outside the list.
    let source = "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    @for (const item of items; key item) {
      <li  title={item}><img src={item} /></li>
    } @empty {
      <li  class=\"none\"><img src=\"none.png\" /></li>
    }
  </ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    let at = |text: &str| u32::try_from(source.find(text).unwrap()).unwrap();
    assert_eq!(
        rows(&result),
        [
            ("vue/no-multi-spaces", 61, 63),
            ("vue/no-multi-spaces", 129, 131),
            ("a11y/alt-text", 144, 162),
            ("vue/no-multi-spaces", 192, 194),
            ("a11y/alt-text", 207, 229),
        ]
    );
    assert_eq!(at("<ul  ") + 3, 61);
    assert_eq!(at("<li  title") + 3, 129);
    assert_eq!(at("<li  class") + 3, 192);
    assert_eq!(at("<img src=\"none.png\""), 207);
}

#[test]
fn an_unkeyed_host_loop_needs_a_key_on_its_items() {
    let source = "export function List({ items }: { items: string[] }) @{
  <ul>
    @for (const item of items) {
      <li>{item}</li>
    }
  </ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    // Reported at the loop, as for a JSX `.map()` list.
    let start = u32::try_from(source.find("@for").unwrap()).unwrap();
    let end = u32::try_from(source.find("\n  </ul>").unwrap()).unwrap();
    assert_eq!(rows(&result), [("vue/require-v-for-key", start, end)]);
    assert_eq!((start, end), (67, 123));
}

fn linter_with(rules: Vec<Box<dyn crate::rule::Rule>>) -> Linter {
    let mut registry = crate::rule::RuleRegistry::new();
    for rule in rules {
        registry.register(rule);
    }
    Linter::with_registry(registry)
}

#[test]
fn image_name_rules_report_on_tsrx_markup() {
    // Moved off the JSX lowering fallback (#6848), so they now lint TSRX too.
    let source = "export function Badge({ src }: { src: string }) @{
  <p><span role=\"img\"></span><img src={src} /><img src={src} alt=\"\" role=\"img\" /></p>
}
";
    let linter = linter_with(vec![
        Box::new(crate::rules::a11y::RoleImgHasName),
        Box::new(crate::rules::vue::A11yImgAlt),
    ]);
    let result = lint_tsrx(&linter, source, "Badge.tsrx");
    let span = |text: &str| {
        let start = u32::try_from(source.find(text).unwrap()).unwrap();
        (start, start + u32::try_from(text.len()).unwrap())
    };
    let role = span("<span role=\"img\"></span>");
    let bare = span("<img src={src} />");
    let empty = span("<img src={src} alt=\"\" role=\"img\" />");
    assert_eq!(
        rows(&result),
        [
            ("a11y/role-img-has-name", role.0, role.1),
            ("vue/a11y-img-alt", bare.0, bare.1),
            ("a11y/role-img-has-name", empty.0, empty.1),
        ]
    );
}

#[test]
fn landmark_roles_report_on_tsrx_and_respect_exclusive_regions() {
    // Conditional arms and a host loop's `@empty` region never render together.
    let source = "export function Page({ ok, items }: { ok: boolean; items: string[] }) @{
  <div>
    <nav>one</nav>
    <nav>two</nav>
    {ok ? <main>a</main> : <main>b</main>}
    @for (const item of items; key item) {
      <aside>{item}</aside>
    } @empty {
      <aside>none</aside>
    }
  </div>
}
";
    let linter = linter_with(vec![Box::new(crate::rules::a11y::LandmarkRoles)]);
    let result = lint_tsrx(&linter, source, "Page.tsrx");
    let span = |text: &str| {
        let start = u32::try_from(source.find(text).unwrap()).unwrap();
        (start, start + u32::try_from(text.len()).unwrap())
    };
    let one = span("<nav>one</nav>");
    let two = span("<nav>two</nav>");
    assert_eq!(
        rows(&result),
        [
            ("a11y/landmark-roles", one.0, one.1),
            ("a11y/landmark-roles", two.0, two.1),
        ]
    );

    let duplicate = "export function Page() @{
  <div><main>a</main><main>b</main></div>
}
";
    let result = lint_tsrx(&linter, duplicate, "Page.tsrx");
    let start = u32::try_from(duplicate.find("<main>b</main>").unwrap()).unwrap();
    assert_eq!(rows(&result), [("a11y/landmark-roles", start, start + 14)]);
}

#[test]
fn id_duplication_reports_on_tsrx_and_respects_exclusive_regions() {
    // Moved off the JSX lowering fallback (#6848), so it now lints TSRX too.
    let source = "export function Rows({ ok, items }: { ok: boolean; items: string[] }) @{
  <div>
    <p id=\"a\">a</p>
    {ok ? <i id=\"b\">x</i> : <i id=\"b\">y</i>}
    @for (const item of items; key item) {
      <span id=\"c\">{item}</span>
    } @empty {
      <span id=\"c\">none</span>
    }
    <b id=\"a\">again</b>
  </div>
}
";
    let linter = linter_with(vec![Box::new(crate::rules::html::IdDuplication)]);
    let result = lint_tsrx(&linter, source, "Rows.tsrx");
    let first = u32::try_from(source.find("id=\"a\">a<").unwrap()).unwrap();
    let again = u32::try_from(source.find("id=\"a\">again").unwrap()).unwrap();
    assert_eq!(rows(&result), [("html/id-duplication", again, again + 6)]);
    let label = &result.diagnostics[0].labels[0];
    assert_eq!((label.start, label.end), (first, first + 6));
}

#[test]
fn vue_markup_rules_report_on_tsrx_markup() {
    // Moved off the JSX lowering fallback (#6848), so they now lint TSRX too.
    let source = "export function Shell() @{
  <section>
    <template><p>lone</p></template>
    <div inline-template=\"x\">old</div>
  </section>
}
";
    let linter = linter_with(vec![
        Box::new(crate::rules::vue::NoLoneTemplate),
        Box::new(crate::rules::vapor::NoInlineTemplate),
    ]);
    let result = lint_tsrx(&linter, source, "Shell.tsrx");
    let at = |text: &str| u32::try_from(source.find(text).unwrap()).unwrap();
    let template = at("<template>");
    let inline = at("inline-template");
    assert_eq!(
        rows(&result),
        [
            ("vue/no-lone-template", template, template + 32),
            ("vapor/no-inline-template", inline, inline + 19),
        ]
    );
}
