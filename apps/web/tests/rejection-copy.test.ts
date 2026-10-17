import assert from 'node:assert/strict';
import { test } from 'node:test';
import { rejectionCopy } from '../src/components/rejection-copy.ts';

test('rejected framing explains a recovery action instead of exposing only an engine code', () => {
  const copy = rejectionCopy('frame_exceeds_scale_limits');
  assert.match(copy, /Aumenta la escala máxima/);
  assert.match(copy, /vuelve a generar/);
  assert.doesNotMatch(copy, /frame_exceeds_scale_limits/);
});

test('future and object-property reason codes retain a safe actionable fallback', () => {
  for (const reason of ['future_engine_reason', 'toString', '__proto__']) {
    const copy = rejectionCopy(reason);
    assert.equal(typeof copy, 'string');
    assert.match(copy, /consulta el código en Información de la variante/);
    assert.ok(!copy.includes(reason));
  }
});
