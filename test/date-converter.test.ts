import assert from "node:assert/strict";
import test from "node:test";
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import timezone from "dayjs/plugin/timezone";
import { listTimeZones } from "timezone-support";

dayjs.extend(utc);
dayjs.extend(timezone);

test("dayjs timezone formatting works", () => {
  const date = new Date("2024-01-15T12:00:00Z");
  const formatted = dayjs(date).tz("America/New_York").format("YYYY-MM-DD");
  assert.equal(formatted, "2024-01-15");
  const iso = dayjs(date).tz("UTC").toISOString();
  assert.match(iso, /2024-01-15T12:00:00/);
});

test("timezone-support lists known zones", () => {
  const zones = listTimeZones();
  assert.ok(zones.includes("Africa/Abidjan"));
  assert.ok(zones.includes("America/New_York"));
  assert.ok(zones.length > 100);
});

test("chrono-node parses human dates", async () => {
  const chrono = await import("chrono-node");
  const results = chrono.parse("tomorrow at 3pm");
  assert.ok(results.length > 0);
  assert.ok(results[0].date() instanceof Date);
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ??
    (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  assert.equal((data as { id: string }).id, "com.tezbar.date-converter");
  assert.equal((data as { author: string }).author, "almatkai");
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/date-converter");
  assert.equal(tezbar.upstream.path, "extensions/date-converter");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  assert.equal(commands.length, 1);
  for (const cmd of commands) assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
});
