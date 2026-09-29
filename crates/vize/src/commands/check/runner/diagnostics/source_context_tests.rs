    use super::{binding_context, binding_context_from_token, utf16_column_to_byte_offset};

    fn context_at(line: &str, column: usize) -> Option<std::string::String> {
        binding_context(line, column).map(|context| context.to_string())
    }

    fn token_context(token: &str) -> Option<std::string::String> {
        binding_context_from_token(token).map(|context| context.to_string())
    }

    #[test]
    fn binding_context_falls_back_to_authored_bound_prop_on_template_lines() {
        assert_eq!(
            context_at(r#"<Child kind="num" :s="'bad'" />"#, 2).as_deref(),
            Some("'s'")
        );
        assert_eq!(
            context_at(
                r#"<Child :model-value="1" kind="num" :n="1" v-slot="{ count }">{{ count.toUpperCase() }}</Child>"#,
                73,
            )
            .as_deref(),
            None
        );
    }

    #[test]
    fn binding_context_excludes_interpolations_after_start_tag() {
        let event = r#"<div @click="onClick">{{ notDefined }}</div>"#;
        assert_eq!(context_at(event, event.find("notDefined").unwrap()), None);
        assert_eq!(
            context_at(event, event.find("onClick").unwrap()).as_deref(),
            Some("@click")
        );

        let prop = r#"<p :title="alsoMissing">{{ stillMissing }}</p>"#;
        assert_eq!(context_at(prop, prop.find("stillMissing").unwrap()), None);
        assert_eq!(
            context_at(prop, prop.find("alsoMissing").unwrap()).as_deref(),
            Some("'title'")
        );
    }

    #[test]
    fn binding_context_ignores_directive_like_text_inside_attribute_values() {
        let line = r#"<Child label="v-model:fake @save #slot" :value="bad" />"#;
        assert_eq!(
            context_at(line, line.find("bad").unwrap()).as_deref(),
            Some("'value'")
        );
    }

    #[test]
    fn binding_context_uses_static_prop_at_diagnostic_position_before_event() {
        let line = r#"<Child count="1" @change="onChange" />"#;
        assert_eq!(
            context_at(line, line.find("count").unwrap()).as_deref(),
            Some("count")
        );
        assert_eq!(
            context_at(line, line.find('1').unwrap()).as_deref(),
            Some("count")
        );
        assert_eq!(
            context_at(line, line.find("@change").unwrap()).as_deref(),
            Some("@change")
        );
        assert_eq!(
            context_at(line, line.find("onChange").unwrap()).as_deref(),
            Some("@change")
        );
    }

    #[test]
    fn binding_context_converts_utf16_columns_before_token_lookup() {
        let line = r#"<Child label="😀" :first="1" :second="bad" />"#;
        let second_byte = line.find(":second").unwrap();
        let second_utf16 = line[..second_byte]
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();

        assert_ne!(second_byte, second_utf16);
        assert_eq!(
            context_at(line, utf16_column_to_byte_offset(line, second_utf16)).as_deref(),
            Some("'second'")
        );
    }

    #[test]
    fn binding_context_maps_event_and_slot_directive_tokens() {
        assert_eq!(token_context("@save.once").as_deref(), Some("@save"));
        assert_eq!(token_context("v-on:submit").as_deref(), Some("@submit"));
        assert_eq!(token_context("#item").as_deref(), Some("#item"));
        assert_eq!(token_context("v-slot:default").as_deref(), Some("#default"));
        assert_eq!(token_context("v-slot").as_deref(), Some("#default"));
    }
