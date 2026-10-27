import assert from 'node:assert/strict';
import test from 'node:test';

import { canonicalJson } from './script-utils.ts';

test('canonicalJson is stable across object key order', () => {
  const left = { z: 1, a: { y: 2, b: [{ q: 1, p: 2 }] } };
  const right = { a: { b: [{ p: 2, q: 1 }], y: 2 }, z: 1 };

  assert.equal(canonicalJson(left), canonicalJson(right));
  assert.equal(
    canonicalJson(left),
    '{"a":{"b":[{"p":2,"q":1}],"y":2},"z":1}',
  );
});

test('canonicalJson preserves array order', () => {
  assert.equal(canonicalJson(['b', 'a']), '["b","a"]');
});
