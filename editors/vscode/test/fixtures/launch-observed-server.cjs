const { proxy, server, protocol } = JSON.parse("@@PROGRAM_SETTINGS@@");

if (server !== undefined) process.env.VIZE_TEST_OBSERVED_SERVER_PATH = server;
if (protocol !== undefined) process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH = protocol;
require(proxy);
