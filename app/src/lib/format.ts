import { LAMPORTS_PER_SOL, TOKEN_BASE_UNITS } from "./config";

const INTEGER_PATTERN = /^\d+$/;
const DECIMAL_PATTERN = /^\d+(?:\.\d+)?$/;

export function shortenAddress(value: string, size = 4) {
  if (value.length <= size * 2 + 1) {
    return value;
  }
  return `${value.slice(0, size)}…${value.slice(-size)}`;
}

export function formatInteger(value: bigint | number) {
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 }).format(
    value,
  );
}

export function formatDecimalUnits(
  value: bigint,
  scale: bigint,
  maximumFractionDigits = 3,
) {
  const whole = value / scale;
  const remainder = value % scale;
  if (remainder === 0n || maximumFractionDigits === 0) {
    return formatInteger(whole);
  }

  const scaleDigits = scale.toString().length - 1;
  const rawFraction = remainder.toString().padStart(scaleDigits, "0");
  const fraction = rawFraction
    .slice(0, maximumFractionDigits)
    .replace(/0+$/, "");
  return fraction
    ? `${formatInteger(whole)}.${fraction}`
    : formatInteger(whole);
}

export function formatPcn(value: bigint) {
  return formatDecimalUnits(value, TOKEN_BASE_UNITS, 3);
}

export function formatSol(value: bigint) {
  return formatDecimalUnits(value, LAMPORTS_PER_SOL, 4);
}

export function parseUnsignedInteger(value: string, label: string) {
  const normalized = value.trim();
  if (!INTEGER_PATTERN.test(normalized)) {
    throw new Error(`${label} must be a non-negative whole number.`);
  }
  return BigInt(normalized);
}

export function parseDecimalUnits(
  value: string,
  decimals: number,
  label: string,
) {
  const normalized = value.trim();
  if (!DECIMAL_PATTERN.test(normalized)) {
    throw new Error(`${label} must be a non-negative number.`);
  }
  const [whole, fraction = ""] = normalized.split(".");
  if (fraction.length > decimals) {
    throw new Error(`${label} supports at most ${decimals} decimal places.`);
  }
  return (
    BigInt(whole) * 10n ** BigInt(decimals) +
    BigInt(fraction.padEnd(decimals, "0") || "0")
  );
}

export function formatPercent(value: bigint, scale = 1_000_000n) {
  return `${formatDecimalUnits(value * 100n, scale, 2)}%`;
}
