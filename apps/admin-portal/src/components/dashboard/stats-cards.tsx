"use client";

import { useQuery } from "@tanstack/react-query";
import { Users, DoorOpen, AlertTriangle, Cpu } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface StatCard {
  title: string;
  value: string | number;
  change?: string;
  changeType?: "positive" | "negative" | "neutral";
  icon: React.ElementType;
}

async function fetchStats(): Promise<StatCard[]> {
  // TODO: Replace with actual API call
  return [
    {
      title: "Total Residents",
      value: "1,284",
      change: "+12 this month",
      changeType: "positive",
      icon: Users,
    },
    {
      title: "Today's Entries",
      value: "847",
      change: "+5.2% vs yesterday",
      changeType: "positive",
      icon: DoorOpen,
    },
    {
      title: "Denied Access",
      value: "23",
      change: "-8% vs yesterday",
      changeType: "positive",
      icon: AlertTriangle,
    },
    {
      title: "Active Verifiers",
      value: "12/12",
      change: "All online",
      changeType: "positive",
      icon: Cpu,
    },
  ];
}

function StatsCardSkeleton() {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div className="h-4 w-24 animate-pulse rounded bg-muted" />
        <div className="h-4 w-4 animate-pulse rounded bg-muted" />
      </CardHeader>
      <CardContent>
        <div className="h-8 w-20 animate-pulse rounded bg-muted" />
        <div className="mt-1 h-3 w-28 animate-pulse rounded bg-muted" />
      </CardContent>
    </Card>
  );
}

export function StatsCards() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ["dashboard-stats"],
    queryFn: fetchStats,
    staleTime: 30_000,
  });

  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <StatsCardSkeleton key={i} />
        ))}
      </div>
    );
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
            {stat.change && (
              <p
                className={cn(
                  "text-xs",
                  stat.changeType === "positive" && "text-success",
                  stat.changeType === "negative" && "text-error",
                  stat.changeType === "neutral" && "text-muted-foreground"
                )}
              >
                {stat.change}
              </p>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
