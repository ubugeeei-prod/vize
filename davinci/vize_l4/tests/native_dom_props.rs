//! Real Component/File factories preserve literal property source windows.

#[path = "native_dom_file/support.rs"]
mod support;

use vize_l0::Allocator;
use vize_l1_to_l2::vue_file::VueFileProducer;
use vize_l2::lang::js::ProgramInput;
use vize_l3::decision::build_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit_file,
    write::{NoLinks, Recorded},
};

#[test]
fn complete_file_literal_property_modules_keep_comments_trivia_and_authentic_owners() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/dom-props-vue-3.5.35.json")).unwrap();
    let mut captured = Vec::new();
    for reference in pack["fixtures"].as_array().unwrap() {
        if reference["family"] != "file" {
            continue;
        }
        let arena = Allocator::default();
        let source = reference["source"].as_str().unwrap();
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        let (syntax, block) = support::script(&arena, source, reference["setup"].as_str().unwrap());
        producer
            .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
            .unwrap();
        let native = support::construct(
            &arena,
            source,
            reference["template"].as_str().unwrap(),
            &mut producer,
        );
        let vue = producer.finish().unwrap();
        let analysis = build_dom_file_decisions(vue.file()).unwrap();
        let recorded = assemble_template(
            emit_file::<Recorded>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit_file::<NoLinks>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text.as_str(),
            reference["code"].as_str().unwrap(),
            "{}",
            reference["id"]
        );
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        let embed = native.embeds.first().unwrap();
        let row = analysis
            .dom()
            .unwrap()
            .file_expression(embed.node.unwrap())
            .unwrap();
        let resolution = row.resolution();
        assert!(core::ptr::eq(resolution.file(), vue.file()));
        let table = resolution.table().unwrap();
        assert!(table.occurrences().is_empty());
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert!(document.links().iter().any(|link| {
            link.authored == table.expression().span
                && link.name.is_none()
                && document
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    == Some(table.expression().source)
        }));
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("PropsDom.vue", source)).unwrap();
        assert_eq!(map["names"], serde_json::json!([]));
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(
            table.expression().source,
            reference["expression"].as_str().unwrap()
        );
        captured.push(serde_json::json!({"id": reference["id"], "source": source, "code": document.as_str(), "map": map}));
    }
    assert_eq!(captured.len(), 7);
    if let Ok(path) = std::env::var("VIZE_L4_PROPS_FILE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}
