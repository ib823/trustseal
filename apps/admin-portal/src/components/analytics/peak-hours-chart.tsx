"use client";

import { useQuery } from "@tanstack/react-query";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface HourlyData {
  hour: string;
  entries: number;
}

async function fetchPeakHoursData(): Promise<HourlyData[]> {
  // TODO: Replace with actual API call
  return [
    { hour: "06:00", entries: 45 },
    { hour: "07:00", entries: 120 },
    { hour: "08:00", entries: 280 },
    { hour: "09:00", entries: 185 },
    { hour: "10:00", entries: 95 },
    { hour: "11:00", entries: 78 },
    { hour: "12:00", entries: 142 },
    { hour: "13:00", entries: 165 },
    { hour: "14:00", entries: 88 },
    { hour: "15:00", entries: 72 },
    { hour: "16:00", entries: 85 },
    { hour: "17:00", entries: 156 },
    { hour: "18:00", entries: 245 },
    { hour: "19:00", entries: 198 },
    { hour: "20:00", entries: 125 },
    { hour: "21:00", entries: 78 },
    { hour: "22:00", entries: 45 },
    { hour: "23:00", entries: 28 },
  ];
}

function ChartSkeleton() {
  return (
    <div className="h-[300px] w-full animate-pulse rounded bg-muted/50" />
  );
}

export function PeakHoursChart() {
  const { data, isLoading } = useQuery({
    queryKey: ["peak-hours"],
    queryFn: fetchPeakHoursData,
    staleTime: 60_000,
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Peak Hours</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <ChartSkeleton />
        ) : (
          <ResponsiveContainer width="100%" height={300}>
            <BarChart
              data={data}
              margin={{ top: 5, right: 10, left: 0, bottom: 5 }}
            >
              <CartesianGrid
                strokeDasharray="3 3"
                className="stroke-muted"
                vertical={false}
              />
              <XAxis
                dataKey="hour"
                tick={{ fontSize: 11 }}
                tickLine={false}
                axisLine={false}
                className="fill-muted-foreground"
                interval={2}
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
                formatter={(value: number) => [value, "Entries"]}
              />
              <Bar
                dataKey="entries"
                fill="hsl(var(--primary))"
                radius={[4, 4, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
