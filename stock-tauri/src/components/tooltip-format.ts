export interface OHLCValues {
  open: number;
  close: number;
  low: number;
  high: number;
}

/**
 * Extract OHLC values from ECharts candlestick tooltip value array.
 *
 * ECharts v5 may prepend a `dataIndex` to the value array, making it
 * `[dataIndex, open, close, low, high]` (5 elements) instead of the
 * documented `[open, close, low, high]` (4 elements).
 *
 * This function auto-detects the extra element by checking array length.
 */
export function extractOHLC(value: (number | null | undefined)[]): OHLCValues {
  const offset = value.length > 4 ? value.length - 4 : 0;
  return {
    open: value[offset] ?? 0,
    close: value[offset + 1] ?? 0,
    low: value[offset + 2] ?? 0,
    high: value[offset + 3] ?? 0,
  };
}
