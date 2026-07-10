; Comments appear as annotations in the outline
(comment) @annotation

; ========== template =========
(template_element
  (start_tag) @name) @item

; Custom elements/components (PascalCase or hyphenated) - self-closing tags
(element
  (start_tag) @name) @item

(element
  (self_closing_tag) @name) @item

; ======= script ======
(script_element
  (start_tag) @name) @item

; ========= style =========
(style_element
  (start_tag) @name) @item
