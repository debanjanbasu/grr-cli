/**
 * The terminal demo cast's own duration, read from its header.
 *
 * Single source of truth: `demo/demo.cast` at the repo root — the file
 * `demo.yml` regenerates on every CLI or version change, and the same file
 * `DemoPlayer` fetches and plays. Prose that quotes a duration imports this
 * instead of typing a number, so re-recording the cast cannot leave the
 * caption behind at an old length.
 */
import demoCastRaw from '../../../demo/demo.cast?raw';

const header: unknown = JSON.parse(demoCastRaw.split('\n')[0]);
const castDuration =
  header && typeof header === 'object' && 'duration' in header && typeof header.duration === 'number'
    ? header.duration
    : 0;

/** The cast length in whole seconds, e.g. 36 — what the demo prose quotes. */
export const demoDurationSeconds = Math.round(castDuration);
