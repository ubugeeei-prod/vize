const runtime = process.env.VIZE_BUN_STUB_RUNTIME === "1" ? "bun" : "node";
module.exports.compileSfc = () => ({
  code: process.env.VIZE_BUN_STUB_DIFFER === "1" ? runtime : "identical",
  errors: [],
  warnings: [],
});
