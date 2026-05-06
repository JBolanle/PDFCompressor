import { readFileSync } from 'fs';
import { resolve } from 'path';
import { describe, it, expect } from 'vitest';

const css = readFileSync(resolve(__dirname, '../app.css'), 'utf-8');

const lightBlock = (() => {
  const match = css.match(/@media\s*\(prefers-color-scheme:\s*light\)\s*\{([\s\S]*?)\n\}/);
  return match ? match[1] : '';
})();

describe('light theme media query', () => {
  it('defines a prefers-color-scheme: light block', () => {
    expect(css).toMatch(/prefers-color-scheme:\s*light/);
  });

  const colorTokens = [
    '--bg-primary',
    '--bg-secondary',
    '--bg-tertiary',
    '--bg-overlay',
    '--text-primary',
    '--text-secondary',
    '--text-tertiary',
    '--border',
    '--border-subtle',
    '--accent',
    '--accent-hover',
    '--accent-muted',
    '--success',
    '--error',
    '--warning',
  ];

  for (const token of colorTokens) {
    it(`overrides ${token} in the light block`, () => {
      expect(lightBlock).toContain(token);
    });
  }
});
