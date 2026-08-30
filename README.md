# Date Converter for Tezbar

Parse human or machine-readable dates and copy them in multiple formats across timezones in Tezbar on Windows or macOS.

## Command

- **Convert Date** — list view with `Enter a human or machine readable date` search, timezone dropdown (local + `timezone-support` list), live parsing via `chrono-node` and `parseMachineReadableDate` (ISO, nanosecond/µs timestamps, hex, seconds vs ms heuristics), and copy actions sorted by preferences. Detail shows local formatted time, relative `javascript-time-ago` string, and ISO.

Preferences (per command): `Preferred Format` (`unix-ms` / `unix-s` / `iso` / `human`), `Secondary Format`, `Copy Action` (`copy` / `paste` / `both` via `Clipboard`), `Default Display Time Format` and `Human-readable Time Format` (both default `dddd, MMMM D, YYYY hh:mm:ss` via `dayjs`).

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api`, `chrono-node`, `dayjs` with `utc`/`timezone` plugins, `javascript-time-ago`, and `timezone-support` (pure JS) and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `date-converter` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `asportnoy` and contributors `vineus`, `danyal`, `alex925` remain credited in `package.json`.
