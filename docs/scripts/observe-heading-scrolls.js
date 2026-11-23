(() => {
  window.headingScrolls = [];
  const original = Reflect.get(Element.prototype, "scrollIntoView");
  Element.prototype.scrollIntoView = function (...args) {
    window.headingScrolls.push(this.id);
    return Reflect.apply(original, this, args);
  };
})();
