local M = {}

local profiles = {
  lint = {
    lint = true,
  },
  off = {},
  recommended = {
    editor = true,
    ecosystem = true,
    lint = true,
    typecheck = true,
  },
}

local default_config = {
  autostart = true,
  cmd = { "vize", "lsp" },
  filetypes = { "vue", "art-vue" },
  init_options = profiles.lint,
  root_markers = { "vize.config.pkl", "vize.config.json", "package.json", ".git" },
  settings = {},
}

local function copy(value)
  return vim.deepcopy(value)
end

local function assert_list(name, value)
  assert(type(value) == "table", "vize." .. name .. " must be a list")
  assert(#value > 0, "vize." .. name .. " must not be empty")
  for index, item in ipairs(value) do
    assert(type(item) == "string", "vize." .. name .. "[" .. index .. "] must be a string")
    assert(item ~= "", "vize." .. name .. "[" .. index .. "] must not be empty")
  end
  return copy(value)
end

function M.profile(name)
  local profile = profiles[name]
  assert(profile ~= nil, "unknown vize profile: " .. tostring(name))
  return copy(profile)
end

function M.default_config()
  return copy(default_config)
end

function M.normalize(opts)
  opts = opts or {}
  assert(type(opts) == "table", "vize.setup options must be a table")

  local config = M.default_config()
  if opts.profile ~= nil then
    config.init_options = M.profile(opts.profile)
  end

  if opts.cmd ~= nil then
    config.cmd = assert_list("cmd", opts.cmd)
  end

  if opts.filetypes ~= nil then
    config.filetypes = assert_list("filetypes", opts.filetypes)
  end

  if opts.root_markers ~= nil then
    config.root_markers = assert_list("root_markers", opts.root_markers)
  end

  if opts.init_options ~= nil then
    assert(type(opts.init_options) == "table", "vize.init_options must be a table")
    config.init_options = copy(opts.init_options)
  end

  if opts.settings ~= nil then
    assert(type(opts.settings) == "table", "vize.settings must be a table")
    config.settings = copy(opts.settings)
  end

  if opts.autostart ~= nil then
    assert(type(opts.autostart) == "boolean", "vize.autostart must be a boolean")
    config.autostart = opts.autostart
  end

  return config
end

return M
