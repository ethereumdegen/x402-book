/**
 * Token formatting utilities
 *
 * Token values are stored as raw 256-bit values with 18 decimals.
 * These utilities normalize them for human-readable display.
 */

const TOKEN_DECIMALS = 18n;
const DIVISOR = 10n ** TOKEN_DECIMALS;

/**
 * Normalize a raw token value string (18 decimals) to a human-readable number.
 * e.g., "1000000000000000000000" (1000 * 10^18) -> "1,000"
 */
export function formatTokenAmount(rawValue: string | undefined | null): string {
  if (!rawValue) return '0';

  try {
    const value = BigInt(rawValue);
    const normalized = value / DIVISOR;
    return normalized.toLocaleString();
  } catch {
    // If parsing fails, try to display as-is (for backwards compatibility with old data)
    const num = Number(rawValue);
    if (!isNaN(num)) {
      return num.toLocaleString();
    }
    return '0';
  }
}

/**
 * Get the raw normalized number (for sorting, comparison, etc.)
 */
export function normalizeTokenAmount(rawValue: string | undefined | null): number {
  if (!rawValue) return 0;

  try {
    const value = BigInt(rawValue);
    const normalized = value / DIVISOR;
    return Number(normalized);
  } catch {
    // If parsing fails, try to parse as-is
    const num = Number(rawValue);
    return isNaN(num) ? 0 : num;
  }
}
