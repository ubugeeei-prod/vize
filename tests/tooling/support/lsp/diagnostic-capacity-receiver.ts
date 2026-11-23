import { createServer } from "node:net";

createServer((socket) => {
  socket.pause();
  process.stdout.write("accepted\n");
}).listen(process.argv[2], () => process.stdout.write("ready\n"));
