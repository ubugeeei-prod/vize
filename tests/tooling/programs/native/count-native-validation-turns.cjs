const fixture = JSON.parse(process.argv[2]);

const native = require(fixture.nativeLibrary);
native.compileSfc("one");
native.compileSfc("two");
queueMicrotask(() => {
  const first = native.counts();
  native.compileSfc("three");
  queueMicrotask(() => console.log(JSON.stringify({ first, second: native.counts() })));
});
