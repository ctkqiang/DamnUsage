import type { CurrencyCode } from "../cqrs/types";

export function formatMoney(
  usdAmount: number,
  currency: CurrencyCode,
  usdToCnyRate: number,
): string {
  const symbol = currency === "CNY" ? "¥" : "$";
  const value = currency === "CNY" ? usdAmount * usdToCnyRate : usdAmount;
  if (value >= 1000) {
    return `${symbol}${value.toLocaleString(undefined, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    })}`;
  }
  return `${symbol}${value.toFixed(2)}`;
}

export function formatCompactNumber(value: number): string {
  if (value >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(2)}B`;
  }
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return `${Math.round(value)}`;
}
