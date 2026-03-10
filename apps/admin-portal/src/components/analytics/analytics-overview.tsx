"use client";

import { useQuery } from "@tanstack/react-query";
import { TrendingUp, TrendingDown, Users, DoorOpen, AlertTriangle, Clock } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface OverviewStat {
  title: string;
  value: string | number;
  change: number;
  period: string;
  icon: React.ElementType;
}

async function fetchOverviewStats(): Promise<OverviewStat[]> {
  // TODO: Replace with actual API call
  return [
    {
      title: "Total Entries",
      value: "24,847",
      change: 12.5,
      period: "vs last month",
      icon: DoorOpen,
    },
    {
      title: "Unique Residents",
      value: "1,156",
      change: 3.2,
      period: "vs last month",
      icon: Users,
    },
    {
      title: "Denial Rate",
      value: "2.3%",
      change: -0.8,
      period: "vs last month",
      icon: AlertTriangle,
    },
    {
      title: "Avg Response Time",
      value: "142ms",
      change: -12.0,
      period: "vs last month",
      icon: Clock,
    },
  ];
}

function OverviewSkeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {Array.from({ length: 4 }).map((_, i) => (
        <Card key={i}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <div className="h-4 w-20 animate-pulse rounded bg-muted" />
            <div className="h-4 w-4 animate-pulse rounded bg-muted" />
          </CardHeader>
          <CardContent>
            <div className="h-8 w-24 animate-pulse rounded bg-muted" />
            <div className="mt-2 h-3 w-32 animate-pulse rounded bg-muted" />
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export function AnalyticsOverview() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ["analytics-overview"],
    queryFn: fetchOverviewStats,
    staleTime: 60_000,
  });

  if (isLoading) {
    return <OverviewSkeleton />;
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {stats?.map((stat) => (
        <Card key={stat.title}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              {stat.title}
            </CardTitle>
            <stat.icon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stat.value}</div>
            <div className="flex items-center gap-1 text-xs">
              {stat.change > 0 ? (
                <TrendingUp className="h-3 w-3 text-success" />
              ) : (
                <TrendingDown className="h-3 w-3 text-error" />
              )}
              <span
                className={cn(
                  stat.change > 0 ? "text-success" : "text-error"
                )}
              >
                {stat.change > 0 ? "+" : ""}
                {stat.change}%
              </span>
              <span className="text-muted-foreground">{stat.period}</span>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
