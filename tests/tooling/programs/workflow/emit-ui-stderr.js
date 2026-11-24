const [head, tail] = process.argv.slice(2).map((part) => Buffer.from(JSON.parse(part)));
process.stdout.write("[]");
process.stderr.write(head);
if (tail.length) setTimeout(() => process.stderr.write(tail), 5);
setTimeout(() => {}, 40);
