import { describe, it, expect } from "vitest";
import {
  cn,
  validateMalaysianIC,
  validatePassport,
  hashIdNumber,
} from "@/lib/utils";

describe("cn utility", () => {
  it("merges class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("handles conditional classes", () => {
    expect(cn("foo", false && "bar", "baz")).toBe("foo baz");
  });

  it("handles undefined values", () => {
    expect(cn("foo", undefined, "bar")).toBe("foo bar");
  });

  it("merges tailwind classes correctly", () => {
    expect(cn("px-2 py-1", "px-4")).toBe("py-1 px-4");
  });
});

describe("validateMalaysianIC", () => {
  it("validates correct IC format", () => {
    expect(validateMalaysianIC("900101-01-1234")).toBe(true);
    expect(validateMalaysianIC("900101011234")).toBe(true);
    expect(validateMalaysianIC("850615 14 5678")).toBe(true);
  });

  it("rejects incorrect length", () => {
    expect(validateMalaysianIC("12345")).toBe(false);
    expect(validateMalaysianIC("1234567890123")).toBe(false);
  });

  it("rejects non-numeric characters", () => {
    expect(validateMalaysianIC("90010A011234")).toBe(false);
    expect(validateMalaysianIC("ABCDEF011234")).toBe(false);
  });

  it("rejects invalid month", () => {
    expect(validateMalaysianIC("901301011234")).toBe(false); // Month 13
    expect(validateMalaysianIC("900001011234")).toBe(false); // Month 00
  });

  it("rejects invalid day", () => {
    expect(validateMalaysianIC("900132011234")).toBe(false); // Day 32
    expect(validateMalaysianIC("900100011234")).toBe(false); // Day 00
  });
});

describe("validatePassport", () => {
  it("validates correct passport format", () => {
    expect(validatePassport("A1234567")).toBe(true);
    expect(validatePassport("AB123456")).toBe(true);
    expect(validatePassport("123456789")).toBe(true);
  });

  it("rejects too short", () => {
    expect(validatePassport("A1234")).toBe(false);
  });

  it("rejects too long", () => {
    expect(validatePassport("A1234567890")).toBe(false);
  });

  it("rejects special characters", () => {
    expect(validatePassport("A123-567")).toBe(false);
    expect(validatePassport("A123@567")).toBe(false);
  });
});

describe("hashIdNumber", () => {
  it("returns a hash string", () => {
    const hash = hashIdNumber("900101011234");
    expect(hash).toMatch(/^sha256:/);
    expect(hash).toContain("...");
  });

  it("normalizes input before hashing", () => {
    const hash1 = hashIdNumber("900101-01-1234");
    const hash2 = hashIdNumber("900101011234");
    expect(hash1).toBe(hash2);
  });

  it("handles spaces in input", () => {
    const hash = hashIdNumber("900101 01 1234");
    expect(hash).toMatch(/^sha256:/);
  });
});
