// Test-only stand-in for a CSS preprocessor: `$name: value;` variables and
// nested rules with `&`, flattened the way Sass and Less flatten them.
"use strict";

function flatten(source, parent) {
  const declarations = [];
  const nested = [];
  let text = "";
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (char === "{") {
      let depth = 1;
      let end = index + 1;
      while (end < source.length && depth > 0) {
        if (source[end] === "{") depth += 1;
        if (source[end] === "}") depth -= 1;
        end += 1;
      }
      const selector = text.trim();
      const full = !parent
        ? selector
        : selector.includes("&")
          ? selector.replaceAll("&", parent)
          : `${parent} ${selector}`;
      nested.push(...flatten(source.slice(index + 1, end - 1), full));
      text = "";
      index = end;
      continue;
    }
    if (char === ";") {
      if (text.trim()) declarations.push(`${text.trim()};`);
      text = "";
    } else {
      text += char;
    }
    index += 1;
  }
  const own = parent && declarations.length > 0 ? [`${parent} { ${declarations.join(" ")} }`] : [];
  return [...own, ...nested];
}

function compile(source) {
  const variables = new Map();
  const body = source.replace(/\$([\w-]+)\s*:\s*([^;]+);/g, (_, name, value) => {
    variables.set(name, value.trim());
    return "";
  });
  const substituted = body.replace(/\$([\w-]+)/g, (match, name) => variables.get(name) ?? match);
  return flatten(substituted, "").join("\n");
}

module.exports = { compile };
