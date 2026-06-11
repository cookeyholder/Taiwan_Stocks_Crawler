import { describe, expect, test } from "vitest";
import { extractOHLC } from "./tooltip-format";

describe("extractOHLC", () => {
  test("handles ECharts v5 format with dataIndex prefix (5 elements)", () => {
    const v = [84, 24.6, 24.1, 24.0, 24.65]; // [dataIndex, open, close, low, high]
    expect(extractOHLC(v)).toEqual({ open: 24.6, close: 24.1, low: 24.0, high: 24.65 });
  });

  test("handles standard format without dataIndex (4 elements)", () => {
    const v = [24.6, 24.1, 24.0, 24.65]; // [open, close, low, high]
    expect(extractOHLC(v)).toEqual({ open: 24.6, close: 24.1, low: 24.0, high: 24.65 });
  });

  test("handles zero values", () => {
    const v = [0, 0, 0, 0];
    expect(extractOHLC(v)).toEqual({ open: 0, close: 0, low: 0, high: 0 });
  });

  test("handles decimal precision", () => {
    const v = [100.5, 101.75, 99.25, 102.0];
    expect(extractOHLC(v)).toEqual({ open: 100.5, close: 101.75, low: 99.25, high: 102.0 });
  });

  test("handles dataIndex prefix with same-length values", () => {
    const v = [5, 150.0, 152.5, 148.0, 153.0];
    expect(extractOHLC(v)).toEqual({ open: 150.0, close: 152.5, low: 148.0, high: 153.0 });
  });

  test("handles large dataIndex values", () => {
    const v = [999, 31.5, 32.0, 30.8, 32.5];
    expect(extractOHLC(v)).toEqual({ open: 31.5, close: 32.0, low: 30.8, high: 32.5 });
  });
});
