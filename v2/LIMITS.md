# Explicit support limits

This is a finite native translation of the actual locked `chrono-node 2.7.8`
English casual parser, dayjs 1.11.13 plus its UTC/timezone plugins, and
javascript-time-ago 2.5.11's default **roundMinute** style. It does not claim
universal JavaScript/V8 or natural-language equivalence. The previous native
2.8.3 translation was adapted: this package extracts its own 2.7.8 grammar,
uses insertion-ordered dayjs relative additions, clamps months/years, rounds
fractional days/weeks, handles 2.7.8 year suffixes and noon certainty, and returns
all matches and their source text. No JS engine runs in the product.

Supported converted civil years are **0100 through 9999 inclusive**, in both source
and target zones, when the resulting dayjs-style reparse is representable. UTC and
fixed offsets work before 1900 and after 2099; future DST uses **tz-rs 0.7.3** with
embedded **tzdb_data 0.2.4 / IANA 2026a** TZif extra rules. No extrapolation from a
frozen chrono-tz offset, system date subprocess, OS timezone database, or JS engine
is used. chrono-tz 0.10.4 remains the zone-name type only. Both new crates have pinned
registry checksums and authentic notices; the full product passes Rust 1.85.

Pre-0100 converted years, actual historical subminute offsets, and civil overflow
past 9999 remain explicit errors with no copy output. The 31 exact input/source/
native pairs in `qa/expected-exclusions.json` are checked individually; they are
not a permission to skip another error, different error, changed output, or shorter
oracle result. `evidence/domain-checks.json` records original and native evidence.
There are no exclusions among the 911 added review-2 century/leap/recurrence probes.

The executed whole-source oracle uses Node 26.7.0 / ICU 78.3 / IANA 2026a, matching
this native data version. Version-specific regression probes include Moldova's
post-2022 EU transition hours (Europe/Chisinau) and Baja California's corrected
1953/1961–1975 DST (America/Tijuana), with source and target combinations. Native
2025b would differ for those cases, so it was replaced with pinned 2026a data.
A different JS host's ICU version can still differ on specific IANA revisions;
this finite corpus makes no universal cross-version claim. Such differences must
be recorded by exact case, never counted exact or hidden by a broad year error.
The original zone list/order still comes from timezone-support 3.1.0
(moment-timezone 0.5.39); V1 conversion itself uses Intl.

The source dayjs `.tz()` path has observable source-local Date reparsing and
15-minute source-offset rounding, including surprising output around DST. The
native conversion translates that actual path rather than substituting normal
instant-to-zone formatting. Fixed-clock cross-zone DST fixtures cover this.
JS Date-compatible gap advancement, earlier-fold choice and ISO rollover are
preserved within the supported range. ISO machine fractions and chrono ISO
fractions have different source semantics; both results survive deduplication
when they represent different instants.

Input and custom formats are bounded to 4096 UTF-8 bytes, relative quantities to
10000 units, natural matches to 128 per parser and 512 total. The regex engine is
bounded and can reject pathological input. Store size is at most 64 KiB; at most
128 adapter/SDK sessions are live. Actions and output rows are bounded by these
limits. Unknown parser cases or regex exhaustion produce errors; there is no
fallback RFC3339-only replacement or pretend AI response.

No automatic source timer exists. Dates and reference time remain memoized until
a query edit; timezone and view changes update labels without refreshing parsed
dates. Locale text is English because the source explicitly uses en-US timeago,
English chrono defaults and dayjs's unchanged English default. This does not claim
testing of real OS locale/font behavior.

Full host build/GUI, native Windows execution, MSVC binary linking, OS clipboard,
focus/paste, settings and a real installation were not run. Windows GNU cross
`cargo check` passed. MSVC `cargo check` was attempted and failed because the target
standard library is absent; no tools were installed. The host patch is unapplied.
