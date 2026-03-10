"use client";

import { useQuery } from "@tanstack/react-query";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface TrendData {
  date: string;
  entries: number;
  exits: number;
}

async function fetchEntryTrends(): Promise<TrendData[]> {
  // TODO: Replace with actual API call
  const data: TrendData[] = [];
  const now = new Date();

  for (let i = 29; i >= 0; i--) {
    const date = new Date(now);
    date.setDate(date.getDate() - i);

    const baseEntries = 800 + Math.floor(Math.random() * 200);
    const dayOfWeek = date.getDay();
    const weekendFactor = dayOfWeek === 0 || dayOfWeek === 6 ? 0.7 : 1;

    data.push({
      date: date.toLocaleDateString("en-MY", {
        month: "short",
        day: "numeric",
      }),
      entries: Math.floor(baseEntries * weekendFactor),
      exits: Math.floor(baseEntries * weekendFactor * 0.95),
    });
  }

  return data;
}

function ChartSkeleton() {
  return (
    <div className="h-[350px] w-full animate-pulse rounded bg-muted/50" />
  );
}

export function EntryTrends() {
  const { data, isLoading } = useQuery({
    queryKey: ["entry-trends"],
    queryFn: fetchEntryTrends,
    staleTime: 60_000,
  });

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Entry Trends</CardTitle>
        <div className="flex gap-2">
          <Button variant="outline" size="sm">
            7 Days
          </Button>
          <Button variant="default" size="sm">
            30 Days
          </Button>
          <Button variant="outline" size="sm">
            90 Days
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <ChartSkeleton />
        ) : (
          <ResponsiveContainer width="100%" height={350}>
            <AreaChart
              data={data}
              margin={{ top: 5, right: 10, left: 0, bottom: 5 }}
            >
              <defs>
                <linearGradient id="colorEntries" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor="hsl(var(--success))"
                    stopOpacity={0.3}
                  />
                  <stop
                    offset="95%"
                    stopColor="hsl(var(--success))"
                    stopOpacity={0}
                  />
                </linearGradient>
                <linearGradient id="colorExits" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor="hsl(var(--primary))"
                    stopOpacity={0.3}
                  />
                  <stop
                    offset="95%"
                    stopColor="hsl(var(--primary))"
                    stopOpacity={0}
                  />
                </linearGradient>
              </defs>
              <CartesianGrid
                strokeDasharray="3 3"
                className="stroke-muted"
                vertical={false}
              />
              <XAxis
                dataKey="date"
                tick={{ fontSize: 11 }}
                tickLine={false}
                axisLine={false}
                className="fill-muted-foreground"
                interval={4}
              />
              <YAxis
                tick={{ fontSize: 11 }}
                tickLine={false}
                axisLine={false}
                className="fill-muted-foreground"
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: "hsl(var(--card))",
                  border: "1px solid hsl(var(--border))",
                  borderRadius: "8px",
                  fontSize: "12px",
                }}
                labelStyle={{ color: "hsl(var(--foreground))" }}
              />
              <Area
                type="monotone"
                dataKey="entries"
                stroke="hsl(var(--success))"
                strokeWidth={2}
                fillOpacity={1}
                fill="url(#colorEntries)"
                name="Entries"
              />
              <Area
                type="monotone"
                dataKey="exits"
                stroke="hsl(var(--primary))"
                strokeWidth={2}
                fillOpacity={1}
                fill="url(#colorExits)"
                name="Exits"
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
