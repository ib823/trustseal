"use client";

import { useQuery } from "@tanstack/react-query";
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface DenialReason {
  reason: string;
  count: number;
  percentage: number;
}

const COLORS = [
  "hsl(var(--error))",
  "hsl(var(--warning))",
  "hsl(var(--primary))",
  "hsl(var(--muted-foreground))",
];

async function fetchDenialReasons(): Promise<DenialReason[]> {
  // TODO: Replace with actual API call
  return [
    { reason: "Credential Revoked", count: 45, percentage: 38 },
    { reason: "Credential Expired", count: 32, percentage: 27 },
    { reason: "Invalid Format", count: 25, percentage: 21 },
    { reason: "Other", count: 16, percentage: 14 },
  ];
}

function ChartSkeleton() {
  return (
    <div className="flex h-[300px] items-center justify-center">
      <div className="h-48 w-48 animate-pulse rounded-full bg-muted/50" />
    </div>
  );
}

export function DenialReasons() {
  const { data, isLoading } = useQuery({
    queryKey: ["denial-reasons"],
    queryFn: fetchDenialReasons,
    staleTime: 60_000,
  });

  const totalDenials = data?.reduce((acc, item) => acc + item.count, 0) ?? 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Denial Reasons</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <ChartSkeleton />
        ) : (
          <div className="flex flex-col items-center">
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={data}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={90}
                  paddingAngle={2}
                  dataKey="count"
                  nameKey="reason"
                >
                  {data?.map((_, index) => (
                    <Cell
                      key={`cell-${index}`}
                      fill={COLORS[index % COLORS.length]}
                    />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--card))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "8px",
                    fontSize: "12px",
                  }}
                  formatter={(value: number, name: string) => [
                    `${value} (${((value / totalDenials) * 100).toFixed(0)}%)`,
                    name,
                  ]}
                />
              </PieChart>
            </ResponsiveContainer>
            <div className="grid w-full grid-cols-2 gap-2 pt-4">
              {data?.map((item, index) => (
                <div key={item.reason} className="flex items-center gap-2 text-sm">
                  <div
                    className="h-3 w-3 rounded-full"
                    style={{ backgroundColor: COLORS[index % COLORS.length] }}
                  />
                  <span className="truncate text-muted-foreground">
                    {item.reason}
                  </span>
                  <span className="ml-auto font-medium">{item.count}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
