import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatTime(date: Date | string): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return d.toLocaleTimeString("en-MY", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

export function formatDateTime(date: Date | string): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return d.toLocaleString("en-MY", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    day: "numeric",
    month: "short",
  });
}

export function formatRelativeTime(date: Date | string): string {
  const d = typeof date === "string" ? new Date(date) : date;
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  return formatDateTime(d);
}

export function truncateId(id: string, prefixLength = 8): string {
  if (id.length <= prefixLength + 4) return id;
  return `${id.slice(0, prefixLength)}...${id.slice(-4)}`;
}

export function maskId(id: string): string {
  if (id.length <= 4) return "****";
  return `****${id.slice(-4)}`;
}
