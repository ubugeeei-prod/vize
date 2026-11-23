const assert=require('node:assert/strict');
const rows=require('node:fs').readFileSync(process.env.VIZE_TOOLCHAIN_HISTORY,'utf8').split('\0');
assert.equal(rows.length,5);
assert.equal(rows[4], require(process.env.VIZE_STYLED_SESSION_OUTPUT)[0].source);
