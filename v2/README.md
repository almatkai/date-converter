# Date Converter — native V2 review

Standalone Rust implementation of the single **Convert Date** (`index`) command.
This is an uncommitted feature branch for parent review. It has not been installed,
published, pushed, released, or added to a catalog.

Identity: `com.tezbar.native.date-converter`; replacements are exactly
`com.tezbar.date-converter` and `raycast.date-converter`. It is a different package
from Date Format Converter. Schema 3, Rust process execution, protocol 1; independent
SDK 0.1.1 is pinned to HTTPS Git revision
`e955911f0bf5b558f9d3eb5183df0adc95c22af2` in Cargo.toml/Cargo.lock. MSRV is 1.85.

The port keeps empty-input Now/Today/Tomorrow/Yesterday, source ISO/decimal/hex/
nanosecond/microsecond parsing, English casual NLP with multiple matches and date
ranges, deduplication, all four formats and preference ordering, display/human
format strings, target timezone selection and its persistence, relative time,
and copy/paste/both behavior. There are no declared launch arguments, clipboard
reads, selection reads, AI, browser actions, history, parsed-result cache, or timers.
Copy callbacks retain the view; pasteText owns writing/hiding/focus/paste.

The original uses `useMemo` for parsing. A query edit captures a new reference;
a target-zone change reformats those same dates without reparsing them. Formatting
failures retain every parsed date and the original reference; selecting a valid
zone restores the complete output automatically, with no partial copy actions. Relative
labels are recalculated when a view is rendered, with no polling (`refreshMs:null`).
SDK preference snapshots are immutable: reopen the command after changing settings.
Timezone selection is stored privately at `dataDir/timezone/storage.json` through
a bounded atomic store; corrupt/oversize/schema/symlink errors preserve the original
file and leave the search editable. Unknown keys survive. No V1 data is imported.

Actions require a fresh view token, matching current search/zone drafts and unused
action ID. Tokens are consumed before effects. Uncertain delivery exposes an
explicit retry of the exact prepared bytes; it never reads/reparses/reclocks them.
In `both` mode, a confirmed write is checkpointed before paste; retry of uncertain
paste does not repeat that first write. Editing invalid input removes old outputs.
User text is rendered as plain List text, with no active Markdown/HTML/media.

Read [LIMITS.md](LIMITS.md), [QA.md](QA.md), [INTEGRATION.md](INTEGRATION.md) and
[PROVENANCE.json](PROVENANCE.json) before reviewing. The generic host patch in
`integration/host-prerequisites.patch` is **unapplied** and is required for safe
current-draft callbacks, timezone controls and verified alias/resource routing.
No host or SDK working file was edited or built.

Original source pin: `ad4ebc6856db6a9c554a4e4a093a319b12f66fdf`.
Original upstream pin: `6734d85992290066fd31a7f5bee3cca3d3545a1a`.
The original PNG is unchanged, SHA256:
`34d84d89c4a6e6692d94b3a8a439875ba32d9249eee592f36a9c8df4ec6e8b65`.
Source manifests intentionally have `artifacts:{}`. The ignored ARM64 ZIP is a
local development artifact, not release metadata. INDEX.json covers every
Git-visible file except itself. Product package inventory and hashes are in
`evidence/development-package.json`.

## Reproduce

Run from this clone, with installed Rust stable/1.85 and Node/npm. All dependency
installs here are disposable QA inputs under ignored `v2/.qa`, not app installation.

```sh
python3 v2/qa/bootstrap.py
node v2/qa/extract-grammar.cjs
python3 v2/qa/make-vectors.py
python3 v2/qa/round2-vectors.py
node v2/qa/source-oracle.cjs
python3 v2/qa/prepare-host-patch.py
python3 v2/qa/generate-host-tests.py
python3 v2/qa/validate.py
python3 v2/qa/isolated-build.py
python3 v2/qa/package.py
python3 v2/qa/index.py
```

`validate.py` runs production-domain, SDK adapter, pinned producer/manifest and
renderer checks with clone-local outputs. Schema QA has its own exact lockfile.
No test invokes real clipboard, browser, OS paste, settings, user profiles or AI.
