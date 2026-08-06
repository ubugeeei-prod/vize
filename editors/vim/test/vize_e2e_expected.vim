" Complete real-server responses for vize_e2e_spec.vim.
"
" The fixture and positions match the VS Code and Neovim host scenarios, but
" this file keeps Vim's native Dictionary/List representation so assert_equal
" detects added, removed, or moved response fields.

let g:vize_e2e_expected = {
      \ 'authored_source': [
      \   '<script setup lang="ts">',
      \   'import Child from "./Child.vue";',
      \   '',
      \   'const total = "3";',
      \   '</script>',
      \   '',
      \   '<template>',
      \   '<Child  :count="total" />',
      \   '</template>',
      \ ],
      \ 'completion': [
      \   {
      \     'detail': ' (const)',
      \     'documentation': {
      \       'kind': 'markdown',
      \       'value': "**Const**\n\nConstant binding (function, class, or literal).",
      \     },
      \     'kind': 21,
      \     'label': 'Child',
      \     'labelDetails': {'detail': ' (const)'},
      \     'sortText': '0Child',
      \   },
      \   {
      \     'detail': ' (literal)',
      \     'documentation': {
      \       'kind': 'markdown',
      \       'value': "**Literal**\n\nLiteral constant value.",
      \     },
      \     'kind': 21,
      \     'label': 'total',
      \     'labelDetails': {'detail': ' (literal)'},
      \     'sortText': '0total',
      \   },
      \ ],
      \ 'diagnostics': [
      \   {
      \     'code': 'vue/no-multi-spaces',
      \     'codeDescription': {'href': 'https://eslint.vuejs.org/rules/no-multi-spaces.html'},
      \     'message': 'Multiple consecutive spaces',
      \     'range': {
      \       'end': {'character': 8, 'line': 7},
      \       'start': {'character': 6, 'line': 7},
      \     },
      \     'severity': 2,
      \     'source': 'vize/lint',
      \   },
      \   {
      \     'code': 2322,
      \     'message': "Type 'string' is not assignable to type 'number'.",
      \     'range': {
      \       'end': {'character': 14, 'line': 7},
      \       'start': {'character': 9, 'line': 7},
      \     },
      \     'severity': 1,
      \     'source': 'vize/types',
      \   },
      \ ],
      \ 'formatting': [
      \   {
      \     'newText': "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\n\nconst total = \"3\";\n</script>\n\n<template>\n  <Child :count=\"total\" />\n</template>\n",
      \     'range': {
      \       'end': {'character': 0, 'line': 9},
      \       'start': {'character': 0, 'line': 0},
      \     },
      \   },
      \ ],
      \ 'hover': {
      \   'contents': {
      \     'kind': 'markdown',
      \     'value': "```typescript\nconst total: \"3\"\n```",
      \   },
      \   'range': {
      \     'end': {'character': 11, 'line': 3},
      \     'start': {'character': 6, 'line': 3},
      \   },
      \ },
      \ 'semantic_tokens': {'data': [7, 8, 6, 9, 0, 0, 8, 5, 8, 0]},
      \ }

function! VizeE2EExpectedCodeActions(uri) abort
  return [
        \ {
        \   'edit': {'changes': {a:uri: [
        \     {
        \       'newText': ' ',
        \       'range': {
        \         'end': {'character': 8, 'line': 7},
        \         'start': {'character': 6, 'line': 7},
        \       },
        \     },
        \   ]}},
        \   'isPreferred': v:true,
        \   'kind': 'quickfix',
        \   'title': 'Fix: Replace multiple spaces with single space',
        \ },
        \ {
        \   'edit': {'changes': {a:uri: [
        \     {
        \       'newText': "<!-- @vize:forget vue/no-multi-spaces -->\n",
        \       'range': {
        \         'end': {'character': 0, 'line': 7},
        \         'start': {'character': 0, 'line': 7},
        \       },
        \     },
        \   ]}},
        \   'isPreferred': v:false,
        \   'kind': 'quickfix',
        \   'title': 'Suppress with @vize:forget (vue/no-multi-spaces)',
        \ },
        \ ]
endfunction

function! VizeE2EExpectedRename(uri) abort
  return {'changes': {a:uri: [
        \ {
        \   'newText': 'quantity',
        \   'range': {
        \     'end': {'character': 11, 'line': 3},
        \     'start': {'character': 6, 'line': 3},
        \   },
        \ },
        \ {
        \   'newText': 'quantity',
        \   'range': {
        \     'end': {'character': 21, 'line': 7},
        \     'start': {'character': 16, 'line': 7},
        \   },
        \ },
        \ ]}}
endfunction
