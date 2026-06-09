use super::{
    LineIndex, SemanticTokensService, TokenModifier, TokenType, encoding::offset_to_line_col,
    expressions, template,
};
use tower_lsp::lsp_types::{
    Position, Range, SemanticToken, SemanticTokensRangeResult, SemanticTokensResult,
};

#[derive(Debug)]
struct DecodedToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}

fn decode_tokens(tokens: &[SemanticToken]) -> Vec<DecodedToken> {
    let mut decoded = Vec::with_capacity(tokens.len());
    let mut line = 0;
    let mut start = 0;

    for token in tokens {
        line += token.delta_line;
        if token.delta_line == 0 {
            start += token.delta_start;
        } else {
            start = token.delta_start;
        }

        decoded.push(DecodedToken {
            line,
            start,
            length: token.length,
            token_type: token.token_type,
        });
    }

    decoded
}

fn has_token_text(
    template_str: &str,
    tokens: &[super::types::AbsoluteToken],
    token_type: TokenType,
    text: &str,
) -> bool {
    let Some(start) = template_str.find(text) else {
        return false;
    };

    tokens.iter().any(|token| {
        token.line == 0
            && token.start == start as u32
            && token.length == text.len() as u32
            && token.token_type == token_type as u32
    })
}

#[test]
fn test_extract_identifiers() {
    let expr = "count + message.length";
    let idents = expressions::extract_identifiers(expr);
    assert_eq!(idents.len(), 3);
    assert_eq!(idents[0].0, "count");
    assert_eq!(idents[1].0, "message");
    assert_eq!(idents[2].0, "length");
}

#[test]
fn test_looks_like_function_call() {
    let expr = "handleClick()";
    assert!(expressions::looks_like_function_call(expr, 0));

    let expr = "count + 1";
    assert!(!expressions::looks_like_function_call(expr, 0));
}

#[test]
fn test_offset_to_line_col() {
    let source = "abc\ndef\nghi";
    assert_eq!(offset_to_line_col(source, 0), (0, 0));
    assert_eq!(offset_to_line_col(source, 4), (1, 0));
    assert_eq!(offset_to_line_col(source, 8), (2, 0));
}

#[test]
fn test_offset_to_line_col_counts_utf16_code_units() {
    let source = "const icon = \"😀\"; missing";
    let offset = source.find("missing").unwrap();

    assert_eq!(offset_to_line_col(source, offset), (0, 19));
}

#[test]
fn test_token_modifier_encode() {
    let modifiers = vec![TokenModifier::Declaration, TokenModifier::Readonly];
    let encoded = TokenModifier::encode(&modifiers);
    assert_eq!(encoded, 0b101); // bits 0 and 2
}

#[test]
fn test_art_tokens_basic() {
    let content = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>

<script setup>
import Button from './Button.vue'
</script>"#;

    let uri = tower_lsp::lsp_types::Url::parse("file:///test.art.vue").unwrap();
    let result = SemanticTokensService::get_tokens(content, &uri);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        assert!(!tokens.data.is_empty());
    }
}

#[test]
fn test_art_block_tokens() {
    let content = "<art title=\"Test\">\n</art>";
    let mut tokens = Vec::new();
    let line_index = LineIndex::new(content);
    SemanticTokensService::collect_art_block_tokens(content, &mut tokens, &line_index);

    // Should find <art and </art>
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].length, 4); // "<art"
    assert_eq!(tokens[1].length, 6); // "</art>"
}

#[test]
fn test_variant_block_tokens() {
    let content = "<variant name=\"Primary\">\n</variant>";
    let mut tokens = Vec::new();
    let line_index = LineIndex::new(content);
    SemanticTokensService::collect_variant_block_tokens(content, &mut tokens, &line_index);

    // Should find <variant and </variant>
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].length, 8); // "<variant"
    assert_eq!(tokens[1].length, 10); // "</variant>"
}

#[test]
fn test_art_attribute_tokens() {
    let content = r#"<art title="Button" component="./Button.vue">"#;
    let mut tokens = Vec::new();
    let line_index = LineIndex::new(content);
    SemanticTokensService::collect_art_attribute_tokens(content, &mut tokens, &line_index);

    // Should find title, "Button", component, "./Button.vue"
    assert!(tokens.len() >= 4);
}

#[test]
fn test_art_variant_template_tokens() {
    let content = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="label" @click="handleClick">{{ label }}</Button>
  </variant>
</art>"#;
    let mut tokens = Vec::new();
    let line_index = LineIndex::new(content);
    SemanticTokensService::collect_art_variant_template_tokens(content, &mut tokens, &line_index);

    assert!(
        tokens
            .iter()
            .any(|token| token.line == 2 && token.token_type == TokenType::Property as u32),
        "{tokens:#?}"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.line == 2 && token.token_type == TokenType::Event as u32),
        "{tokens:#?}"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.line == 2 && token.token_type == TokenType::Variable as u32),
        "{tokens:#?}"
    );
}

#[test]
fn test_art_script_tokens() {
    let content = r#"<script setup>
import Button from './Button.vue'
</script>"#;
    let mut tokens = Vec::new();
    let line_index = LineIndex::new(content);
    SemanticTokensService::collect_art_script_tokens(content, &mut tokens, &line_index);

    // Should find import, from, and string literal
    assert!(tokens.len() >= 3);
}

#[test]
fn test_interpolation_tokens() {
    let template_str = "  {{ message }}";
    let mut tokens = Vec::new();
    template::collect_interpolation_tokens(template_str, 1, &mut tokens);

    // Should find 'message' as a variable
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::Variable as u32);
    assert_eq!(tokens[0].length, 7); // "message"
}

#[test]
fn test_interpolation_string_token_uses_utf16_length() {
    let template_str = "  {{ \"😀\" }}";
    let mut tokens = Vec::new();
    template::collect_interpolation_tokens(template_str, 1, &mut tokens);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::String as u32);
    assert_eq!(tokens[0].length, 4);
}

#[test]
fn test_template_semantic_tokens_ignore_plain_text_lookalikes() {
    let template_str = "<div>email dev@example.com and text v-if :class @click</div>";
    let mut tokens = Vec::new();
    template::collect_template_tokens(template_str, 0, &mut tokens);

    assert!(tokens.is_empty(), "{tokens:#?}");
}

#[test]
fn test_template_semantic_tokens_ignore_static_attribute_text() {
    let template_str = r#"<div title="plain v-if @click :class"></div>"#;
    let mut tokens = Vec::new();
    template::collect_template_tokens(template_str, 0, &mut tokens);

    assert!(tokens.is_empty(), "{tokens:#?}");
}

#[test]
fn test_directive_expression_does_not_steal_next_attribute_value() {
    let template_str = r#"<div v-else title="message"></div>"#;
    let mut tokens = Vec::new();
    template::collect_directive_expression_tokens(template_str, 0, &mut tokens);

    assert!(tokens.is_empty(), "{tokens:#?}");
}

#[test]
fn test_template_semantic_tokens_still_collect_attribute_tokens() {
    let template_str = r#"<div v-if="ready" @click="save" :class="classes"></div>"#;
    let mut tokens = Vec::new();
    template::collect_template_tokens(template_str, 0, &mut tokens);

    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Keyword as u32),
        "{tokens:#?}"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Event as u32),
        "{tokens:#?}"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Property as u32),
        "{tokens:#?}"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Variable as u32),
        "{tokens:#?}"
    );
}

#[test]
fn test_template_semantic_tokens_collect_dynamic_shorthand_args() {
    let template_str = r#"<button @[eventName].stop="run" :[propName].camel="value"></button>"#;
    let mut tokens = Vec::new();
    template::collect_template_tokens(template_str, 0, &mut tokens);

    assert!(
        has_token_text(template_str, &tokens, TokenType::Event, "@[eventName].stop"),
        "{tokens:#?}"
    );
    assert!(
        has_token_text(template_str, &tokens, TokenType::Property, ":[propName]"),
        "{tokens:#?}"
    );
    for name in ["eventName", "propName", "run", "value"] {
        assert!(
            has_token_text(template_str, &tokens, TokenType::Variable, name),
            "missing {name}: {tokens:#?}"
        );
    }
}

#[test]
fn test_template_semantic_tokens_collect_unquoted_directive_values() {
    let template_str = r#"<div v-if=ready @click=save :class=classes></div>"#;
    let mut tokens = Vec::new();
    template::collect_template_tokens(template_str, 0, &mut tokens);

    for name in ["ready", "save", "classes"] {
        assert!(
            has_token_text(template_str, &tokens, TokenType::Variable, name),
            "missing {name}: {tokens:#?}"
        );
    }
}

#[test]
fn test_full_sfc_semantic_tokens() {
    let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const count = ref(0)
</script>
"#;

    let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
    let result = SemanticTokensService::get_tokens(content, &uri);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should have tokens for:
        // - 'count' in template interpolation
        // - 'ref' in script
        assert!(!tokens.data.is_empty(), "Should have semantic tokens");
    }
}

#[test]
fn test_full_sfc_semantic_tokens_use_lsp_coordinates() {
    let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const icon = "😀"
const count = ref(icon)
</script>
"#;

    let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
    let result = SemanticTokensService::get_tokens(content, &uri);
    let Some(SemanticTokensResult::Tokens(tokens)) = result else {
        panic!("expected semantic tokens");
    };
    let decoded = decode_tokens(&tokens.data);

    assert!(
        decoded.iter().any(|token| {
            token.line == 1
                && token.start == 10
                && token.length == "count".len() as u32
                && token.token_type == TokenType::Variable as u32
        }),
        "{decoded:#?}"
    );
    assert!(
        decoded.iter().any(|token| {
            token.line == 6
                && token.start == 14
                && token.length == "ref".len() as u32
                && token.token_type == TokenType::Function as u32
        }),
        "{decoded:#?}"
    );
}

#[test]
fn test_range_semantic_tokens_return_only_requested_lines() {
    let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const count = ref(0)
</script>
"#;

    let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
    let result = SemanticTokensService::get_tokens_range(
        content,
        &uri,
        Range {
            start: Position {
                line: 5,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 0,
            },
        },
    );
    let Some(SemanticTokensRangeResult::Tokens(tokens)) = result else {
        panic!("expected range semantic tokens");
    };
    let decoded = decode_tokens(&tokens.data);

    assert!(!decoded.is_empty());
    assert!(decoded.iter().all(|token| token.line == 5), "{decoded:#?}");
    assert!(
        decoded
            .iter()
            .any(|token| token.start == 14 && token.token_type == TokenType::Function as u32),
        "{decoded:#?}"
    );
}

#[test]
fn test_directive_expression_tokenization() {
    let template_str =
        r#"<div v-if="todoGuards.isActive(todo) || todoGuards.isCompleted(todo)"></div>"#;
    let mut tokens = Vec::new();
    template::collect_directive_expression_tokens(template_str, 1, &mut tokens);

    // Debug: print all tokens
    for token in &tokens {
        eprintln!(
            "Token: line={}, start={}, length={}, type={}",
            token.line, token.start, token.length, token.token_type
        );
    }

    // Should find tokens for the expression:
    // - todoGuards (variable)
    // - isActive (function)
    // - todo (variable)
    // - || (operator)
    // - todoGuards (variable)
    // - isCompleted (function)
    // - todo (variable)
    assert!(
        tokens.len() >= 7,
        "Expected at least 7 tokens, got {}",
        tokens.len()
    );

    // Check that we have both variable and function tokens
    let has_variable = tokens
        .iter()
        .any(|t| t.token_type == TokenType::Variable as u32);
    let has_function = tokens
        .iter()
        .any(|t| t.token_type == TokenType::Function as u32);
    let has_operator = tokens
        .iter()
        .any(|t| t.token_type == TokenType::Operator as u32);

    assert!(has_variable, "Should have variable tokens");
    assert!(has_function, "Should have function tokens");
    assert!(has_operator, "Should have operator tokens");
}

#[test]
fn test_tokenize_expression() {
    let expr = "todoGuards.isActive(todo) || todoGuards.isCompleted(todo)";
    let template_str = expr; // Use the expression as the "template" for position calculation
    let mut tokens = Vec::new();
    expressions::tokenize_expression(expr, template_str, 0, 1, &mut tokens);

    // Debug: print all tokens
    for token in &tokens {
        let token_name = match token.token_type {
            x if x == TokenType::Variable as u32 => "Variable",
            x if x == TokenType::Function as u32 => "Function",
            x if x == TokenType::Property as u32 => "Property",
            x if x == TokenType::Operator as u32 => "Operator",
            x if x == TokenType::Keyword as u32 => "Keyword",
            x if x == TokenType::Number as u32 => "Number",
            x if x == TokenType::String as u32 => "String",
            _ => "Unknown",
        };
        eprintln!(
            "Token: start={}, length={}, type={} ({})",
            token.start, token.length, token.token_type, token_name
        );
    }

    // Count token types
    let var_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Variable as u32)
        .count();
    let func_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Function as u32)
        .count();
    let prop_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Property as u32)
        .count();
    let op_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Operator as u32)
        .count();

    eprintln!(
        "Variables: {}, Functions: {}, Properties: {}, Operators: {}",
        var_count, func_count, prop_count, op_count
    );

    // We expect:
    // - todoGuards (variable) x2
    // - isActive (function) - after dot, so might be property
    // - isCompleted (function) - after dot, so might be property
    // - todo (variable) x2
    // - || (operator)
    assert!(tokens.len() >= 7, "Expected at least 7 tokens");
}

#[test]
fn test_inline_art_tokens_in_vue() {
    let content = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>"#;

    let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
    let result = SemanticTokensService::get_tokens(content, &uri);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        assert!(!tokens.data.is_empty(), "Should have inline art tokens");

        // Verify we have enough tokens (at least art/variant tags + attributes)
        assert!(
            tokens.data.len() >= 4,
            "Expected at least 4 tokens for inline art, got {}",
            tokens.data.len()
        );
    }
}
