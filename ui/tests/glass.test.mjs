import test from 'node:test';
import assert from 'node:assert/strict';
import { materialFor } from '../glass.js';

const grid = color => Array.from({ length: 15 }, () => color);
const gray = value => grid([value, value, value]);

test('ink uses a hysteresis band and dark appearance always has light ink', () => {
  assert.equal(materialFor(gray(125), false).ink, 'light');
  assert.equal(materialFor(gray(130), false).ink, 'dark');
  assert.equal(materialFor(gray(130), false, 'light').ink, 'light');
  assert.equal(materialFor(gray(125), false, 'dark').ink, 'dark');
  assert.equal(materialFor(gray(145), false, 'light').ink, 'dark');
  assert.equal(materialFor(gray(110), false, 'dark').ink, 'light');
  assert.equal(materialFor(gray(255), true).ink, 'light');
});

test('outliers do not change ink but busy scenes raise material density', () => {
  const flat = gray(205), busy = [...gray(205).slice(0, 11), [0, 0, 0], [0, 0, 0], [255, 255, 255], [255, 255, 255]];
  const quiet = materialFor(flat, false), textured = materialFor(busy, false);
  assert.equal(textured.ink, quiet.ink);
  assert.ok(textured.alpha > quiet.alpha);
  assert.ok(materialFor(grid([255, 80, 40]), false).saturation < quiet.saturation);
});

test('secondary ink stays at 4.5:1 on black, white and saturated extremes at default tint', () => {
  const samples = [[0, 0, 0], [255, 255, 255], [255, 0, 0], [0, 255, 0], [0, 0, 255], ...gray(190).slice(0, 10)];
  const lum = rgb => rgb.map(c => c / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((a, c, i) => a + c * [.2126, .7152, .0722][i], 0);
  for (const dark of [false, true]) {
    const material = materialFor(samples, dark), whiteInk = material.ink === 'light';
    const ink = whiteInk ? [186, 196, 188] : [80, 91, 83], tint = whiteInk ? [20, 20, 24] : [255, 255, 255];
    for (const sample of samples) {
      const grayValue = sample.reduce((sum, c, i) => sum + c * [.213, .715, .072][i], 0);
      const backdrop = sample.map((c, i) => Math.min(255, Math.max(0, (grayValue + material.saturation * (c - grayValue)) * material.brightness)) * (1 - material.alpha) + tint[i] * material.alpha);
      for (const highlight of [0, whiteInk ? .1 : .3]) {
        const background = lum(backdrop.map(c => c * (1 - highlight) + 255 * highlight)), foreground = lum(ink);
        assert.ok((Math.max(background, foreground) + .05) / (Math.min(background, foreground) + .05) >= 4.5);
      }
    }
  }
});

test('each appearance scales tint independently and clear tint removes brightness lift', () => {
  assert.equal(materialFor(gray(210), false, undefined, 0).alpha, 0);
  assert.equal(materialFor(gray(210), false, undefined, 0).brightness, 1);
  assert.ok(materialFor(gray(210), false, undefined, 1.5).alpha > materialFor(gray(210), false).alpha);
  assert.ok(materialFor(gray(210), false, undefined, 1, true).alpha >= .92);
});
