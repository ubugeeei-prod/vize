-- TS-45 Neovim driver: plays the scenario through Neovim's own LSP client,
-- configured exactly as a user configures vize.nvim (`require("vize").setup`
-- with `vim.lsp.config`/`vim.lsp.enable`). Parameters come from Neovim's own
-- builders (`make_position_params`, `vim.lsp.buf.format`, `vim.lsp.buf.rename`)
-- and edits go through Neovim's own buffer sync. This file only paces the
-- scenario; the verdict is `conformance.ts` on the tap transcript.
local scenario = vim.json.decode(io.open(os.getenv("VIZE_TS45_SCENARIO")):read("*a"))
local workspace = os.getenv("VIZE_TS45_WORKSPACE")
local document = workspace .. "/" .. scenario.document
local timeout = 180000

local function step(id)
  for _, candidate in ipairs(scenario.steps) do
    if candidate.id == id then
      return candidate
    end
  end
  error("unknown scenario step " .. id)
end

-- Pacing only: count publishes for the scenario buffer.
local publishes = {}
local default_publish = vim.lsp.handlers["textDocument/publishDiagnostics"]
vim.lsp.handlers["textDocument/publishDiagnostics"] = function(err, result, ctx, config)
  if result ~= nil and vim.uri_to_fname(result.uri) == vim.uv.fs_realpath(document) then
    table.insert(publishes, { count = #result.diagnostics, at = vim.uv.now() })
  end
  return default_publish(err, result, ctx, config)
end

local function settle(mark, count)
  local ok = vim.wait(timeout, function()
    local last = publishes[#publishes]
    return #publishes > mark and last.count == count and vim.uv.now() - last.at >= 1000
  end, 100)
  assert(ok, "diagnostics did not settle at " .. count)
end

local function client()
  local clients = vim.lsp.get_clients({ name = "vize", bufnr = 0 })
  assert(#clients == 1, "expected one vize client, found " .. #clients)
  return clients[1]
end

local function request(method, params)
  local response, err = client():request_sync(method, params, timeout, 0)
  assert(err == nil and response ~= nil and response.err == nil, method .. " failed: " .. vim.inspect(err or response))
  return response.result
end

local function cursor(position)
  vim.api.nvim_win_set_cursor(0, { position.line + 1, position.character })
  return vim.lsp.util.make_position_params(0, client().offset_encoding)
end

local function text()
  return table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), "\n") .. "\n"
end

local function run()
  local config = require("vize.config")
  local init_options = config.profile("recommended")
  init_options.formatting = true -- the user opt-in every client makes for this scenario
  require("vize").setup({ init_options = init_options })

  vim.cmd("filetype on")
  vim.cmd.edit(vim.fn.fnameescape(document))
  vim.bo.shiftwidth = 2
  vim.bo.expandtab = true
  assert(vim.wait(timeout, function()
    return #vim.lsp.get_clients({ name = "vize", bufnr = 0 }) == 1 and client().initialized
  end, 50), "vize did not attach")
  settle(0, #step("diagnostics-open").expect)

  request("textDocument/hover", cursor(step("hover").match.position))
  local completion = cursor(step("completion").match.position)
  completion.context = { triggerKind = 1 }
  request("textDocument/completion", completion)
  request("textDocument/definition", cursor(step("definition").match.position))

  -- What `vim.lsp.buf.code_action()` sends for a one-character selection,
  -- then what it does with the chosen action.
  local range = step("code-action").match.range
  local params = vim.lsp.util.make_given_range_params(
    { range.start.line + 1, range.start.character },
    { range["end"].line + 1, range["end"].character - 1 },
    0,
    client().offset_encoding
  )
  params.context = {
    diagnostics = vim.lsp.diagnostic.from(vim.diagnostic.get(0, { lnum = range.start.line })),
    triggerKind = 1,
  }
  local chosen
  for _, action in ipairs(request("textDocument/codeAction", params)) do
    if action.title == step("apply-quick-fix").action.title then
      chosen = action
    end
  end
  assert(chosen ~= nil, "the quick fix was not offered")
  local mark = #publishes
  vim.lsp.util.apply_workspace_edit(chosen.edit, client().offset_encoding)
  settle(mark, #step("diagnostics-quick-fix").expect)

  vim.lsp.buf.format({ async = false, bufnr = 0, timeout_ms = timeout })
  assert(text() == step("apply-formatting").expect, "formatting was not applied")

  local edit = step("edit").action
  mark = #publishes
  vim.api.nvim_buf_set_text(
    0,
    edit.replace.start.line,
    edit.replace.start.character,
    edit.replace["end"].line,
    edit.replace["end"].character,
    { edit.text }
  )
  settle(mark, #step("diagnostics-edit").expect)

  local rename = step("rename").match
  cursor(rename.position)
  vim.lsp.buf.rename(rename.newName, { bufnr = 0 })
  assert(vim.wait(timeout, function()
    return text() == step("apply-rename").expect
  end, 50), "rename was not applied")
  -- Let the rename's didChange flush before the client stops.
  vim.wait(500)

  -- `vim.lsp.stop_client` (what `:lsp stop` runs): shutdown, then exit once
  -- the server answered. A bare `:qall!` would not wait for the answer
  -- (`exit_timeout` defaults to false), so the handshake would go unobserved.
  local id = client().id
  client():stop()
  -- `is_stopped()` turns true as soon as stopping begins; the client is only
  -- gone once the server process exited.
  assert(vim.wait(30000, function()
    return vim.lsp.get_client_by_id(id) == nil
  end, 50), "vize did not exit after shutdown")
end

local ok, err = xpcall(run, debug.traceback)
if not ok then
  io.stderr:write("[ts-45] neovim driver: " .. tostring(err) .. "\n")
  vim.cmd("cquit 1")
end
vim.cmd("qall!")
