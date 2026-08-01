-- Headless Neovim end-to-end scenario against a real `vize lsp` process.
--
-- Covers the #3224 parity scorecard row for Neovim: type bug -> diagnostic at
-- the authored span -> quick fix -> format-on-save -> semantic tokens ->
-- rename. Every step asserts the COMPLETE server response with
-- `vim.deep_equal`; there is no substring or "contains" check anywhere here.
--
-- `tools/nvim-vize/run-real-server.mjs` prepares the workspace and launches
-- this file; run it with `vp run --workspace-root test:nvim-extension:real-server`.
--
-- The session starts through `vim.lsp.start` rather than `require("vize").setup()`
-- so the scenario runs on every Neovim that ships a Lua LSP client, not only
-- the 0.11+ releases that provide `vim.lsp.config`/`vim.lsp.enable`. The config
-- it starts from is still the plugin's own `vize.config.normalize` output, so a
-- regression in the packaged defaults still fails here.

local plugin_root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h")
vim.opt.runtimepath:prepend(plugin_root)

local expected = dofile(plugin_root .. "/test/vize_e2e_expected.lua")
local config = require("vize.config")

local server_path = os.getenv("VIZE_E2E_SERVER")
local workspace_path = os.getenv("VIZE_E2E_WORKSPACE")

local function fail(label, actual)
  error(label .. "\nactual: " .. vim.inspect(actual), 0)
end

local function assert_eq(actual, want, label)
  if not vim.deep_equal(actual, want) then
    fail(label .. "\nexpected: " .. vim.inspect(want), actual)
  end
end

local function request(bufnr, method, params)
  local responses, request_error = vim.lsp.buf_request_sync(bufnr, method, params, 120000)
  assert(request_error == nil, method .. " failed: " .. tostring(request_error))
  assert(responses ~= nil, method .. " timed out")
  local _, response = next(responses)
  assert(response ~= nil, method .. " produced no response")
  assert(response.error == nil, method .. " returned an error: " .. vim.inspect(response.error))
  return response.result
end

local function buffer_text(bufnr)
  return table.concat(vim.api.nvim_buf_get_lines(bufnr, 0, -1, true), "\n") .. "\n"
end

local function read_file(path)
  local handle = assert(io.open(path, "r"))
  local contents = handle:read("*a")
  handle:close()
  return contents
end

local function sorted_by_start(diagnostics)
  local sorted = vim.deepcopy(diagnostics)
  table.sort(sorted, function(left, right)
    if left.range.start.line ~= right.range.start.line then
      return left.range.start.line < right.range.start.line
    end
    return left.range.start.character < right.range.start.character
  end)
  return sorted
end

--- Step 1: the real server publishes both authored bugs, on authored spans.
local function step_diagnostics(uri, published)
  local settled = vim.wait(240000, function()
    local diagnostics = published[uri]
    return diagnostics ~= nil and #diagnostics >= #expected.diagnostics
  end, 200)
  if not settled then
    fail("real server did not publish the scenario diagnostics", published)
  end

  -- `vim.tbl_keys` has no defined order, so sort before the deep compare.
  local published_uris = vim.tbl_keys(published)
  table.sort(published_uris)
  assert_eq(published_uris, { uri }, "diagnostics were published for exactly one file")
  assert_eq(sorted_by_start(published[uri]), expected.diagnostics, "published diagnostics")
end

--- Step 2: the quick fix the server offers on the lint warning's own span.
local function step_quick_fix(bufnr, uri, offset_encoding)
  local actions = request(bufnr, "textDocument/codeAction", {
    context = { diagnostics = {} },
    range = expected.quick_fix_range,
    textDocument = { uri = uri },
  })
  assert_eq(actions, expected.code_actions(uri), "code actions on the lint warning span")

  vim.lsp.util.apply_workspace_edit(actions[1].edit, offset_encoding)
  assert_eq(buffer_text(bufnr), expected.quick_fixed_source, "buffer after applying the quick fix")
end

--- Step 3: format-on-save, wired the way a Neovim user wires it.
local function step_format_on_save(bufnr, uri, scenario_path)
  vim.api.nvim_create_autocmd("BufWritePre", {
    buffer = bufnr,
    callback = function()
      vim.lsp.buf.format({ async = false, bufnr = bufnr, timeout_ms = 120000 })
    end,
  })

  local edits = request(bufnr, "textDocument/formatting", {
    options = { insertSpaces = true, tabSize = 2 },
    textDocument = { uri = uri },
  })
  assert_eq(edits, expected.formatting_edits, "formatting edits")
  assert_eq(buffer_text(bufnr), expected.quick_fixed_source, "formatting must not edit on its own")

  vim.cmd("write")
  assert_eq(buffer_text(bufnr), expected.formatted_source, "buffer after format-on-save")
  assert_eq(read_file(scenario_path), expected.formatted_source, "file on disk after format-on-save")
  assert_eq(vim.bo[bufnr].modified, false, "format-on-save leaves the buffer saved")
end

--- Step 4: semantic tokens for the formatted document.
local function step_semantic_tokens(bufnr, uri)
  local tokens = request(bufnr, "textDocument/semanticTokens/full", {
    textDocument = { uri = uri },
  })
  assert_eq(tokens, expected.semantic_tokens, "semantic tokens")
end

--- Step 5: rename the script binding the template consumes.
local function step_rename(bufnr, uri, offset_encoding)
  local edit = request(bufnr, "textDocument/rename", {
    newName = expected.rename_new_name,
    position = expected.rename_position,
    textDocument = { uri = uri },
  })
  assert_eq(edit, expected.rename_edit(uri), "rename workspace edit")

  vim.lsp.util.apply_workspace_edit(edit, offset_encoding)
  assert_eq(buffer_text(bufnr), expected.renamed_source, "buffer after applying the rename")
end

local function start_client(bufnr, published)
  -- The packaged `recommended` profile leaves formatting off, matching the
  -- server default. Format-on-save is an explicit opt-in, so the scenario
  -- turns it on the same way a user would: through `init_options`.
  local init_options = config.profile("recommended")
  init_options.formatting = true

  local resolved = config.normalize({
    cmd = { server_path, "lsp" },
    init_options = init_options,
  })
  assert_eq(resolved.filetypes, { "vue", "art-vue" }, "packaged filetypes")
  assert_eq(
    resolved.root_markers,
    { "vize.config.pkl", "vize.config.json", "package.json", ".git" },
    "packaged root markers"
  )

  local client_id = vim.lsp.start({
    cmd = resolved.cmd,
    handlers = {
      ["textDocument/publishDiagnostics"] = function(_, result)
        if result ~= nil then
          published[result.uri] = result.diagnostics
        end
      end,
    },
    init_options = resolved.init_options,
    name = "vize",
    root_dir = workspace_path,
  }, { bufnr = bufnr })
  assert(client_id ~= nil, "vim.lsp.start did not return a client id")

  local ready = vim.wait(120000, function()
    local client = vim.lsp.get_client_by_id(client_id)
    return client ~= nil and client.initialized == true and vim.lsp.buf_is_attached(bufnr, client_id)
  end, 100)
  assert(ready, "the vize language server did not initialize")

  return client_id
end

local function main()
  assert(server_path ~= nil and server_path ~= "", "VIZE_E2E_SERVER must be set")
  assert(workspace_path ~= nil and workspace_path ~= "", "VIZE_E2E_WORKSPACE must be set")

  local scenario_path = workspace_path .. "/src/Scenario.vue"
  vim.cmd("runtime! ftdetect/vize.lua")
  vim.cmd("edit " .. vim.fn.fnameescape(scenario_path))

  local bufnr = vim.api.nvim_get_current_buf()
  assert_eq(vim.bo[bufnr].filetype, "vue", "ftdetect maps the fixture to the vue filetype")
  assert_eq(buffer_text(bufnr), expected.authored_source, "authored fixture source")

  local published = {}
  local client_id = start_client(bufnr, published)
  local uri = vim.uri_from_bufnr(bufnr)

  -- Apply edits with the position encoding the client actually negotiated
  -- rather than assuming UTF-16, so a future encoding change is not silently
  -- mis-applied here.
  local client = vim.lsp.get_client_by_id(client_id)
  assert(client ~= nil, "the vize client disappeared after initialization")
  local offset_encoding = client.offset_encoding
  assert(offset_encoding ~= nil, "the vize client negotiated no position encoding")

  step_diagnostics(uri, published)
  step_quick_fix(bufnr, uri, offset_encoding)
  step_format_on_save(bufnr, uri, scenario_path)
  step_semantic_tokens(bufnr, uri)
  step_rename(bufnr, uri, offset_encoding)

  vim.lsp.stop_client(client_id, true)
end

local ok, err = pcall(main)
if not ok then
  io.stderr:write("neovim real-server scenario failed:\n" .. tostring(err) .. "\n")
  vim.cmd("cquit 1")
end

io.stdout:write("neovim real-server scenario passed\n")
