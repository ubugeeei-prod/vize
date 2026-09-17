use super::{project, write};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

#[test]
fn component_props_in_match_scopes_reject_wrong_types_and_recover() {
    let project = project();
    let cases = [
        (
            "Root",
            r#"<template v-match="result"><Child v-when="{ kind: 'ok', const rows }" :count="rows[0]"/><p v-when="_"/></template>"#,
        ),
        (
            "RootLoop",
            r#"<template v-match="result"><template v-when="{ kind: 'ok', const rows }"><Child v-for="rows in rows" :count="rows"/></template><p v-when="_"/></template>"#,
        ),
        (
            "Nested",
            r#"<template><div v-match="result"><Child v-when="{ kind: 'ok', const rows }" :count="rows[0]"/><p v-when="_"/></div></template>"#,
        ),
        (
            "ParentLoop",
            r#"<template><template v-for="result in [result]"><div v-match="result"><Child v-when="{ kind: 'ok', const rows }" :count="rows[0]"/><p v-when="_"/></div></template></template>"#,
        ),
        (
            "ParentSlot",
            r#"<template><Host v-slot="{ result }"><div v-match="result"><Child v-when="{ kind: 'ok', const rows }" :count="rows[0]"/><p v-when="_"/></div></Host></template>"#,
        ),
    ];
    for wrong_type in [true, false] {
        for (name, template) in cases {
            let template = if wrong_type {
                template
                    .replace(":count=\"rows[0]\"", ":count=\"rows[0].toString()\"")
                    .replace(":count=\"rows\"", ":count=\"rows.toString()\"")
            } else {
                template.into()
            };
            write(
                project.path(),
                &vize_s0::cstr!("src/{name}.vue"),
                &vize_s0::cstr!(
                    "<script setup lang=\"ts\">\nimport type {{ Result }} from './types';\nimport Child from './Child.vue';\nconst result = {{}} as Result<number>;\nconst Host = {{}} as {{ readonly __vizeSlots?: {{ default: (props: {{ result: Result<number> }}) => unknown }} }};\n</script>\n{template}"
                ),
            );
        }
        let mut checker = BatchTypeChecker::new(project.path()).unwrap();
        checker.set_experimental_patterned_template(true);
        checker.scan_project().unwrap();
        let result = checker.check_project().unwrap();
        assert_eq!(result.success, !wrong_type, "{result:#?}");
        let mut actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.file.file_name().unwrap().to_str().unwrap(),
                    diagnostic.code,
                    diagnostic.severity,
                    diagnostic.line,
                )
            })
            .collect();
        actual.sort_unstable();
        let expected = if wrong_type {
            vec![
                ("Nested.vue", Some(2322), 1, 6),
                ("ParentLoop.vue", Some(2322), 1, 6),
                ("ParentSlot.vue", Some(2322), 1, 6),
                ("Root.vue", Some(2322), 1, 6),
                ("RootLoop.vue", Some(2322), 1, 6),
            ]
        } else {
            vec![]
        };
        assert_eq!(actual, expected, "{result:#?}");
    }
}
