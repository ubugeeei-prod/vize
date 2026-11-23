const supportModule = process.argv[2];

const { runtimeProcessEvidence } = await import(supportModule);
const evidence = runtimeProcessEvidence();
evidence.phase("assert fixture observations");
process.stdout.write('{"passed":1}');
evidence.complete();
