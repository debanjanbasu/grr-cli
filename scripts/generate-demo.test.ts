import assert from "node:assert/strict";
import test from "node:test";

// Extracted to its own module so the privacy rules are unit-testable
// independently of the whole generator (which runs at import time).
import { sanitize } from "./generate-demo.ts";

test("sanitize strips the bearer scheme word, not just its token", () => {
  // A live agent explaining "bearer auth" used to survive as the bare word
  // and trip the privacy gate (`/bearer/i.test(all)`).
  assert.equal(
    sanitize("The transport uses bearer auth"),
    "The transport uses [redacted]",
  );
  assert.equal(
    sanitize("Authorization: Bearer abc123"),
    "Authorization: [redacted]",
  );
  assert.equal(sanitize("Bearer"), "[redacted]");
  assert.doesNotMatch(sanitize("Bearer eyJhbGci"), /bearer/i);
});

test("sanitize still redacts client ids, emails and home paths", () => {
  assert.equal(sanitize("test-id.apps.googleusercontent.com"), "[redacted]");
  assert.equal(sanitize("someuser@example.com"), "[redacted]");
  assert.equal(sanitize("C:\\Users\\debanjan\\app.log"), "~");
});
