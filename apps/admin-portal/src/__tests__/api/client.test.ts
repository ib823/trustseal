import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Import after mocking
import { apiClient } from "@/lib/api/client";

describe("ApiClient", () => {
  beforeEach(() => {
    mockFetch.mockReset();
    apiClient.clearToken();
  });

  describe("GET requests", () => {
    it("should make GET request", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: "test" }),
      });

      const result = await apiClient.get("/test");

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("/test"),
        expect.objectContaining({
          method: "GET",
          headers: expect.objectContaining({
            "Content-Type": "application/json",
          }),
        })
      );
      expect(result.data).toEqual({ data: "test" });
    });

    it("should include query params", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      await apiClient.get("/test", { foo: "bar", baz: "qux" });

      const calledUrl = mockFetch.mock.calls[0][0];
      expect(calledUrl).toContain("foo=bar");
      expect(calledUrl).toContain("baz=qux");
    });
  });

  describe("POST requests", () => {
    it("should make POST request with body", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ id: "123" }),
      });

      const result = await apiClient.post("/test", { name: "Test" });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("/test"),
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ name: "Test" }),
        })
      );
      expect(result.data).toEqual({ id: "123" });
    });
  });

  describe("Authentication", () => {
    it("should include auth token when set", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      apiClient.setToken("test-token");
      await apiClient.get("/test");

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            Authorization: "Bearer test-token",
          }),
        })
      );
    });

    it("should not include auth token when cleared", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      apiClient.setToken("test-token");
      apiClient.clearToken();
      await apiClient.get("/test");

      const headers = mockFetch.mock.calls[0][1].headers;
      expect(headers.Authorization).toBeUndefined();
    });
  });

  describe("Error handling", () => {
    it("should return error for non-OK response", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ code: "SAHI_4000", message: "Bad request" }),
      });

      const result = await apiClient.get("/test");

      expect(result.error).toEqual({
        code: "SAHI_4000",
        message: "Bad request",
      });
      expect(result.data).toBeUndefined();
    });

    it("should handle network errors", async () => {
      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      const result = await apiClient.get("/test");

      expect(result.error?.code).toBe("SAHI_5001");
      expect(result.error?.message).toContain("Network error");
    });
  });
});
