import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(date: Date | string, locale = "en"): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return d.toLocaleDateString(locale === "ms" ? "ms-MY" : "en-MY", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

export function formatTime(date: Date | string, locale = "en"): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return d.toLocaleTimeString(locale === "ms" ? "ms-MY" : "en-MY", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatDateTime(date: Date | string, locale = "en"): string {
  return `${formatDate(date, locale)} ${formatTime(date, locale)}`;
}

export function formatRelativeTime(date: Date | string): string {
  const d = typeof date === "string" ? new Date(date) : date;
  const now = new Date();
  const diffMs = d.getTime() - now.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) {
    return `${diffDays}d`;
  }
  if (diffHours > 0) {
    return `${diffHours}h`;
  }
  if (diffMins > 0) {
    return `${diffMins}m`;
  }
  return "now";
}

export function hashIdNumber(idNumber: string): string {
  // In production, this would use a proper hashing algorithm
  // For now, we'll create a mock hash
  const normalized = idNumber.replace(/[-\s]/g, "").toUpperCase();
  return `sha256:${btoa(normalized).slice(0, 12)}...`;
}

export function validateMalaysianIC(ic: string): boolean {
  // Malaysian IC format: YYMMDD-SS-GGGG
  const cleaned = ic.replace(/[-\s]/g, "");
  if (cleaned.length !== 12) return false;
  if (!/^\d{12}$/.test(cleaned)) return false;

  // Validate date portion (YYMMDD)
  const month = parseInt(cleaned.slice(2, 4));
  const day = parseInt(cleaned.slice(4, 6));

  if (month < 1 || month > 12) return false;
  if (day < 1 || day > 31) return false;

  return true;
}

export function validatePassport(passport: string): boolean {
  // Basic passport validation - alphanumeric, 6-9 characters
  const cleaned = passport.replace(/\s/g, "").toUpperCase();
  return /^[A-Z0-9]{6,9}$/.test(cleaned);
}
