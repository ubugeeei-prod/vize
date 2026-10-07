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
      \   {'label': '$attrs', 'kind': 10, 'detail': 'Fallthrough attributes', 'sortText': '1$attrs'},
      \   {'label': '$slots', 'kind': 10, 'detail': 'Slots from parent', 'sortText': '1$slots'},
      \   {'label': '$refs', 'kind': 10, 'detail': 'Template refs', 'sortText': '1$refs'},
      \   {'label': '$el', 'kind': 10, 'detail': 'Root element', 'sortText': '1$el'},
      \   {'label': '$props', 'kind': 10, 'detail': 'Props object', 'sortText': '1$props'},
      \   {'label': '$data', 'kind': 10, 'detail': 'Component data', 'sortText': '1$data'},
      \   {'label': '$options', 'kind': 10, 'detail': 'Component options', 'sortText': '1$options'},
      \   {'label': '$parent', 'kind': 10, 'detail': 'Parent instance', 'sortText': '1$parent'},
      \   {'label': '$root', 'kind': 10, 'detail': 'Root instance', 'sortText': '1$root'},
      \   {'label': '$host', 'kind': 10, 'detail': 'Custom element host', 'sortText': '1$host'},
      \   {'label': '$emit', 'kind': 2, 'detail': 'Emit event', 'sortText': '1$emit'},
      \   {'label': '$forceUpdate', 'kind': 2, 'detail': 'Schedule a component update', 'sortText': '1$forceUpdate'},
      \   {'label': '$nextTick', 'kind': 2, 'detail': 'Wait for the next DOM update', 'sortText': '1$nextTick'},
      \   {'label': '$watch', 'kind': 2, 'detail': 'Watch a reactive source', 'sortText': '1$watch'},
      \   {'label': 'Infinity', 'kind': 21, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Infinity'},
      \   {'label': 'undefined', 'kind': 21, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1undefined'},
      \   {'label': 'NaN', 'kind': 21, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1NaN'},
      \   {'label': 'isFinite', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1isFinite'},
      \   {'label': 'isNaN', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1isNaN'},
      \   {'label': 'parseFloat', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1parseFloat'},
      \   {'label': 'parseInt', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1parseInt'},
      \   {'label': 'decodeURI', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1decodeURI'},
      \   {'label': 'decodeURIComponent', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1decodeURIComponent'},
      \   {'label': 'encodeURI', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1encodeURI'},
      \   {'label': 'encodeURIComponent', 'kind': 3, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1encodeURIComponent'},
      \   {'label': 'Number', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Number'},
      \   {'label': 'Date', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Date'},
      \   {'label': 'Array', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Array'},
      \   {'label': 'Object', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Object'},
      \   {'label': 'Boolean', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Boolean'},
      \   {'label': 'String', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1String'},
      \   {'label': 'RegExp', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1RegExp'},
      \   {'label': 'Map', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Map'},
      \   {'label': 'Set', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Set'},
      \   {'label': 'BigInt', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1BigInt'},
      \   {'label': 'Error', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Error'},
      \   {'label': 'Symbol', 'kind': 7, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Symbol'},
      \   {'label': 'Math', 'kind': 9, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Math'},
      \   {'label': 'JSON', 'kind': 9, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1JSON'},
      \   {'label': 'Intl', 'kind': 9, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1Intl'},
      \   {'label': 'console', 'kind': 9, 'detail': 'JavaScript global allowed in Vue templates', 'sortText': '1console'},
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
