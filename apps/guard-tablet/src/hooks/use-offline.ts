"use client";

import { useEffect } from "react";
import { useOfflineStore } from "@/lib/stores/offline-store";

export function useOnlineStatus() {
  const { isOnline, setOnline } = useOfflineStore();

  useEffect(() => {
    const handleOnline = () => setOnline(true);
    const handleOffline = () => setOnline(false);

    // Set initial state
    setOnline(navigator.onLine);

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, [setOnline]);

  return isOnline;
}

export function useQueueSync() {
  const { isOnline, queue, processQueue } = useOfflineStore();

  useEffect(() => {
    if (!isOnline || queue.length === 0) return;
    processQueue();
  }, [isOnline, queue, processQueue]);

  return {
    pendingActions: queue.length,
  };
}
