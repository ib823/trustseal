import "@testing-library/jest-dom";

// Mock navigator.onLine
Object.defineProperty(globalThis, "navigator", {
  value: {
    onLine: true,
  },
  writable: true,
});
