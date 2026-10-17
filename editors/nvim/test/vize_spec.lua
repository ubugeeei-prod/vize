local plugin_root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h")
vim.opt.runtimepath:prepend(plugin_root)

if vim.env.VIZE_TEST_ART_VUE_NATIVE_PARSER == "1" then
  -- Reuse a bundled parser binary to exercise Neovim's real loaded-parser path
  -- under the otherwise unavailable Art Vue language name.
  local parser_path = vim.api.nvim_get_runtime_file("parser/vim.*", false)[1]
  assert(parser_path, "finds Neovim's bundled Vim parser")
  local loaded, err = vim.treesitter.language.add("art-vue", {
    path = parser_path,
    symbol_name = "vim",
  })
  assert(loaded, err)
  vim.cmd("runtime! ftdetect/vize.lua")
  assert(
    vim.treesitter.language.get_lang("art-vue") == "art-vue",
    "preserves a loaded parser under the Art Vue language name"
  )
  io.stdout:write("VIZE_HEADLESS_NATIVE_PARSER_PASSED\n")
  return
end

-- A required assertion must fail the same headless Lua carrier used by CI.
-- Ex-command `+luafile ... +qa` logs Lua errors but can still exit successfully.
local control_path = vim.fn.tempname() .. ".lua"
local control_args = { vim.v.progpath, "--headless", "-u", "NONE", "--noplugin", "-l", control_path }
vim.fn.writefile({ 'io.stdout:write("VIZE_HEADLESS_CONTROL_PASSED\\n")' }, control_path)
local success_control = vim.system(control_args, { text = true }):wait(10000)
vim.fn.writefile({ 'assert(false, "VIZE_HEADLESS_CONTROL_FAILED")' }, control_path)
local failure_control = vim.system(control_args, { text = true }):wait(10000)
vim.fn.delete(control_path)
assert(
  success_control.code == 0 and success_control.stdout:find("VIZE_HEADLESS_CONTROL_PASSED", 1, true),
  "headless Lua success executes and exits zero"
)
assert(
  failure_control.code == 1 and failure_control.stderr:find("VIZE_HEADLESS_CONTROL_FAILED", 1, true),
  "headless Lua assertion executes and exits one"
)

local config = require("vize.config")

local function assert_eq(actual, expected, label)
  assert(vim.deep_equal(actual, expected), label .. "\nactual: " .. vim.inspect(actual))
end

local function assert_fails(label, fn, pattern)
  local ok, err = pcall(fn)
  assert(not ok and err:match(pattern), label .. "\nerror: " .. tostring(err))
end

local defaults = config.normalize()
assert_eq(defaults.cmd, { "vize", "lsp" }, "default cmd")
assert_eq(defaults.filetypes, { "vue", "art-vue" }, "default filetypes")
assert_eq(defaults.init_options, {}, "default defers feature settings to the server")
assert(vim.json.encode(defaults.init_options) == "{}", "empty defaults serialize as an LSP object")
assert_eq(defaults.root_markers, {
  "vite.config.ts",
  "vite.config.js",
  "vite.config.mts",
  "vite.config.mjs",
  "vite.config.cts",
  "vite.config.cjs",
  "tsconfig.json",
  "package.json",
  ".git",
}, "project root markers")

local recommended = config.normalize({ profile = "recommended" })
assert_eq(recommended.init_options, {
  editor = true,
  ecosystem = true,
  lint = true,
  typecheck = true,
}, "recommended profile")

local off_profile = {
  enabled = false,
  nativeLinkedEditing = false,
  diagnostics = false,
  lint = false,
  typecheck = false,
  editor = false,
  ecosystem = false,
  optionsApi = false,
  legacyVue2 = false,
  completion = false,
  signatureHelp = false,
  hover = false,
  definition = false,
  references = false,
  documentSymbols = false,
  workspaceSymbols = false,
  codeActions = false,
  rename = false,
  formatting = false,
  codeLens = false,
  semanticTokens = false,
  documentLinks = false,
  foldingRanges = false,
  inlayHints = false,
  fileRename = false,
  autoInsert = false,
  corsa = false,
  tsgo = false,
  crossFile = false,
}
local lint_profile_expected = vim.deepcopy(off_profile)
lint_profile_expected.enabled = true
lint_profile_expected.lint = true
assert_eq(config.profile("lint"), lint_profile_expected, "complete lint-only profile")
assert_eq(config.profile("off"), off_profile, "complete off profile")

local lint_profile = config.profile("lint")
lint_profile.lint = false
assert_eq(config.profile("lint"), lint_profile_expected, "profile returns copy")

local project = {}
for key, _ in pairs(off_profile) do
  project[key] = true
end
for _, name in ipairs({ "off", "lint" }) do
  local wire = vim.json.decode(vim.json.encode(config.profile(name)))
  local expected = name == "off" and off_profile or lint_profile_expected
  assert_eq(vim.tbl_extend("force", project, wire), expected, "explicit profile beats all-on project")
end
assert_eq(vim.tbl_extend("force", project, defaults.init_options), project, "default inherits project")
local partial = config.normalize({ init_options = { hover = false } }).init_options
local partial_expected = vim.deepcopy(project)
partial_expected.hover = false
assert_eq(vim.tbl_extend("force", project, partial), partial_expected, "partial override keeps other features")

local default_copy = config.default_config()
default_copy.cmd[1] = "changed"
assert_eq(config.default_config().cmd, { "vize", "lsp" }, "default config returns copy")

local custom = config.normalize({
  autostart = false,
  cmd = { "/tmp/vize", "lsp", "--debug" },
  filetypes = { "vue" },
  init_options = { hover = true, references = true },
  root_markers = { "deno.json", ".git" },
  settings = { vize = { trace = "messages" } },
})

assert_eq(custom.cmd, { "/tmp/vize", "lsp", "--debug" }, "custom cmd")
assert_eq(custom.filetypes, { "vue" }, "custom filetypes")
assert_eq(custom.init_options, { hover = true, references = true }, "custom init options")
assert_eq(custom.root_markers, { "deno.json", ".git" }, "custom root markers")
assert(custom.autostart == false, "custom autostart")

local init_options_override_profile = config.normalize({
  profile = "lint",
  init_options = { hover = true, references = true },
})
assert_eq(
  init_options_override_profile.init_options,
  { hover = true, references = true },
  "explicit init_options override profile"
)

local mutable_opts = { init_options = { hover = true } }
local mutable_config = config.normalize(mutable_opts)
mutable_opts.init_options.hover = false
assert_eq(mutable_config.init_options, { hover = true }, "normalization copies init options")

local mutable_nested_opts = { init_options = { nested = { hover = true } } }
local mutable_nested_config = config.normalize(mutable_nested_opts)
mutable_nested_opts.init_options.nested.hover = false
assert_eq(
  mutable_nested_config.init_options,
  { nested = { hover = true } },
  "normalization copies nested init options"
)

local mutable_settings_opts = { settings = { vize = { trace = "messages" } } }
local mutable_settings_config = config.normalize(mutable_settings_opts)
mutable_settings_opts.settings.vize.trace = "off"
assert_eq(
  mutable_settings_config.settings,
  { vize = { trace = "messages" } },
  "normalization copies nested settings"
)

local mutable_cmd_opts = { cmd = { "vize", "lsp" } }
local mutable_cmd_config = config.normalize(mutable_cmd_opts)
mutable_cmd_opts.cmd[1] = "changed"
assert_eq(mutable_cmd_config.cmd, { "vize", "lsp" }, "normalization copies cmd")

local mutable_filetypes_opts = { filetypes = { "vue", "art-vue" } }
local mutable_filetypes_config = config.normalize(mutable_filetypes_opts)
mutable_filetypes_opts.filetypes[1] = "changed"
assert_eq(mutable_filetypes_config.filetypes, { "vue", "art-vue" }, "normalization copies filetypes")

local mutable_root_opts = { root_markers = { "tsconfig.json", ".git" } }
local mutable_root_config = config.normalize(mutable_root_opts)
mutable_root_opts.root_markers[1] = "changed"
assert_eq(
  mutable_root_config.root_markers,
  { "tsconfig.json", ".git" },
  "normalization copies root markers"
)

assert_fails("rejects unknown profile", function()
  config.profile("missing")
end, "unknown vize profile")
assert_fails("rejects non-table setup options", function()
  config.normalize("invalid")
end, "options")
assert_fails("rejects empty cmd", function()
  config.normalize({ cmd = {} })
end, "cmd")
assert_fails("rejects non-list cmd", function()
  config.normalize({ cmd = "vize" })
end, "cmd")
assert_fails("rejects non-string cmd item", function()
  config.normalize({ cmd = { "vize", 42 } })
end, "cmd")
assert_fails("rejects empty filetype", function()
  config.normalize({ filetypes = { "" } })
end, "filetypes")
assert_fails("rejects empty root marker", function()
  config.normalize({ root_markers = { "" } })
end, "root_markers")
assert_fails("rejects non-table init options", function()
  config.normalize({ init_options = false })
end, "init_options")
assert_fails("rejects non-table settings", function()
  config.normalize({ settings = "trace" })
end, "settings")
assert_fails("rejects non-boolean autostart", function()
  config.normalize({ autostart = "yes" })
end, "autostart")

local setup_config = require("vize").setup({ autostart = false })
assert_eq(setup_config.init_options, {}, "setup preserves project-owned feature settings")
assert_eq(vim.lsp.config.vize.init_options, {}, "registered LSP sends no feature overrides")
assert_eq(vim.lsp.config.vize.cmd, { "vize", "lsp" }, "registered LSP cmd")
assert_eq(vim.lsp.config.vize.filetypes, { "vue", "art-vue" }, "registered LSP filetypes")

local original_enable = vim.lsp.enable
local enable_calls = {}
vim.lsp.enable = function(name)
  table.insert(enable_calls, name)
end

local autostart_config = require("vize").setup({ profile = "lint" })
assert_eq(autostart_config.init_options, lint_profile_expected, "setup returns lint profile")
assert_eq(enable_calls, { "vize" }, "setup autostarts vize")
assert_eq(vim.lsp.config.vize.init_options, lint_profile_expected, "setup registers normalized init options")

require("vize").setup({ autostart = false, profile = "off" })
assert_eq(enable_calls, { "vize" }, "autostart false does not enable vize")
vim.lsp.enable = original_enable

vim.cmd("runtime! ftdetect/vize.lua")
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".vue"))
assert(vim.bo.filetype == "vue", "detects .vue")
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".art.vue"))
assert(vim.bo.filetype == "art-vue", "detects .art.vue")
assert(vim.bo.syntax == "vue", "reuses Vue syntax for .art.vue")
assert(
  vim.treesitter.language.get_lang(vim.bo.filetype) == "vue",
  "reuses the Vue tree-sitter parser for .art.vue"
)

vim.treesitter.language.register("art-vue", "art-vue")
vim.cmd("runtime! ftdetect/vize.lua")
assert(
  vim.treesitter.language.get_lang("art-vue") == "art-vue",
  "preserves an explicit identity Art Vue parser mapping"
)

vim.treesitter.language.register("custom-art", "art-vue")
vim.cmd("runtime! ftdetect/vize.lua")
assert(
  vim.treesitter.language.get_lang("art-vue") == "custom-art",
  "preserves an explicit Art Vue tree-sitter parser mapping"
)

local syntax_root = vim.fn.tempname()
vim.fn.mkdir(syntax_root .. "/syntax", "p")
vim.fn.writefile({ 'let b:current_syntax = "art-vue"' }, syntax_root .. "/syntax/art-vue.vim")
vim.opt.runtimepath:prepend(syntax_root)
vim.cmd("syntax on")
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".art.vue"))
assert(vim.bo.syntax == "art-vue", "preserves a standard Art Vue syntax file")
assert(vim.b.current_syntax == "art-vue", "loads the standard Art Vue syntax file")
vim.opt.runtimepath:remove(syntax_root)
vim.fn.delete(syntax_root, "rf")

vim.api.nvim_create_autocmd("FileType", {
  pattern = "art-vue",
  callback = function()
    vim.bo.syntax = "custom-art"
  end,
})
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".art.vue"))
assert(vim.bo.syntax == "custom-art", "preserves an explicit Art Vue syntax override")

if vim.env.VIZE_PROFILE_PACKET then
  vim.fn.writefile({ vim.json.encode({
    defaults = defaults.init_options,
    partial = partial,
    recommended = recommended.init_options,
    lint = config.profile("lint"),
    off = config.profile("off"),
  }) }, vim.env.VIZE_PROFILE_PACKET)
end

io.stdout:write("VIZE_HEADLESS_NORMAL_PASSED\n")
