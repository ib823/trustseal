"use client";

import { useState } from "react";
import { Building2, Shield, Bell, Clock, Users } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

const tabs = [
  { id: "property", label: "Property", icon: Building2 },
  { id: "security", label: "Security", icon: Shield },
  { id: "notifications", label: "Notifications", icon: Bell },
  { id: "schedule", label: "Schedule", icon: Clock },
  { id: "team", label: "Team", icon: Users },
];

export function SettingsTabs() {
  const [activeTab, setActiveTab] = useState("property");

  return (
    <div className="flex flex-col gap-6 lg:flex-row">
      {/* Tab Navigation */}
      <nav className="flex flex-row gap-1 lg:w-48 lg:flex-col lg:gap-2">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
              activeTab === tab.id
                ? "bg-primary/10 text-primary"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
          >
            <tab.icon className="h-4 w-4" />
            <span className="hidden lg:inline">{tab.label}</span>
          </button>
        ))}
      </nav>

      {/* Tab Content */}
      <div className="flex-1">
        {activeTab === "property" && <PropertySettings />}
        {activeTab === "security" && <SecuritySettings />}
        {activeTab === "notifications" && <NotificationSettings />}
        {activeTab === "schedule" && <ScheduleSettings />}
        {activeTab === "team" && <TeamSettings />}
      </div>
    </div>
  );
}

function PropertySettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Property Information</CardTitle>
        <CardDescription>
          Basic information about your property
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <label className="text-sm font-medium">Property Name</label>
            <Input defaultValue="Sunway Geo Residences" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Property Code</label>
            <Input defaultValue="SGR" disabled className="bg-muted" />
          </div>
        </div>
        <div className="space-y-2">
          <label className="text-sm font-medium">Address</label>
          <Input defaultValue="Jalan Lagoon Selatan, Bandar Sunway, 47500 Subang Jaya, Selangor" />
        </div>
        <div className="grid gap-4 sm:grid-cols-3">
          <div className="space-y-2">
            <label className="text-sm font-medium">Total Units</label>
            <Input type="number" defaultValue="1284" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Total Blocks</label>
            <Input type="number" defaultValue="4" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Total Floors</label>
            <Input type="number" defaultValue="35" />
          </div>
        </div>
        <div className="flex justify-end pt-4">
          <Button>Save Changes</Button>
        </div>
      </CardContent>
    </Card>
  );
}

function SecuritySettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Security Settings</CardTitle>
        <CardDescription>
          Configure security policies and access control
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <h4 className="text-sm font-medium">Credential Policy</h4>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                Default Credential Validity
              </label>
              <Input type="number" defaultValue="365" />
              <p className="text-xs text-muted-foreground">Days</p>
            </div>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                Status List Refresh Interval
              </label>
              <Input type="number" defaultValue="900" />
              <p className="text-xs text-muted-foreground">Seconds</p>
            </div>
          </div>
        </div>

        <div className="space-y-4">
          <h4 className="text-sm font-medium">Brute Force Protection</h4>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                Max Failed Attempts
              </label>
              <Input type="number" defaultValue="5" />
            </div>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                Lockout Duration
              </label>
              <Input type="number" defaultValue="300" />
              <p className="text-xs text-muted-foreground">Seconds</p>
            </div>
          </div>
        </div>

        <div className="flex justify-end pt-4">
          <Button>Save Changes</Button>
        </div>
      </CardContent>
    </Card>
  );
}

function NotificationSettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Notification Preferences</CardTitle>
        <CardDescription>
          Configure how you receive alerts and notifications
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          {[
            { label: "Security alerts", description: "Unusual access patterns, brute force attempts" },
            { label: "Device status", description: "Verifier offline/degraded notifications" },
            { label: "Daily summary", description: "Daily access statistics report" },
            { label: "Credential events", description: "New issuances, revocations, expirations" },
          ].map((item) => (
            <div key={item.label} className="flex items-start justify-between rounded-lg border p-4">
              <div>
                <p className="font-medium">{item.label}</p>
                <p className="text-sm text-muted-foreground">{item.description}</p>
              </div>
              <label className="relative inline-flex cursor-pointer items-center">
                <input type="checkbox" defaultChecked className="peer sr-only" />
                <div className="peer h-6 w-11 rounded-full bg-muted after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-primary peer-checked:after:translate-x-full"></div>
              </label>
            </div>
          ))}
        </div>
        <div className="flex justify-end pt-4">
          <Button>Save Changes</Button>
        </div>
      </CardContent>
    </Card>
  );
}

function ScheduleSettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Access Schedule</CardTitle>
        <CardDescription>
          Configure default access hours and restrictions
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          <h4 className="text-sm font-medium">Facility Access Hours</h4>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">Gym</label>
              <div className="flex gap-2">
                <Input type="time" defaultValue="06:00" />
                <span className="flex items-center text-muted-foreground">to</span>
                <Input type="time" defaultValue="22:00" />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">Pool</label>
              <div className="flex gap-2">
                <Input type="time" defaultValue="07:00" />
                <span className="flex items-center text-muted-foreground">to</span>
                <Input type="time" defaultValue="21:00" />
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-4">
          <h4 className="text-sm font-medium">Visitor Access Hours</h4>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">Weekdays</label>
              <div className="flex gap-2">
                <Input type="time" defaultValue="08:00" />
                <span className="flex items-center text-muted-foreground">to</span>
                <Input type="time" defaultValue="22:00" />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">Weekends</label>
              <div className="flex gap-2">
                <Input type="time" defaultValue="09:00" />
                <span className="flex items-center text-muted-foreground">to</span>
                <Input type="time" defaultValue="21:00" />
              </div>
            </div>
          </div>
        </div>

        <div className="flex justify-end pt-4">
          <Button>Save Changes</Button>
        </div>
      </CardContent>
    </Card>
  );
}

function TeamSettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Team Members</CardTitle>
        <CardDescription>
          Manage admin portal access for your team
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          {[
            { name: "Admin User", email: "admin@sunwaygeo.my", role: "Owner" },
            { name: "Security Lead", email: "security@sunwaygeo.my", role: "Admin" },
            { name: "Guard Supervisor", email: "supervisor@sunwaygeo.my", role: "Viewer" },
          ].map((member) => (
            <div key={member.email} className="flex items-center justify-between rounded-lg border p-4">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-sm font-medium text-primary">
                  {member.name.split(" ").map((n) => n[0]).join("")}
                </div>
                <div>
                  <p className="font-medium">{member.name}</p>
                  <p className="text-sm text-muted-foreground">{member.email}</p>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <span className="text-sm text-muted-foreground">{member.role}</span>
                <Button variant="ghost" size="sm">
                  Edit
                </Button>
              </div>
            </div>
          ))}
        </div>
        <div className="flex justify-between pt-4">
          <Button variant="outline">Invite Member</Button>
        </div>
      </CardContent>
    </Card>
  );
}
