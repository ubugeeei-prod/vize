//! A packed library must preserve public literal types containing private names.
use super::*;

#[test]
fn packed_declarations_preserve_marker_literals_for_ts_and_tsx_consumers() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let library = Case::new("marker-literal-library");
    library.link_real_vue();
    library.write(
        "tsconfig.json",
        include_str!("../fixtures/declaration_marker_text/tsconfig.json"),
    );
    library.write(
        "src/Button.vue",
        include_str!("../fixtures/declaration_marker_text/Button.vue"),
    );
    let emitted = library.run(
        &corsa_path,
        &[
            "check",
            "--format",
            "json",
            "--declaration",
            "--declaration-dir",
            "types",
        ],
    );
    assert_eq!(
        (emitted.status, parse_json(&emitted)["errorCount"].as_u64()),
        (Some(0), Some(0)),
        "{}\n{}",
        emitted.stdout,
        emitted.stderr
    );

    let consumer = Case::new("marker-literal-consumer");
    consumer.link_real_vue();
    let package = consumer.root.join("node_modules/packed-button");
    std::fs::create_dir_all(&package).unwrap();
    for entry in std::fs::read_dir(library.root.join("types")).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), package.join(entry.file_name())).unwrap();
    }
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"packed-button","types":"./Button.vue.d.ts"}"#,
    )
    .unwrap();
    // No source-workspace link or surviving library source may satisfy imports.
    drop(library);
    consumer.write("tsconfig.json", r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true, "skipLibCheck": false, "jsx": "preserve", "jsxImportSource": "vue", "types": ["vue/jsx"] },
  "include": ["src/**/*"]
}"#);
    let clean_ts = "import type { PublicProps } from \"packed-button\";\ntype Label = PublicProps[\"label\"];\nconst label: Label = \"__vizeComponentMarker\";\nvoid label;\n";
    let broken_ts = "import type { PublicProps } from \"packed-button\";\nconst props: PublicProps = { label: \"wrong\" };\nvoid props;\n";
    let clean_tsx = "import Button from \"packed-button\";\nconst node = <Button label=\"__vizeComponentMarker\" />;\nvoid node;\n";
    let broken_tsx = "import Button from \"packed-button\";\nconst node = <Button label=\"wrong\" />;\nvoid node;\n";
    consumer.write("src/index.ts", clean_ts);
    consumer.write("src/consumer.tsx", clean_tsx);
    assert_clean(&consumer.tsc(&corsa_path));

    for (path, broken, clean, expected) in [
        (
            "src/index.ts",
            broken_ts,
            clean_ts,
            "src/index.ts(2,30): error TS2322: Type '\"wrong\"' is not assignable to type '\"__vizeComponentMarker\"'.\n",
        ),
        (
            "src/consumer.tsx",
            broken_tsx,
            clean_tsx,
            "src/consumer.tsx(2,22): error TS2322: Type '\"wrong\"' is not assignable to type '\"__vizeComponentMarker\"'.\n",
        ),
    ] {
        consumer.write(path, broken);
        let result = consumer.tsc(&corsa_path);
        assert_eq!(
            (
                result.status,
                result.stdout.as_str(),
                result.stderr.as_str()
            ),
            (Some(1), expected, "")
        );
        consumer.write(path, clean);
        assert_clean(&consumer.tsc(&corsa_path));
    }
}

fn assert_clean(result: &RunResult) {
    assert_eq!(
        (
            result.status,
            result.stdout.as_str(),
            result.stderr.as_str()
        ),
        (Some(0), "", "")
    );
}
