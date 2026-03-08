const I18N_FN_MAP: Record<string, string> = {
  $t: "t: $t",
  $rt: "rt: $rt",
  $d: "d: $d",
  $n: "n: $n",
  $tm: "tm: $tm",
  $te: "te: $te",
};

const I18N_FN_RE = /(?<![.\w])\$([tdn]|rt|tm|te)\s*\(/g;
const SETUP_FN_RE = /setup\s*\(__props[\s\S]*?\)\s*\{/;
const USE_I18N_DESTRUCTURE_RE = /const\s*\{([^}]*)\}\s*=\s*useI18n\s*\(\s*\)\s*;?/;

function getLocalAlias(specifier: string): string {
  const colon = specifier.indexOf(":");
  return (colon === -1 ? specifier : specifier.slice(colon + 1)).trim();
}

function collectUsedI18nSpecifiers(code: string): string[] {
  const used = new Set<string>();

  for (const match of code.matchAll(I18N_FN_RE)) {
    const fnName = `$${match[1]}`;
    const specifier = I18N_FN_MAP[fnName];
    if (specifier) {
      used.add(specifier);
    }
  }

  return Array.from(used);
}

function collectDestructuredLocalNames(destructure: string): Set<string> {
  const locals = new Set<string>();

  for (const rawPart of destructure.split(",")) {
    const part = rawPart.trim();
    if (!part) continue;

    const withoutDefault = (part.split("=")[0] ?? part).trim();
    const aliasMatch = withoutDefault.match(/^(?:\.\.\.)?[^:]+:\s*(.+)$/);
    const localName = (aliasMatch ? aliasMatch[1] : withoutDefault).trim();

    if (localName) {
      locals.add(localName);
    }
  }

  return locals;
}

export function injectNuxtI18nHelpers(code: string): string {
  const usedSpecifiers = collectUsedI18nSpecifiers(code);
  if (usedSpecifiers.length === 0) {
    return code;
  }

  const setupMatch = code.match(SETUP_FN_RE);
  if (!setupMatch || setupMatch.index === undefined) {
    return code;
  }

  const setupBodyStart = setupMatch.index + setupMatch[0].length;
  const setupBody = code.slice(setupBodyStart);
  const existingMatch = setupBody.match(USE_I18N_DESTRUCTURE_RE);

  if (existingMatch && existingMatch.index !== undefined) {
    const existingLocals = collectDestructuredLocalNames(existingMatch[1]);
    const missingSpecifiers = usedSpecifiers.filter((specifier) => {
      return !existingLocals.has(getLocalAlias(specifier));
    });

    if (missingSpecifiers.length === 0) {
      return code;
    }

    const merged = existingMatch[1].trim();
    const nextDestructure = merged
      ? `${merged}, ${missingSpecifiers.join(", ")}`
      : missingSpecifiers.join(", ");
    const matchStart = setupBodyStart + existingMatch.index;
    const matchEnd = matchStart + existingMatch[0].length;

    return (
      code.slice(0, matchStart) +
      `const { ${nextDestructure} } = useI18n();` +
      code.slice(matchEnd)
    );
  }

  return (
    code.slice(0, setupBodyStart) +
    `\nconst { ${usedSpecifiers.join(", ")} } = useI18n();\n` +
    code.slice(setupBodyStart)
  );
}
