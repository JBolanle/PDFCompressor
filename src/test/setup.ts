import "@testing-library/jest-dom";

// happy-dom doesn't implement the Web Animations API, which Svelte transitions use internally.
// The stub must fire onfinish/addEventListener('finish') so Svelte removes elements after out-transitions.
Element.prototype.animate = function () {
  let _onfinish: (() => void) | null = null;
  const animation = {
    finished: Promise.resolve(),
    cancel: () => {},
    get onfinish() { return _onfinish; },
    set onfinish(fn: (() => void) | null) {
      _onfinish = fn;
      if (fn) Promise.resolve().then(() => fn());
    },
    addEventListener: (_event: string, cb: () => void) => {
      Promise.resolve().then(() => cb());
    },
    removeEventListener: () => {},
  };
  return animation as unknown as Animation;
};
