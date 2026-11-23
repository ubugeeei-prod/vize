const s = require(process.argv[2]);
process.exit(s.selected.includes("vize_atelier_vapor") ? 0 : 1);
