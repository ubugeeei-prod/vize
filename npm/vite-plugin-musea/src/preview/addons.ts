/**
 * Addon initialization code for Musea preview iframes.
 *
 * Contains DOM event capture, measure overlay, and message handler logic
 * injected into preview modules.
 *
 * Extracted from preview.ts to keep file sizes manageable.
 */

/**
 * Addon initialization code injected into preview iframe modules.
 * Shared between generatePreviewModule and generatePreviewModuleWithProps.
 */
export const MUSEA_ADDONS_INIT_CODE = `
function __museaInitAddons(container, variantName, extraCaptureEvents = []) {
  // === DOM event capture ===
  // Note: wheel and hover-style events are excluded by default because they are too noisy.
  const BASE_CAPTURE_EVENTS = ['click','dblclick','input','change','submit','focus','blur','keydown','keyup','mousedown','mouseup','contextmenu','pointerdown','pointerup'];
  const CAPTURE_EVENTS = [...new Set([
    ...BASE_CAPTURE_EVENTS,
    ...((Array.isArray(extraCaptureEvents) ? extraCaptureEvents : [])
      .filter((eventName) => typeof eventName === 'string')
      .map((eventName) => eventName.trim())
      .filter(Boolean))
  ])];
  for (const evt of CAPTURE_EVENTS) {
    container.addEventListener(evt, (e) => {
      // Extract raw event properties
      const rawEvent = {
        type: e.type,
        bubbles: e.bubbles,
        cancelable: e.cancelable,
        composed: e.composed,
        defaultPrevented: e.defaultPrevented,
        eventPhase: e.eventPhase,
        isTrusted: e.isTrusted,
        timeStamp: e.timeStamp,
      };
      // Mouse/Pointer event properties
      if ('clientX' in e) {
        rawEvent.clientX = e.clientX;
        rawEvent.clientY = e.clientY;
        rawEvent.screenX = e.screenX;
        rawEvent.screenY = e.screenY;
        rawEvent.pageX = e.pageX;
        rawEvent.pageY = e.pageY;
        rawEvent.offsetX = e.offsetX;
        rawEvent.offsetY = e.offsetY;
        rawEvent.button = e.button;
        rawEvent.buttons = e.buttons;
        rawEvent.altKey = e.altKey;
        rawEvent.ctrlKey = e.ctrlKey;
        rawEvent.metaKey = e.metaKey;
        rawEvent.shiftKey = e.shiftKey;
      }
      // Keyboard event properties
      if ('key' in e) {
        rawEvent.key = e.key;
        rawEvent.code = e.code;
        rawEvent.repeat = e.repeat;
        rawEvent.altKey = e.altKey;
        rawEvent.ctrlKey = e.ctrlKey;
        rawEvent.metaKey = e.metaKey;
        rawEvent.shiftKey = e.shiftKey;
      }
      // Input event properties
      if ('inputType' in e) {
        rawEvent.inputType = e.inputType;
        rawEvent.data = e.data;
      }
      // Wheel event properties
      if ('deltaX' in e) {
        rawEvent.deltaX = e.deltaX;
        rawEvent.deltaY = e.deltaY;
        rawEvent.deltaZ = e.deltaZ;
        rawEvent.deltaMode = e.deltaMode;
      }
      const payload = {
        name: e.type,
        target: e.target?.tagName,
        timestamp: Date.now(),
        source: 'dom',
        rawEvent,
        variantName
      };
      if (e.target && 'value' in e.target) {
        payload.value = e.target.value;
      }
      window.parent.postMessage({ type: 'musea:event', payload }, '*');
    }, true);
  }

  // === Message handler for parent commands ===
  let measureActive = false;
  let measureOverlay = null;
  let measureLabel = null;

  function toggleStyleById(id, enabled, css) {
    let el = document.getElementById(id);
    if (enabled) {
      if (!el) {
        el = document.createElement('style');
        el.id = id;
        el.textContent = css;
        document.head.appendChild(el);
      }
    } else {
      if (el) el.remove();
    }
  }

  function createMeasureOverlay() {
    if (measureOverlay) return;
    measureOverlay = document.createElement('div');
    measureOverlay.id = 'musea-measure-overlay';
    measureOverlay.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:99999;';
    document.body.appendChild(measureOverlay);

    measureLabel = document.createElement('div');
    measureLabel.className = 'musea-measure-label';
    measureLabel.style.cssText = 'position:fixed;background:#333;color:#fff;font-size:11px;padding:2px 6px;border-radius:3px;pointer-events:none;z-index:100000;display:none;';
    document.body.appendChild(measureLabel);
  }

  function removeMeasureOverlay() {
    if (measureOverlay) { measureOverlay.remove(); measureOverlay = null; }
    if (measureLabel) { measureLabel.remove(); measureLabel = null; }
  }

  function onMeasureMouseMove(e) {
    if (!measureActive || !measureOverlay) return;
    const el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el || el === measureOverlay || el === measureLabel) return;

    const rect = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    const mt = parseFloat(cs.marginTop) || 0;
    const mr = parseFloat(cs.marginRight) || 0;
    const mb = parseFloat(cs.marginBottom) || 0;
    const ml = parseFloat(cs.marginLeft) || 0;
    const bt = parseFloat(cs.borderTopWidth) || 0;
    const br = parseFloat(cs.borderRightWidth) || 0;
    const bb = parseFloat(cs.borderBottomWidth) || 0;
    const blw = parseFloat(cs.borderLeftWidth) || 0;
    const pt = parseFloat(cs.paddingTop) || 0;
    const pr = parseFloat(cs.paddingRight) || 0;
    const pb = parseFloat(cs.paddingBottom) || 0;
    const pl = parseFloat(cs.paddingLeft) || 0;

    const cw = rect.width - blw - br - pl - pr;
    const ch = rect.height - bt - bb - pt - pb;

    measureOverlay.innerHTML = ''
      // Margin
      + '<div style="position:fixed;background:rgba(255,165,0,0.3);'
      + 'left:' + (rect.left - ml) + 'px;top:' + (rect.top - mt) + 'px;'
      + 'width:' + (rect.width + ml + mr) + 'px;height:' + mt + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,165,0,0.3);'
      + 'left:' + (rect.left - ml) + 'px;top:' + (rect.bottom) + 'px;'
      + 'width:' + (rect.width + ml + mr) + 'px;height:' + mb + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,165,0,0.3);'
      + 'left:' + (rect.left - ml) + 'px;top:' + rect.top + 'px;'
      + 'width:' + ml + 'px;height:' + rect.height + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,165,0,0.3);'
      + 'left:' + rect.right + 'px;top:' + rect.top + 'px;'
      + 'width:' + mr + 'px;height:' + rect.height + 'px;"></div>'
      // Border
      + '<div style="position:fixed;background:rgba(255,255,0,0.3);'
      + 'left:' + rect.left + 'px;top:' + rect.top + 'px;'
      + 'width:' + rect.width + 'px;height:' + bt + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,255,0,0.3);'
      + 'left:' + rect.left + 'px;top:' + (rect.bottom - bb) + 'px;'
      + 'width:' + rect.width + 'px;height:' + bb + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,255,0,0.3);'
      + 'left:' + rect.left + 'px;top:' + (rect.top + bt) + 'px;'
      + 'width:' + blw + 'px;height:' + (rect.height - bt - bb) + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(255,255,0,0.3);'
      + 'left:' + (rect.right - br) + 'px;top:' + (rect.top + bt) + 'px;'
      + 'width:' + br + 'px;height:' + (rect.height - bt - bb) + 'px;"></div>'
      // Padding
      + '<div style="position:fixed;background:rgba(144,238,144,0.3);'
      + 'left:' + (rect.left + blw) + 'px;top:' + (rect.top + bt) + 'px;'
      + 'width:' + (rect.width - blw - br) + 'px;height:' + pt + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(144,238,144,0.3);'
      + 'left:' + (rect.left + blw) + 'px;top:' + (rect.bottom - bb - pb) + 'px;'
      + 'width:' + (rect.width - blw - br) + 'px;height:' + pb + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(144,238,144,0.3);'
      + 'left:' + (rect.left + blw) + 'px;top:' + (rect.top + bt + pt) + 'px;'
      + 'width:' + pl + 'px;height:' + (rect.height - bt - bb - pt - pb) + 'px;"></div>'
      + '<div style="position:fixed;background:rgba(144,238,144,0.3);'
      + 'left:' + (rect.right - br - pr) + 'px;top:' + (rect.top + bt + pt) + 'px;'
      + 'width:' + pr + 'px;height:' + (rect.height - bt - bb - pt - pb) + 'px;"></div>'
      // Content
      + '<div style="position:fixed;background:rgba(100,149,237,0.3);'
      + 'left:' + (rect.left + blw + pl) + 'px;top:' + (rect.top + bt + pt) + 'px;'
      + 'width:' + cw + 'px;height:' + ch + 'px;"></div>';

    // Label
    measureLabel.textContent = Math.round(rect.width) + ' x ' + Math.round(rect.height);
    measureLabel.style.display = 'block';
    measureLabel.style.left = (rect.right + 8) + 'px';
    measureLabel.style.top = rect.top + 'px';
  }

  window.addEventListener('message', (e) => {
    if (!e.data?.type?.startsWith('musea:')) return;
    const { type, payload } = e.data;
    switch (type) {
      case 'musea:set-background': {
        if (payload.pattern === 'checkerboard') {
          document.body.style.background = '';
          document.body.classList.add('musea-bg-checkerboard');
        } else {
          document.body.classList.remove('musea-bg-checkerboard');
          document.body.style.background = payload.color || '';
        }
        break;
      }
      case 'musea:toggle-outline': {
        toggleStyleById('musea-outline', payload.enabled,
          '* { outline: 1px solid rgba(255, 0, 0, 0.3) !important; }');
        break;
      }
      case 'musea:toggle-measure': {
        measureActive = payload.enabled;
        if (measureActive) {
          createMeasureOverlay();
          document.addEventListener('mousemove', onMeasureMouseMove);
        } else {
          document.removeEventListener('mousemove', onMeasureMouseMove);
          removeMeasureOverlay();
        }
        break;
      }
      case 'musea:set-props': {
        // Store props for remount - handled by preview module
        if (window.__museaSetProps) {
          window.__museaSetProps(payload.props || {});
        }
        break;
      }
      case 'musea:set-slots': {
        // Store slots for remount - handled by preview module
        if (window.__museaSetSlots) {
          window.__museaSetSlots(payload.slots || {});
        }
        break;
      }
      case 'musea:run-a11y': {
        // Run axe-core a11y test
        const requestId = typeof payload?.requestId === 'string' ? payload.requestId : undefined;
        (async () => {
          try {
            // Dynamically load axe-core from local vendor route if not already loaded
            if (!window.axe) {
              const script = document.createElement('script');
              const _basePath = location.pathname.replace(/\\/preview$/, '');
              script.src = _basePath + '/vendor/axe-core.min.js';
              await new Promise((resolve, reject) => {
                script.onload = resolve;
                script.onerror = reject;
                document.head.appendChild(script);
              });
            }
            // Run axe-core on the .musea-variant container only (not the full document)
            const context = document.querySelector('.musea-variant') || document;
            const results = await window.axe.run(context, {
              // Run all rules without restrictions for comprehensive testing
              resultTypes: ['violations', 'incomplete', 'passes']
            });
            window.parent.postMessage({
              type: 'musea:a11y-result',
              payload: {
                requestId,
                violations: results.violations.map(v => ({
                  id: v.id,
                  impact: v.impact,
                  description: v.description,
                  helpUrl: v.helpUrl,
                  nodes: v.nodes.map(n => ({
                    html: n.html,
                    target: n.target,
                    failureSummary: n.failureSummary
                  }))
                })),
                passes: results.passes.length,
                incomplete: results.incomplete.length
              }
            }, '*');
          } catch (err) {
            window.parent.postMessage({
              type: 'musea:a11y-result',
              payload: {
                requestId,
                error: err instanceof Error ? err.message : String(err),
                violations: [],
                passes: 0,
                incomplete: 0
              }
            }, '*');
          }
        })();
        break;
      }
    }
  });

  // Notify parent that iframe is ready
  window.parent.postMessage({ type: 'musea:ready', payload: {} }, '*');
}
`;
