"use client";

import { useQuery } from "@tanstack/react-query";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface ChartData {
  hour: string;
  entries: number;
  exits: number;
}

async function fetchAccessData(): Promise<ChartData[]> {
  // TODO: Replace with actual API call
  return [
    { hour: "00:00", entries: 12, exits: 8 },
    { hour: "02:00", entries: 5, exits: 3 },
    { hour: "04:00", entries: 8, exits: 6 },
    { hour: "06:00", entries: 45, exits: 22 },
    { hour: "08:00", entries: 156, exits: 180 },
    { hour: "10:00", entries: 78, exits: 65 },
    { hour: "12:00", entries: 95, exits: 88 },
    { hour: "14:00", entries: 72, exits: 80 },
    { hour: "16:00", entries: 65, exits: 58 },
    { hour: "18:00", entries: 145, exits: 120 },
    { hour: "20:00", entries: 88, exits: 95 },
    { hour: "22:00", entries: 35, exits: 28 },
  ];
}

function ChartSkeleton() {
  return (
    <div className="h-[300px] w-full animate-pulse rounded bg-muted/50" />
  );
}

export function AccessChart() {
  const { data, isLoading } = useQuery({
    queryKey: ["access-chart"],
    queryFn: fetchAccessData,
    staleTime: 60_000,
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Access Patterns</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <ChartSkeleton />
        ) : (
          <ResponsiveContainer width="100%" height={300}>
            <LineChart
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
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                className="fill-muted-foreground"
              />
              <YAxis
                tick={{ fontSize: 12 }}
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
              <Legend
                wrapperStyle={{ fontSize: "12px" }}
                iconType="circle"
                iconSize={8}
              />
              <Line
                type="monotone"
                dataKey="entries"
                stroke="hsl(var(--success))"
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 4 }}
                name="Entries"
              />
              <Line
                type="monotone"
                dataKey="exits"
                stroke="hsl(var(--primary))"
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 4 }}
                name="Exits"
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
