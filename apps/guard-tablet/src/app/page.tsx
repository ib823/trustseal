"use client";

import { VisitorQueue } from "@/components/visitor/visitor-queue";
import { StatusPanel } from "@/components/visitor/status-panel";
import { Header } from "@/components/header";
import { OfflineIndicator } from "@/components/offline-indicator";

export default function GuardTabletPage() {
  return (
    <div className="flex h-screen flex-col bg-background">
      <Header />
      <OfflineIndicator />
      <main className="flex flex-1 overflow-hidden">
        {/* Left Panel - Visitor Queue (60%) */}
        <div className="flex w-[60%] flex-col border-r">
          <VisitorQueue />
        </div>

        {/* Right Panel - Status & Controls (40%) */}
        <div className="flex w-[40%] flex-col">
          <StatusPanel />
        </div>
      </main>
    </div>
  );
}
