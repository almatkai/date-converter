# Executed review validation

The tests ran in this clone on macOS ARM64. All product behavior tests use synthetic
host producers; no real profile, clipboard, settings window, browser, foreground
application, Downloads, credentials or live AI API was touched.

- Rust **1.98.0 stable** and **1.85.0 MSRV**: all-target tests and strict all-target
  Clippy passed; formatting check passed. Thirteen meaningful Rust tests cover
  production core, fixed-clock adapter, closed ACKs, private storage bounds/schema/
  unknown keys/symlinks, actual pinned host producer/manifest/hello/resource extracts,
  verified alias policy, and platform coverage including a Windows policy fixture.
- **2,239 whole-source oracle vectors**: **2,208 exact**, **31 case-specific exclusions**, **zero unresolved differences**. `source-vectors.json` contains authentic
  outputs of the whole pinned own TSX with its exact root lockfile dependencies.
  `domain-checks.json` retains original and native results for every support-limit
  case. Source references are fixed rather than current-wall-clock snapshots. The 911
  review-2 probes are all exact, including 1899/1900, 2099/2100, leap 2100/2400,
  2500 and civil bounds, UTC/fixed-offset inputs, future ns/μs and natural language,
  all four outputs and preference formats/orders, winter/summer and local gap/fold
  in New York/Berlin/Lord Howe with source/target combinations. The remaining 31
  exclusions compare the complete expected source and native result for each
  listed case; no error-prefix skip is allowed and output/corpus lengths are asserted.
  Native IANA 2026a matches the executed Node 26.7.0/ICU 78.3 oracle; Chisinau and
  Tijuana probes verify known changes since 2025b.
- Six source timezones: UTC, America/New_York, Europe/Berlin, Asia/Kathmandu,
  Australia/Lord_Howe, Pacific/Apia; additional target zones include Kiritimati,
  Chatham and Etc/GMT+12. Coverage includes source-local gap/fold and cross-zone DST
  reparsing, date rollover/leap days, empty/untrimmed/invalid/safe-text input, multi
  matches/ranges, English casual grammar, fractional durations, numeric thresholds,
  hex, ns/μs truncation, partial decimal parseInt, invalid ISO fields, ISO fractions,
  custom tokens/literals/week-year tokens, all four formats, all preference orders,
  copy/paste/both, relative-time threshold boundaries and date extremes. Explicit
  English source behavior is used independently of OS locale.
- Actual SDK JSONL pipes with the production executable: 11 scenario groups,
  covering success/error parsing and editing, A–B–A/current-draft/stale action
  rejection, strict ACKs, exact-byte uncertain retry, confirmed-write/paste checkpoint,
  cancellation and late ACK, EOF, 140 canceled opens, capacity 128 and recovery,
  timezone persistence, unknown keys, corruption and separate profile roots. Review-2
  adds error→valid restoration without reparse, persisted unsupported zones,
  historical offsets and target-year overflow, atomic multiple results, error refresh,
  repeated Now toggles with identical parsed dates, stale actions and zero effects.
  The same tests pass against the extracted release executable from the dev ZIP.
- Actual pinned renderer ListRuntime/ExtensionRuntimeSurface/ExtensionRuntimeView
  functions run in closed before/after fixtures: controlled timezone/large select,
  immediate search/dropdown draft guards, A–B–A, delayed timezone fields, correct
  empty feedback, and discarded search/action response fences. Plain row text has
  no active HTML rendering. Actual pinned Raycast worker copy/paste functions and
  Rust platform return producers run with inert clipboard/OS boundaries.
- Exact pinned schema validation plus 10 negatives passed. Extracted actual Rust
  manifest/hello validators and negatives passed. No fabricated generic ACK is used
  as the default producer response; tests default to extracted producer results.
- V1 `npm test` (4 tests) and TypeScript typecheck passed in an ignored disposable
  exact-root-lock install; root lock/package/source files remain byte exact.
- Independent release build passed with a **fresh clone-local Cargo home**, HTTPS
  SDK Git revision, standalone source copy and no sibling path dependency. Native
  ARM64 archive has a closed inventory (see exact member count in development-package.json), exact member
  hashes, verified binary hash and Mach-O ARM64 identity. `artifacts:{}` remains
  development-only; ZIP/targets/npm/cache/output trees are ignored.
- Windows GNU `cargo check --locked --target x86_64-pc-windows-gnu` passed. MSVC
  check was attempted and failed with E0463 (target core/std absent). No target or
  tool was installed. Windows linking/execution/GUI/fonts, real macOS GUI/OS locale,
  app build/install, and applied host-patch acceptance are **not run**.

The source oracle bundles the unmodified whole own TSX and pinned locked libraries.
Inside the VM, unmocked fs/os/process/network/browser/clipboard/settings access is
unavailable; string/wasm code generation is disabled. Synthetic producers record
exact source action payloads/order. Rust domain tests call the same production
core as the adapter. Metadata switches are not counted as domain QA.

Evidence: `validation.json`, `domain-checks.json`, `source-vectors.json`,
`process-checks.json`, `extracted-process-checks.json`, `renderer-checks.json`,
`worker-copy-checks.json`, `v1-checks.json`, `windows-attempts.json`,
`isolated-build.json`, `development-package.json`, `rust-notices.json`.
Private generated inputs/outputs are reproducible from checked-in scripts, own
source/lock, pinned host/SDK/locked-source fixtures and resources.

## XERJ coverage

Only this clone used prefix `ax-59ae786eda6c` and its own state directory
`v2/.qa/xerj-state`. Initial indexing covered 4/4 original code files, with assets
reported as junk/skipped; initial `ConvertDate` search had no match. Final indexing
was attempted once on the same prefix. It aborted (exit 1) because the frozen
dataset/schema could not accept the new Cargo.toml docs fields. **Final coverage is
incomplete**; the original generation is not evidence of V2 coverage. No alternate
prefix, global restart, schema mutation or server reconfiguration was attempted.
The recorded error was schema evolution, not an observed 429 or RSS failure; memory
availability was reported unknown by XERJ. Exact file reads, `rg`, Git diffs and the
whole Git-visible hash inventory provide the final checks. Full log and search
result are in `evidence/xerj.json`.

Final Git verification requires unchanged HEAD, empty index, status consisting only
of README modification and untracked v2/, unchanged V1 hashes except the appended
README link, and INDEX.json covering every Git-visible file except itself. The
clone remains intact for parent review. No commit/push/PR/release/install/deletion
or sibling/host/SDK/catalog/ledger modification is part of this handoff.
