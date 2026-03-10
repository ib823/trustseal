import { describe, it, expect } from "vitest";
import { cn, formatDate, formatDateTime, formatRelativeTime, truncateId } from "@/lib/utils";

describe("cn utility", () => {
  it("should merge class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("should handle conditional classes", () => {
    expect(cn("base", true && "conditional")).toBe("base conditional");
    expect(cn("base", false && "conditional")).toBe("base");
  });

  it("should merge tailwind classes correctly", () => {
    expect(cn("px-4 py-2", "px-6")).toBe("py-2 px-6");
  });
});

describe("formatDate", () => {
  it("should format date string", () => {
    const result = formatDate("2024-03-15T10:30:00Z");
    expect(result).toContain("2024");
    expect(result).toContain("Mar");
    expect(result).toContain("15");
  });

  it("should format Date object", () => {
    const date = new Date("2024-06-20T15:00:00Z");
    const result = formatDate(date);
    expect(result).toContain("2024");
    expect(result).toContain("Jun");
    expect(result).toContain("20");
  });
});

describe("formatDateTime", () => {
  it("should include time in output", () => {
    const result = formatDateTime("2024-03-15T10:30:00Z");
    expect(result).toContain("2024");
    expect(result).toContain("Mar");
    // Time component present (format may vary by locale)
    expect(result.length).toBeGreaterThan(10);
  });
});

describe("formatRelativeTime", () => {
  it("should return 'Just now' for recent times", () => {
    const now = new Date();
    expect(formatRelativeTime(now)).toBe("Just now");
  });

  it("should return minutes ago", () => {
    const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000);
    expect(formatRelativeTime(fiveMinutesAgo)).toBe("5m ago");
  });

  it("should return hours ago", () => {
    const threeHoursAgo = new Date(Date.now() - 3 * 60 * 60 * 1000);
    expect(formatRelativeTime(threeHoursAgo)).toBe("3h ago");
  });

  it("should return days ago", () => {
    const twoDaysAgo = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000);
    expect(formatRelativeTime(twoDaysAgo)).toBe("2d ago");
  });
});

describe("truncateId", () => {
  it("should truncate long IDs", () => {
    const id = "res_01HQ123ABCDEF456GHI";
    const result = truncateId(id);
    expect(result).toContain("...");
    expect(result.length).toBeLessThan(id.length);
  });

  it("should return short IDs unchanged", () => {
    const id = "res_01HQ";
    const result = truncateId(id);
    expect(result).toBe(id);
  });

  it("should preserve prefix and suffix", () => {
    const id = "res_01HQ123ABCDEF456GHI";
    const result = truncateId(id, 8);
    expect(result.startsWith("res_01HQ")).toBe(true);
    expect(result.endsWith("6GHI")).toBe(true);
  });
});
