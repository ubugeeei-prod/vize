(comment) @comment

((tag_name) @tag
  (#match? @tag "^[a-z]"))

((tag_name) @tag @tag.component.type.constructor
  (#match? @tag "^[A-Z]"))

(attribute) @attribute

(directive_attribute) @property

(quoted_attribute_value) @string

(interpolation) @punctuation.special

(raw_text) @embedded

(directive_name) @keyword.directive

(directive_argument) @constant

(start_tag) @tag

(end_tag) @tag

(self_closing_tag) @tag

"=" @operator

[
  "<"
  ">"
  "</"
  "/>"
] @punctuation.bracket
