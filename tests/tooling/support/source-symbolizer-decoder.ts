import { createInterface } from "node:readline";

const mode = process.argv[2];
for await (const address of createInterface({ input: process.stdin })) {
  if (address === "0xdeadbeefbadc0ffe") {
    process.stdout.write(address + "\ntombstone\n/unrelated.rs:12\ninline\n");
    if (mode !== "partial") process.stdout.write("/other.rs:99\n");
  } else {
    process.stdout.write(
      (mode === "wrong-address" ? "0x999" : address) + "\noriginal_function\n/source/雪.rs:12\n",
    );
  }
  if (mode === "early-exit") break;
}
