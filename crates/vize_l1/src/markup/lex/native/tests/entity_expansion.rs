use super::{TokenEvent, tokenize};
use crate::markup::entity::DecodedEntity;

#[test]
fn named_entity_emits_every_scalar_with_the_source_reference_span() {
    let source = "<div data='&fjlig;'>&fjlig;</div>";
    let cb = tokenize(source);
    let attr_start = source.find("&fjlig;").unwrap();
    let text_start = source.rfind("&fjlig;").unwrap();
    assert_eq!(
        cb.attrib_entity_values.as_slice(),
        &[(DecodedEntity::Named("fj"), attr_start, attr_start + 7)]
    );
    assert_eq!(
        cb.text_entity_values.as_slice(),
        &[(DecodedEntity::Named("fj"), text_start, text_start + 7)]
    );
    for ch in ['f', 'j'] {
        assert!(
            cb.events
                .contains(&TokenEvent::AttribEntity(ch, attr_start, attr_start + 7))
        );
        assert!(
            cb.events
                .contains(&TokenEvent::TextEntity(ch, text_start, text_start + 7))
        );
    }
}
