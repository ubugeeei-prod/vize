const assert=require('node:assert/strict');
const rows=require('node:fs').readFileSync(process.env.VIZE_TOOLCHAIN_HISTORY,'utf8').split('\0');
assert.equal(rows.length,6);
assert.equal(rows[4], require(process.env.VIZE_STYLED_SESSION_OUTPUT)[0].source);
assert.equal(rows[5], require(process.env.VIZE_SCOPED_SESSION_OUTPUT)[0].source);
