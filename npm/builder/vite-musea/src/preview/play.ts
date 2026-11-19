/** Browser-side interaction helpers injected into every isolated preview. */
export const MUSEA_PLAY_RUNTIME_CODE = String.raw`
function __museaCreatePlayContext(canvasElement) {
  const trace = [];
  const describeTarget = (element) => {
    if (!(element instanceof Element)) return undefined;
    const id = element.id ? '#' + element.id : '';
    const classes = [...element.classList].slice(0, 3).map((name) => '.' + name).join('');
    return element.tagName.toLowerCase() + id + classes;
  };
  const record = async (kind, name, target, run) => {
    const startedAt = performance.now();
    const entry = { kind, name, target: describeTarget(target), status: 'running', duration: 0 };
    trace.push(entry);
    try {
      const value = await run();
      entry.status = 'passed';
      entry.duration = Math.round((performance.now() - startedAt) * 100) / 100;
      return value;
    } catch (error) {
      entry.status = 'failed';
      entry.duration = Math.round((performance.now() - startedAt) * 100) / 100;
      entry.error = error instanceof Error ? error.message : String(error);
      throw error;
    }
  };
  const matchesText = (value, matcher) => {
    const text = String(value ?? '').replace(/\s+/g, ' ').trim();
    if (matcher instanceof RegExp) {
      matcher.lastIndex = 0;
      return matcher.test(text);
    }
    return text === String(matcher);
  };
  const accessibleName = (element) => {
    const labelledBy = element.getAttribute?.('aria-labelledby');
    if (labelledBy) {
      return labelledBy.split(/\s+/).map((id) => document.getElementById(id)?.textContent || '').join(' ');
    }
    return element.getAttribute?.('aria-label')
      || element.getAttribute?.('alt')
      || element.getAttribute?.('title')
      || element.textContent
      || '';
  };
  const implicitRole = (element) => {
    const tag = element.tagName?.toLowerCase();
    if (tag === 'button') return 'button';
    if (tag === 'a' && element.hasAttribute('href')) return 'link';
    if (tag === 'img') return 'img';
    if (tag === 'select') return 'combobox';
    if (tag === 'textarea') return 'textbox';
    if (tag === 'input') {
      const type = (element.getAttribute('type') || 'text').toLowerCase();
      if (type === 'checkbox') return 'checkbox';
      if (type === 'radio') return 'radio';
      if (type === 'button' || type === 'submit' || type === 'reset') return 'button';
      return 'textbox';
    }
    if (/^h[1-6]$/.test(tag || '')) return 'heading';
    if (tag === 'ul' || tag === 'ol') return 'list';
    if (tag === 'li') return 'listitem';
    return null;
  };
  const allElements = () => [canvasElement, ...canvasElement.querySelectorAll('*')];
  const queryAllByRole = (role, options = {}) => allElements().filter((element) => {
    const actual = element.getAttribute?.('role') || implicitRole(element);
    return actual === role && (options.name === undefined || matchesText(accessibleName(element), options.name));
  });
  const queryAllByText = (matcher) => allElements().filter((element) =>
    matchesText(element.textContent, matcher)
    && ![...element.children].some((child) => matchesText(child.textContent, matcher))
  );
  const queryAllByLabelText = (matcher) => [...canvasElement.querySelectorAll('label')]
    .filter((label) => matchesText(label.textContent, matcher))
    .map((label) => label.htmlFor ? document.getElementById(label.htmlFor) : label.querySelector('input,select,textarea,button'))
    .filter(Boolean);
  const one = (kind, matcher, values) => {
    if (values.length !== 1) throw new Error('Expected one element for ' + kind + ' ' + String(matcher) + ', found ' + values.length);
    return values[0];
  };
  const canvas = {
    element: canvasElement,
    queryByRole: (role, options) => queryAllByRole(role, options)[0] || null,
    queryAllByRole,
    getByRole: (role, options) => one('role', role, queryAllByRole(role, options)),
    getAllByRole: (role, options) => {
      const values = queryAllByRole(role, options);
      if (!values.length) throw new Error('No elements found for role ' + role);
      return values;
    },
    queryByText: (matcher) => queryAllByText(matcher)[0] || null,
    queryAllByText,
    getByText: (matcher) => one('text', matcher, queryAllByText(matcher)),
    getAllByText: (matcher) => {
      const values = queryAllByText(matcher);
      if (!values.length) throw new Error('No elements found for text ' + String(matcher));
      return values;
    },
    queryByLabelText: (matcher) => queryAllByLabelText(matcher)[0] || null,
    getByLabelText: (matcher) => one('label', matcher, queryAllByLabelText(matcher)),
    querySelector: (selector) => canvasElement.querySelector(selector),
    querySelectorAll: (selector) => [...canvasElement.querySelectorAll(selector)],
  };
  const dispatch = (element, type, init = {}) => element.dispatchEvent(new Event(type, { bubbles: true, cancelable: true, ...init }));
  const userEvent = {
    click: (element) => record('action', 'click', element, async () => { element.focus?.(); element.click(); await Promise.resolve(); }),
    dblClick: (element) => record('action', 'double click', element, async () => { element.focus?.(); element.click(); element.click(); element.dispatchEvent(new MouseEvent('dblclick', { bubbles: true })); await Promise.resolve(); }),
    clear: (element) => record('action', 'clear', element, async () => { element.focus?.(); element.value = ''; dispatch(element, 'input'); dispatch(element, 'change'); await Promise.resolve(); }),
    type: (element, text) => record('action', 'type ' + JSON.stringify(String(text)), element, async () => {
      element.focus?.();
      for (const character of String(text)) {
        element.dispatchEvent(new KeyboardEvent('keydown', { key: character, bubbles: true }));
        element.value = String(element.value ?? '') + character;
        element.dispatchEvent(new InputEvent('input', { data: character, inputType: 'insertText', bubbles: true }));
        element.dispatchEvent(new KeyboardEvent('keyup', { key: character, bubbles: true }));
        await Promise.resolve();
      }
      dispatch(element, 'change');
    }),
    selectOptions: (element, values) => record('action', 'select options', element, async () => {
      const selected = new Set((Array.isArray(values) ? values : [values]).map(String));
      for (const option of element.options || []) option.selected = selected.has(option.value);
      dispatch(element, 'input'); dispatch(element, 'change'); await Promise.resolve();
    }),
  };
  const deepEqual = (left, right) => {
    if (Object.is(left, right)) return true;
    try { return JSON.stringify(left) === JSON.stringify(right); } catch { return false; }
  };
  const createExpectation = (actual, negate = false) => {
    const check = (pass, message) => {
      const passed = negate ? !pass : pass;
      const detail = (negate ? 'Expected not: ' : 'Expected: ') + message;
      trace.push({ kind: 'assertion', name: detail, status: passed ? 'passed' : 'failed', duration: 0 });
      if (!passed) throw new Error(detail);
    };
    const api = {
      get not() { return createExpectation(actual, !negate); },
      toBe: (expected) => check(Object.is(actual, expected), String(actual) + ' to be ' + String(expected)),
      toEqual: (expected) => check(deepEqual(actual, expected), JSON.stringify(actual) + ' to equal ' + JSON.stringify(expected)),
      toBeTruthy: () => check(Boolean(actual), String(actual) + ' to be truthy'),
      toBeFalsy: () => check(!actual, String(actual) + ' to be falsy'),
      toContain: (expected) => check(actual?.includes?.(expected) === true, String(actual) + ' to contain ' + String(expected)),
      toBeInTheDocument: () => check(actual instanceof Node && document.contains(actual), 'element to be in the document'),
      toBeVisible: () => {
        const style = actual instanceof Element ? getComputedStyle(actual) : null;
        check(Boolean(style && style.display !== 'none' && style.visibility !== 'hidden' && actual.getClientRects().length), 'element to be visible');
      },
      toHaveTextContent: (expected) => check(matchesText(actual?.textContent, expected), JSON.stringify(actual?.textContent) + ' to match ' + String(expected)),
      toHaveValue: (expected) => check(Object.is(actual?.value, expected), String(actual?.value) + ' to be ' + String(expected)),
      toBeChecked: () => check(actual?.checked === true, 'element to be checked'),
      toHaveAttribute: (name, value) => check(actual?.hasAttribute?.(name) && (value === undefined || actual.getAttribute(name) === String(value)), 'element to have attribute ' + name),
      toHaveClass: (...names) => check(names.every((name) => actual?.classList?.contains(name)), 'element to have classes ' + names.join(', ')),
    };
    return api;
  };
  return {
    canvas,
    canvasElement,
    user: userEvent,
    userEvent,
    expect: (actual) => createExpectation(actual),
    step: (name, run) => record('step', String(name), undefined, run),
    __museaReport: () => ({ trace: trace.map((entry) => ({ ...entry })) }),
  };
}
`;
