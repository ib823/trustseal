"use client";

import { useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import { Search, Users, Clock, CheckCircle, XCircle } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  useVisitorStore,
  type Visitor,
  type VisitorStatus,
} from "@/lib/stores/visitor-store";
import { VisitorDetailDialog } from "./visitor-detail-dialog";
import { cn } from "@/lib/utils";

const AUTO_REFRESH_INTERVAL = 30000; // 30 seconds

export function VisitorQueue() {
  const t = useTranslations();
  const { visitors, filter, setFilter, fetchVisitors, isLoading, error } =
    useVisitorStore();
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<"newest" | "oldest" | "name">("newest");
  const [selectedVisitor, setSelectedVisitor] = useState<Visitor | null>(null);

  // Auto-refresh every 30 seconds
  useEffect(() => {
    fetchVisitors();
    const interval = setInterval(fetchVisitors, AUTO_REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [fetchVisitors]);

  const filteredVisitors = visitors
    .filter((v) => {
      if (filter !== "all" && v.status !== filter) return false;
      if (search) {
        const searchLower = search.toLowerCase();
        return (
          v.name.toLowerCase().includes(searchLower) ||
          v.hostResident.toLowerCase().includes(searchLower) ||
          v.hostUnit.toLowerCase().includes(searchLower)
        );
      }
      return true;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case "newest":
          return new Date(b.arrivedAt).getTime() - new Date(a.arrivedAt).getTime();
        case "oldest":
          return new Date(a.arrivedAt).getTime() - new Date(b.arrivedAt).getTime();
        case "name":
          return a.name.localeCompare(b.name);
        default:
          return 0;
      }
    });

  const stats = {
    total: visitors.length,
    pending: visitors.filter((v) => v.status === "pending").length,
    verified: visitors.filter((v) => v.status === "verified").length,
    denied: visitors.filter((v) => v.status === "denied").length,
  };

  const getStatusBadgeVariant = (
    status: VisitorStatus
  ): "warning" | "success" | "error" => {
    switch (status) {
      case "pending":
        return "warning";
      case "verified":
        return "success";
      case "denied":
        return "error";
    }
  };

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  return (
    <>
      <div className="flex flex-col h-full">
        {/* Stats bar */}
        <div className="flex items-center gap-4 border-b px-6 py-4">
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5 text-muted-foreground" />
            <span className="text-sm">
              {t("visitor.total")}: <strong>{stats.total}</strong>
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-warning" />
            <span className="text-sm text-warning">
              {t("visitor.pending")}: {stats.pending}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <CheckCircle className="h-4 w-4 text-success" />
            <span className="text-sm text-success">
              {t("visitor.verified")}: {stats.verified}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <XCircle className="h-4 w-4 text-error" />
            <span className="text-sm text-error">
              {t("visitor.denied")}: {stats.denied}
            </span>
          </div>
        </div>

        {/* Filters */}
        <div className="flex items-center gap-4 border-b px-6 py-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              placeholder={t("common.search")}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-10 w-full rounded-md border border-input bg-background pl-10 pr-4 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
            />
          </div>

          <Select
            value={filter}
            onValueChange={(v) => setFilter(v as VisitorStatus | "all")}
          >
            <SelectTrigger className="w-[160px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t("visitor.filter.all")}</SelectItem>
              <SelectItem value="pending">
                {t("visitor.filter.pending")}
              </SelectItem>
              <SelectItem value="verified">
                {t("visitor.filter.verified")}
              </SelectItem>
              <SelectItem value="denied">{t("visitor.filter.denied")}</SelectItem>
            </SelectContent>
          </Select>

          <Select
            value={sortBy}
            onValueChange={(v) => setSortBy(v as typeof sortBy)}
          >
            <SelectTrigger className="w-[160px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="newest">{t("visitor.sort.newest")}</SelectItem>
              <SelectItem value="oldest">{t("visitor.sort.oldest")}</SelectItem>
              <SelectItem value="name">{t("visitor.sort.name")}</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Visitor list */}
        <div className="flex-1 overflow-y-auto p-4">
          {isLoading && visitors.length === 0 ? (
            <div className="flex h-full items-center justify-center">
              <p className="text-muted-foreground">{t("common.loading")}</p>
            </div>
          ) : error ? (
            <div className="flex h-full flex-col items-center justify-center gap-4">
              <p className="text-error">{t("error.loadFailed")}</p>
              <Button variant="outline" onClick={() => fetchVisitors()}>
                {t("common.retry")}
              </Button>
            </div>
          ) : filteredVisitors.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center gap-2">
              <Users className="h-12 w-12 text-muted-foreground/50" />
              <p className="text-lg font-medium text-muted-foreground">
                {t("visitor.queueEmpty")}
              </p>
              <p className="text-sm text-muted-foreground">
                {t("visitor.queueEmptyDescription")}
              </p>
            </div>
          ) : (
            <div className="grid gap-3">
              {filteredVisitors.map((visitor) => (
                <Card
                  key={visitor.id}
                  className={cn(
                    "cursor-pointer transition-colors hover:bg-accent/50",
                    visitor.status === "pending" && "border-warning/50"
                  )}
                  onClick={() => setSelectedVisitor(visitor)}
                >
                  <CardContent className="flex items-center justify-between p-4">
                    <div className="flex items-center gap-4">
                      <div
                        className={cn(
                          "flex h-12 w-12 items-center justify-center rounded-full text-lg font-semibold",
                          visitor.status === "pending" &&
                            "bg-warning/10 text-warning",
                          visitor.status === "verified" &&
                            "bg-success/10 text-success",
                          visitor.status === "denied" && "bg-error/10 text-error"
                        )}
                      >
                        {visitor.name.charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <p className="font-medium">{visitor.name}</p>
                        <p className="text-sm text-muted-foreground">
                          {visitor.purpose} - {visitor.hostResident} (
                          {visitor.hostUnit})
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <div className="text-right">
                        <p className="text-sm text-muted-foreground">
                          {formatTime(visitor.arrivedAt)}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {t(`visitor.credentialType.${visitor.credentialType}`)}
                        </p>
                      </div>
                      <Badge variant={getStatusBadgeVariant(visitor.status)}>
                        {t(`visitor.status.${visitor.status}`)}
                      </Badge>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>

      <VisitorDetailDialog
        visitor={selectedVisitor}
        onClose={() => setSelectedVisitor(null)}
      />
    </>
  );
}
