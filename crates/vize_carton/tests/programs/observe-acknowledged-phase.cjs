const frame = Buffer.from(process.argv[2]);
process.stderr.write("e".repeat(100000));
process.stdout.write(`B${frame.length}\n`);
process.stdout.write(frame);
process.stdin.once("data", (ack) => {
  if (ack.length !== 1 || ack[0] !== 6) process.exit(2);
  process.stdout.end("R\n" + "x".repeat(100000));
  process.stdin.destroy();
});
