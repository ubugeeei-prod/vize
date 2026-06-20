local plugin_root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h")
vim.opt.runtimepath:prepend(plugin_root)

local config = require("vize.config")

local function assert_eq(actual, expected, label)
  assert(vim.deep_equal(actual, expected), label .. "\nactual: " .. vim.inspect(actual))
end

local defaults = config.normalize()
assert_eq(defaults.cmd, { "vize", "lsp" }, "default cmd")
assert_eq(defaults.filetypes, { "vue", "art-vue" }, "default filetypes")
assert_eq(defaults.init_options, { lint = true }, "default lint profile")
assert(defaults.root_markers[1] == "vize.config.pkl", "prefers vize config root marker")

local recommended = config.normalize({ profile = "recommended" })
assert_eq(recommended.init_options, {
  editor = true,
  ecosystem = true,
  lint = true,
  typecheck = true,
}, "recommended profile")

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

local ok, err = pcall(config.normalize, { cmd = {} })
assert(not ok and err:match("cmd"), "rejects empty cmd")

local setup_config = require("vize").setup({ autostart = false, profile = "off" })
assert_eq(setup_config.init_options, {}, "setup returns normalized config")
assert_eq(vim.lsp.config.vize.cmd, { "vize", "lsp" }, "registered LSP cmd")
assert_eq(vim.lsp.config.vize.filetypes, { "vue", "art-vue" }, "registered LSP filetypes")

vim.cmd("runtime! ftdetect/vize.lua")
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".vue"))
assert(vim.bo.filetype == "vue", "detects .vue")
vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.tempname() .. ".art.vue"))
assert(vim.bo.filetype == "art-vue", "detects .art.vue")
