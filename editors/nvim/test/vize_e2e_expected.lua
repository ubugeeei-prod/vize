-- Expected real-server responses for `vize_e2e_spec.lua` (#3457).
--
-- Every table here is a COMPLETE response captured from a raw LSP probe of the
-- real `vize lsp` binary against the `real-vue` fixture's `src/Scenario.vue`.
-- They are compared with `vim.deep_equal`, so a capability that starts
-- answering with more, fewer, or differently anchored results fails the
-- scenario instead of quietly losing coverage.

local authored_source = table.concat({
  '<script setup lang="ts">',
  'import Child from "./Child.vue";',
  "",
  'const total = "3";',
  "</script>",
  "",
  "<template>",
  '<Child  :count="total" />',
  "</template>",
  "",
}, "\n")

-- Each derived source is one authored edit away from the previous stage. A
-- silent zero-replacement `gsub` would leave the expectation equal to the
-- previous stage and fail later under a misleading label, so every step
-- asserts it actually matched the fixture text.
local function replace_once(source, pattern, replacement)
  local result, count = source:gsub(pattern, replacement, 1)
  assert(count == 1, "expected fixture text not found: " .. pattern)
  return result
end

local quick_fixed_source = replace_once(authored_source, "<Child  :count", "<Child :count")
local formatted_source =
  replace_once(quick_fixed_source, '<Child :count="total" />', '  <Child :count="total" />')
local renamed_source = replace_once(formatted_source, "const total =", "const quantity =")
renamed_source = replace_once(renamed_source, ':count="total"', ':count="quantity"')

local ref_surface_source = table.concat({
  '<script setup lang="ts">',
  'import { computed, ref, useTemplateRef } from "vue";',
  "",
  "const count = ref(1);",
  "const doubled = computed(() => count.value * 2);",
  'const button = useTemplateRef<HTMLButtonElement>("button");',
  "</script>",
  "",
  "<template>",
  '  <button ref="button">{{ count }} {{ doubled }} {{ button }}</button>',
  "</template>",
  "",
}, "\n")

local component_contract_child_source = table.concat({
  '<script setup lang="ts">',
  "defineProps<{ label: string; count?: number }>()",
  "defineEmits<{ save: [value: string] }>()",
  "defineSlots<{ default(props: { value: string }): unknown }>()",
  "defineModel<boolean>()",
  "</script>",
  "",
  '<template><slot value="ready" /></template>',
  "",
}, "\n")

local component_contract_host_source = table.concat({
  '<script setup lang="ts">',
  "import ContractChild from './ContractChild.vue'",
  "",
  "ContractChild",
  "</script>",
  "",
  "<template>",
  '  <ContractChild label="ready" />',
  "</template>",
  "",
}, "\n")

local component_contract_hover_value = [[```typescript
const ContractChild: VueComponent
{
  props: { label: string; count?: number };
  emits: { save: [value: string] };
  slots: { default(props: { value: string }): unknown };
  model: "modelValue": boolean;
}
```

Vue component: ContractChild.vue]]

return {
  authored_source = authored_source,
  component_contract_child_source = component_contract_child_source,
  component_contract_host_source = component_contract_host_source,
  quick_fixed_source = quick_fixed_source,
  formatted_source = formatted_source,
  renamed_source = renamed_source,
  ref_surface_source = ref_surface_source,

  -- The authored `<Child  :count="total" />` carries two independent authored
  -- bugs on one line: two spaces after the tag name (a fixable lint warning)
  -- and a string bound to a `number` prop (the type bug the #3224 scorecard
  -- asks for). Both anchor on the authored span, never on virtual TS.
  diagnostics = {
    {
      code = "vue/no-multi-spaces",
      codeDescription = { href = "https://eslint.vuejs.org/rules/no-multi-spaces.html" },
      message = "Multiple consecutive spaces",
      range = {
        ["end"] = { character = 8, line = 7 },
        start = { character = 6, line = 7 },
      },
      severity = 2,
      source = "vize/lint",
    },
    {
      code = 2322,
      message = "Type 'string' is not assignable to type 'number'.",
      range = {
        ["end"] = { character = 14, line = 7 },
        start = { character = 9, line = 7 },
      },
      severity = 1,
      source = "vize/types",
    },
  },

  completion_position = { character = 16, line = 7 },
  completion_include = { "Child", "total" },
  completion_exclude = { "count", "v-if" },

  hover_position = { character = 8, line = 3 },
  hover = {
    contents = {
      kind = "markdown",
      value = '```typescript\nconst total: "3"\n```',
    },
    range = {
      ["end"] = { character = 11, line = 3 },
      start = { character = 6, line = 3 },
    },
  },

  ref_surface_hovers = {
    script_count = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst count: Ref<number, number>\n```",
      },
      range = {
        ["end"] = { character = 11, line = 3 },
        start = { character = 6, line = 3 },
      },
    },
    script_doubled = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst doubled: ComputedRef<number>\n```",
      },
      range = {
        ["end"] = { character = 13, line = 4 },
        start = { character = 6, line = 4 },
      },
    },
    script_button = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst button: Readonly<ShallowRef<HTMLButtonElement | null, HTMLButtonElement | null>>\n```",
      },
      range = {
        ["end"] = { character = 12, line = 5 },
        start = { character = 6, line = 5 },
      },
    },
    template_count = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst count: number\n```",
      },
      range = {
        ["end"] = { character = 31, line = 9 },
        start = { character = 26, line = 9 },
      },
    },
    template_doubled = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst doubled: number\n```",
      },
      range = {
        ["end"] = { character = 45, line = 9 },
        start = { character = 38, line = 9 },
      },
    },
    template_button = {
      contents = {
        kind = "markdown",
        value = "```typescript\nconst button: HTMLButtonElement | null\n```",
      },
      range = {
        ["end"] = { character = 58, line = 9 },
        start = { character = 52, line = 9 },
      },
    },
  },

  component_contract_hovers = {
    import_binding = {
      contents = {
        kind = "markdown",
        value = component_contract_hover_value,
      },
      range = {
        ["end"] = { character = 20, line = 1 },
        start = { character = 7, line = 1 },
      },
    },
    script_usage = {
      contents = {
        kind = "markdown",
        value = component_contract_hover_value,
      },
      range = {
        ["end"] = { character = 13, line = 3 },
        start = { character = 0, line = 3 },
      },
    },
  },

  quick_fix_range = {
    ["end"] = { character = 8, line = 7 },
    start = { character = 6, line = 7 },
  },

  code_actions = function(uri)
    return {
      {
        edit = {
          changes = {
            [uri] = {
              {
                newText = " ",
                range = {
                  ["end"] = { character = 8, line = 7 },
                  start = { character = 6, line = 7 },
                },
              },
            },
          },
        },
        isPreferred = true,
        kind = "quickfix",
        title = "Fix: Replace multiple spaces with single space",
      },
      {
        edit = {
          changes = {
            [uri] = {
              {
                newText = "<!-- @vize:forget vue/no-multi-spaces -->\n",
                range = {
                  ["end"] = { character = 0, line = 7 },
                  start = { character = 0, line = 7 },
                },
              },
            },
          },
        },
        isPreferred = false,
        kind = "quickfix",
        title = "Suppress with @vize:forget (vue/no-multi-spaces)",
      },
    }
  end,

  -- The SFC formatter answers with one whole-document replacement.
  formatting_edits = {
    {
      newText = formatted_source,
      range = {
        ["end"] = { character = 0, line = 9 },
        start = { character = 0, line = 0 },
      },
    },
  },

  -- `{deltaLine, deltaStart, length, tokenType, tokenModifiers} * 2` against
  -- the server legend: `:count` is a `property` (type 9) and `total` a
  -- `variable` (type 8), both on the formatted template line.
  semantic_tokens = { data = { 7, 9, 6, 9, 0, 0, 8, 5, 8, 0 } },

  rename_new_name = "quantity",
  rename_position = { character = 8, line = 3 },
  rename_edit = function(uri)
    return {
      changes = {
        [uri] = {
          {
            newText = "quantity",
            range = {
              ["end"] = { character = 11, line = 3 },
              start = { character = 6, line = 3 },
            },
          },
          {
            newText = "quantity",
            range = {
              ["end"] = { character = 22, line = 7 },
              start = { character = 17, line = 7 },
            },
          },
        },
      },
    }
  end,
}
