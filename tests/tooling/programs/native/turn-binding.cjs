let begins = 0;
let closes = 0;
class TypeResolutionBatch {
  close() {
    closes += 1;
    return true;
  }
}
module.exports = {
  TypeResolutionBatch,
  beginTypeResolutionBatch() {
    begins += 1;
    return new TypeResolutionBatch();
  },
  compileSfc(source) {
    return source;
  },
  counts() {
    return { begins, closes };
  },
};
