#!/usr/bin/env bash

set -euo pipefail

readonly graph_crates=(vize_atelier_dom vize_atelier_ssr vize_atelier_vapor)

for crate in "${graph_crates[@]}"; do
  cargo check -p "$crate" --no-default-features --features graph

  resolved="$({
    cargo tree \
      -p "$crate" \
      --no-default-features \
      --features graph \
      --edges normal \
      --prefix none \
      --format '{p}'
  })"

  internal="$({
    printf '%s\n' "$resolved" \
      | awk '$1 ~ /^vize_/ { print $1 }' \
      | sort -u
  })"
  expected="$({
    printf '%s\n' "$crate" vize_atlas vize_carton vize_rendu \
      | sort -u
  })"

  if [[ "$internal" != "$expected" ]]; then
    printf 'graph-only dependency boundary failed for %s\n' "$crate" >&2
    printf 'expected internal crates:\n%s\n' "$expected" >&2
    printf 'resolved internal crates:\n%s\n' "$internal" >&2
    exit 1
  fi

  forbidden="$({
    printf '%s\n' "$resolved" \
      | awk '$1 ~ /^(vize_(armature|atelier_core|croquis|relief)|oxc_)/ { print $1 }' \
      | sort -u
  })"
  if [[ -n "$forbidden" ]]; then
    printf 'graph-only dependency tree for %s contains frontend crates:\n%s\n' \
      "$crate" "$forbidden" >&2
    exit 1
  fi
done

# Atelier Core still exposes legacy root re-exports for downstream source
# compatibility. Production workspace consumers must name the owning crate,
# so the compatibility facade cannot quietly become the architecture again.
readonly owned_modules='errors|options|parser|tokenizer'
readonly owned_symbols='Allocator|AllocBox|AllocVec|CloneIn|Parser|parse|parse_with_options|parse_with_options_and_invalid_html_self_closing|parse_with_options_and_template_syntax|CompilerError|CompilerResult|ErrorCode|BindingMetadata|BindingType|CodegenMode|CodegenOptions|CompilerOptions|ParseMode|ParserOptions|TemplateSyntaxMode|TextMode|TransformOptions|WhitespaceStrategy|AttributeNode|CommentNode|CompoundExpressionChild|CompoundExpressionNode|ConstantType|DirectiveNode|ElementNode|ElementType|ExpressionNode|ForNode|ForParseResult|IfBranchNode|IfNode|ImportItem|InterpolationNode|JsExpression|Namespace|NodeType|Position|PropNode|RootNode|RuntimeHelper|SimpleExpressionNode|SourceLocation|TemplateChildNode|TextCallContent|TextCallNode|TextNode'

core_facade_routes="$({
  rg -n -U \
    --glob 'crates/*/src/**/*.rs' \
    --glob '!crates/vize_atelier_core/**' \
    "vize_atelier_core::(${owned_modules})\\b|vize_atelier_core::(${owned_symbols})\\b|use vize_atelier_core::\\{[^;]*\\b(${owned_symbols})\\b[^;]*\\};" \
    crates || true
})"

if [[ -n "$core_facade_routes" ]]; then
  printf 'production consumers route owned syntax/parser/allocator APIs through vize_atelier_core:\n%s\n' \
    "$core_facade_routes" >&2
  printf 'import Relief, Armature, or Carton directly; Atelier Core owns only shared transform/emission helpers\n' >&2
  exit 1
fi
