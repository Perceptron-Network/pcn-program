import assert from "node:assert/strict";
import test from "node:test";
import {
  formatDecimalUnits,
  parseDecimalUnits,
  parseUnsignedInteger,
  shortenAddress,
} from "./format";

test("formats bigint token values without floating-point loss", () => {
  assert.equal(
    formatDecimalUnits(1_234_567_890_000n, 1_000_000_000n),
    "1,234.567"
  );
  assert.equal(formatDecimalUnits(42_000_000_000n, 1_000_000_000n), "42");
});

test("parses exact decimal units and rejects excess precision", () => {
  assert.equal(parseDecimalUnits("1.25", 9, "Amount"), 1_250_000_000n);
  assert.throws(() => parseDecimalUnits("0.0000000001", 9, "Amount"));
});

test("parses unsigned integers and shortens addresses", () => {
  assert.equal(parseUnsignedInteger("216000", "Slots"), 216_000n);
  assert.throws(() => parseUnsignedInteger("-1", "Slots"));
  assert.equal(shortenAddress("1234567890", 3), "123…890");
});
