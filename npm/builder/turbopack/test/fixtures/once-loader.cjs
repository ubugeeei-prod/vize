module.exports = function onceLoader(source, map) {
  const marker = "vize-consumer-preprocessor-already-ran";
  if (source.includes(marker)) {
    this.callback(new Error("The same stylesheet entered its preprocessor twice."));
    return;
  }
  this.callback(null, `${source}\n/* ${marker} */\n`, map);
};
