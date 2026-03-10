"use client";

import { useState } from "react";
import { Search, Filter, Calendar, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

interface FilterState {
  search: string;
  status: "all" | "granted" | "denied";
  direction: "all" | "entry" | "exit";
  verifier: string;
  dateRange: "today" | "week" | "month" | "custom";
}

export function AccessLogFilters() {
  const [filters, setFilters] = useState<FilterState>({
    search: "",
    status: "all",
    direction: "all",
    verifier: "",
    dateRange: "today",
  });

  const [showFilters, setShowFilters] = useState(false);

  const activeFiltersCount = [
    filters.status !== "all",
    filters.direction !== "all",
    filters.verifier !== "",
    filters.dateRange !== "today",
  ].filter(Boolean).length;

  const clearFilters = () => {
    setFilters({
      search: "",
      status: "all",
      direction: "all",
      verifier: "",
      dateRange: "today",
    });
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="search"
            placeholder="Search by resident, unit, or verifier..."
            value={filters.search}
            onChange={(e) =>
              setFilters((prev) => ({ ...prev, search: e.target.value }))
            }
            className="pl-9"
          />
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowFilters(!showFilters)}
            className="gap-2"
          >
            <Filter className="h-4 w-4" />
            Filters
            {activeFiltersCount > 0 && (
              <Badge variant="secondary" className="ml-1 h-5 w-5 p-0 text-xs">
                {activeFiltersCount}
              </Badge>
            )}
          </Button>

          <Button variant="outline" size="sm" className="gap-2">
            <Calendar className="h-4 w-4" />
            Today
          </Button>
        </div>
      </div>

      {showFilters && (
        <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-card p-4">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Status:</span>
            <div className="flex gap-1">
              {(["all", "granted", "denied"] as const).map((status) => (
                <Button
                  key={status}
                  variant={filters.status === status ? "default" : "outline"}
                  size="sm"
                  onClick={() =>
                    setFilters((prev) => ({ ...prev, status }))
                  }
                >
                  {status === "all" ? "All" : status}
                </Button>
              ))}
            </div>
          </div>

          <div className="h-6 w-px bg-border" />

          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Direction:</span>
            <div className="flex gap-1">
              {(["all", "entry", "exit"] as const).map((direction) => (
                <Button
                  key={direction}
                  variant={filters.direction === direction ? "default" : "outline"}
                  size="sm"
                  onClick={() =>
                    setFilters((prev) => ({ ...prev, direction }))
                  }
                >
                  {direction === "all" ? "All" : direction}
                </Button>
              ))}
            </div>
          </div>

          {activeFiltersCount > 0 && (
            <>
              <div className="h-6 w-px bg-border" />
              <Button
                variant="ghost"
                size="sm"
                onClick={clearFilters}
                className="gap-1 text-muted-foreground"
              >
                <X className="h-4 w-4" />
                Clear all
              </Button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
