globalThis.wrapperLoads = (globalThis.wrapperLoads ?? 0) + 1;
module.exports = require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
