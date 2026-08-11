# Changelog

- Read this document in [Portuguese (pt-BR)](CHANGELOG.pt-BR.md).

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]
- Nothing yet.


## [0.5.5] - 2026-08-10

### Security
- **Translated text no longer reaches the JSON wire.** Nineteen call sites in SCP and SFTP built a failure out of `i18n::t(…)` — either as the body of `SshCliError::InvalidArgument` or straight into the `error` field of a batch entry — so under `--lang pt-BR` the envelope carried Portuguese. `docs/schemas/error-envelope.schema.json` states that `message` is "always English, regardless of `--lang`", because an agent branching on prose that moves with the operator's locale has no stable discriminator at all. Two of those sites wrote to **stdout**: a cancelled `scp --all` put `Operação cancelada pelo usuário.` in the batch payload. The three phrases now live in `crate::constants` as plain `&str`, and `Message::OperationCancelled`, `ScpUploadFileOnly` and `ScpDownloadLocalNotDirectory` were removed — a `Message` variant is a promise the string is safe to localize, and for these three it never was. The human-facing *label* around each body is still translated. The existing gate checked the emitter, which did nothing wrong; the new one checks the constructors, because an invariant that can be broken before the checkpoint has to be tested at the source.
- **BREAKING — `ssh_cli::telemetry` is now `ssh_cli::tracing_setup`.** The module never emitted telemetry — its own documentation has ruled out OTLP, metrics backends and log files since the first commit — but a reader auditing the tree against this project's telemetry ban found a module by that name and had to read it to learn it was innocent. A name that costs every reader a verification is wrong however defensible its history. No deprecated alias is provided: an alias would preserve the name the rename exists to remove. The wire field `"telemetry": false` in `vps doctor` keeps its name, where the word is the claim being denied.
- **BREAKING — `health-check` now demands an explicit target.** It used to probe the active marker whenever the name was omitted, defended as safe because a connectivity probe is an idempotent read. The defence is true about the probe and false about the habit: an operator who learns a target is optional on the read surface carries that expectation to `exec`, which is where GAP-SSH-EXEC-ARGC-001 cost a day of work. Ergonomic asymmetry between a read and a write is a trap, and the read is the cheaper half to make strict. The exit taxonomy is now identical across both families: **64** means no target was designated, **66** means the designated target does not exist. This narrows the older `GAP-SSH-EXIT-002` contract, which mapped a nameless `health-check` to 66 and thereby conflated "you named nothing" with "what you named is absent". Migration: `ssh-cli health-check` becomes `ssh-cli health-check --use-active` or, preferably, `ssh-cli health-check <VPS>`.
- **GAP-SSH-EXEC-ARGC-001 rule 6 — exit 127 on step zero now aborts the batch.** That code on the *first* step is the fingerprint of a host name having been promoted to a command: the shell answers `command not found` and, before this, every remaining `--step` then ran against a machine nobody designated while the aggregate exit stayed 0. The abort is scoped to index zero; a 127 at any later index keeps the existing run-everything contract, because by then the session has already proven it is aimed where the caller believes. Defence in depth: the argv that produced this is now rejected at parse time, and this is the second lock on the same door.
- **BREAKING — GAP-SSH-EXEC-ARGC-001: the exec family could run on a host the caller never named.** `exec`, `sudo-exec` and `su-exec` declared `num_args = 1..=2` on a single positional vector and used the *count* to decide meaning: two positionals were `VPS COMMAND`, one was `COMMAND` against whatever host the on-disk `active` marker held. `--step` is `ArgAction::Append` and therefore never counted as a positional, so `exec <HOST> --step <CMD>` collapsed into the one-positional branch — the host name was executed as a remote binary on a *different* machine, step zero returned 127, and every remaining `--step` ran there while the aggregate process exit stayed 0. On 2026-08-09 that put a full auto-clean configuration, systemd units and timers included, onto the wrong server; the intended host was never touched and the operator was told it succeeded. The class is **CWE-441 (Unintended Proxy or Intermediary, "Confused Deputy")**: the CLI held authority the caller could not see it exercising, and spent it on a target drawn from ambient state instead of from the request. Naming the class matters because the fix generalizes from it — every surface that can act on a host must take that host from the argv, never from the environment. `clap`'s own `Arg::num_args` documentation warns about precisely this shape: a positional with a range keeps consuming values "until it finds another flag or option". Arity no longer carries meaning. The mode is declared by a flag: `<VPS> <COMMAND>` for one host, a selector plus one positional for a fleet, and the new **`--use-active <COMMAND>`** to reach the marker deliberately. A lone positional with no flag is now a usage error (exit 64) — it used to be the dangerous shortcut, and there is no argv left that reaches a host the caller did not designate. A second guard rejects `--use-active <registered-name>`, which is the same mistake one flag further along. The fleet path already failed closed; this brings the single-host path to parity. Migration: `ssh-cli exec 'uptime'` becomes `ssh-cli exec --use-active 'uptime'` or, preferably, `ssh-cli exec <VPS> 'uptime'`.
- **A1** `russh` moved from **0.62.2** to **0.62.5**. CVE-2026-68930 (GHSA-m65r-rprj-r5rg, published 2026-08-03) affects `<= 0.62.4`: channel-scoped handler callbacks are dispatched for recipient channel IDs that were never opened. The defect is **server-side** and this CLI is a pure client — there is not one reference to `russh::server` in `src/` — so the reachable impact here was nil. The pin moved anyway, because `cargo deny check advisories` reported **zero findings** with the advisory database synchronised the same day. A green supply-chain gate is not evidence that no published CVE applies.
- **A3** The at-rest crypto stack is no longer duplicated. `chacha20poly1305` 0.10 pulled `aead` 0.5 / `chacha20` 0.9 / `poly1305` 0.8 while russh and aws-lc-rs resolved the 0.6 / 0.10 / 0.9 generation, so **two independent ChaCha20-Poly1305 implementations** were compiled into a security binary. Bumping to `chacha20poly1305` 0.11, `getrandom` 0.3, `toml` 1 and `windows-sys` 0.61 collapsed 41 duplicate crate versions to 5, all transitive and individually justified in `deny.toml`.
- `deny.toml` `multiple-versions` moved from `warn` to **`deny`**. The warning was load-bearing and nobody read it; every surviving duplicate is now enumerated with a reason, so a *new* one — the kind that is actually fixable — fails the gate.
- **A6** `tls mtls import` read operator-supplied PEM paths with an unbounded `fs::read`. Both reads are now capped at `MAX_PEM_FILE_BYTES` (1 MiB) via the new `paths::read_bytes_capped`.

### Added
- **GAP-SSH-EXEC-ENVELOPE-002 completed — the error envelope names the host too.** The 0.5.5 work added the audit fields to the success envelope; the error path, where the information matters most, did not carry them. Measured: a failure reached through argv and one reached through the marker produced byte-identical envelopes, so the misdirection stayed invisible in the one channel an agent actually reads. `target_resolved`, `target_source` and `active_fallback` — plus their `host_*` read aliases — now appear on the error envelope as well, and are **absent rather than empty** when the failure predates target resolution — `vps_not_found` is that case. An empty string would assert that a host was chosen and has no name; absence says no host was chosen at all, and only one of those is true.
- **GAP-SSH-EXEC-ENVELOPE-002 — the single-host envelope now names its own target.** `ExecutionJson` carried `stdout`, `stderr`, `exit_code`, the truncation flags and `duration_ms`, and nothing that identified the host. `ExecHostJson` and `ExecBatchJson` — the fleet path — always carried `name`, so the asymmetry ran opposite to the risk: the one path that accepted a silent fallback was the one that could not be audited from stdout. Detecting a misdirected run required spending an extra SSH round-trip on a `hostname -f` badge command, which is optional and therefore skipped exactly when there is time pressure. The envelope now carries `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) and `active_fallback`, with `host_resolved` and `host_source` alongside as read aliases — on the success path always, and on the error path whenever the failure happened after the host was resolved — so a misdirected run is a string comparison rather than an investigation. All three are additive with serde defaults, so events written by earlier versions still deserialize. `health-check` reports `host_source` for the same reason: naming a host without naming its provenance cannot distinguish a deliberate check from an inherited one. That surface was defended here as a legitimate place to inherit the marker, because a probe is an idempotent read; the **BREAKING** entry above revokes that defence within this same release, and the field is what makes the new opt-in checkable rather than merely claimed.
- **GAP-SSH-DOCTOR-ACTIVE — `doctor` reports the active host, not just the path to it.** `local.active_file` gave the marker's *path* and no command in the CLI exposed its *contents*, so an agent had to read the file itself, outside the tool contract, to answer which host an ambient-target command would hit. `local.active_vps` now carries the name, or `null` when no marker is set.

### Changed
- **Usage lines stopped teaching an argv the parser rejects.** `clap` generated `Usage: ssh-cli exec --use-active <VPS> [COMMAND]...`, advertising a two-positional shape that `--use-active` forbids — printed at exactly the moment the caller had already got it wrong. The positional stays a vector, because two named slots would bind `exec --all uptime` to a `VPS` slot; the fix is an explicit `override_usage` on all three exec surfaces and on `health-check`, spelling the three legal forms. The `su-exec` opt-in doc-comment also stopped describing `sudo`, which it had been copied from.
- **Commit provenance no longer freezes inside a checkout.** `.commit_hash` is tracked in the repository, so it outranked git on every local build: the binary reported `f3b98b7` while `HEAD` was six commits later. Git now wins whenever a checkout exists, and the file keeps its stated purpose exactly — a crates.io tarball ships without `.git`, so the git probe finds nothing there and the file answers, as designed.
- **BREAKING — G-ERR-R01** New exit codes **69** (`EX_UNAVAILABLE`) and **70** (`EX_SOFTWARE`), with matching `SshCliError::Unavailable` / `Software` variants. `SshCliError::Config` — exit 65, classified permanent — had become the destination of every `map_err` without an obvious variant, 16 of them in `src/secrets.rs` alone. An OS keyring that was locked exited 65 with `retryable: false`, so the one failure in that group a plain retry actually fixes was the one agents were told to abandon. Keyring failures now exit **69** and are transient; CSPRNG failures exit **70**; `secrets.key` I/O exits **74**. `Config` in that module dropped from 16 call sites to 3, all of them genuine data errors.
- **B2 — error text is now localized in human mode; the JSON envelope stays English by contract.** 17 of 44 `i18n::Message` variants shipped with complete English and Brazilian Portuguese translations and had zero call sites, so `--lang pt-BR` produced byte-identical English for every failure. Six were shadowed by `thiserror`'s `#[error("…")]` attribute literals, which render at the real emission site; eleven had been superseded by per-mode variants and never deleted. Nothing catches this: `Message` is `pub`, so `dead_code` never fires on an unconstructed variant, and the exhaustive `match` in `en()` / `pt()` enforces a *translation*, never a *caller*. The superseded variants are gone, the error variants now carry the upstream detail, and `i18n::localized_error_text` maps them by `error_code` — the same stable discriminator the schema publishes. The `--json` branch deliberately keeps `SshCliError`'s English `Display`: agents branch on `error_code`, and a locale-dependent `message` would silently change the payload they parse when the host locale changes. Untranslated codes fall back to English, so the localization is fail-open.
- **B3 / A7 — component budget.** `#[allow(clippy::too_many_arguments)]` appeared 16 times in production code, concentrated in exactly the files A7 flagged as oversized: the one lint that measures that coupling was silenced where it mattered, which is why `-D warnings` stayed green over it. Removing all 16 showed only 10 still fired — six were dead attributes whose functions had already dropped below the threshold. `crate::vps::AuthOverrides`, `HealthCheckRequest`, `tunnel::ServeContext` and `output::TunnelClosedInput` replace the positional lists; 16 suppressions became 3, each with a comment justifying why the flat shape is correct. `src/vps/exec_ops.rs` (748 lines) split into `exec_ops/{types,single,elevation,fleet}.rs` and `src/sftp/mod.rs` (630) into `sftp/{setup,dispatch,emit}.rs`, with every public path preserved by re-export. `src/ssh/sftp_session.rs` stays whole by declared decision — it is one cohesive SFTP v3 state machine. Three new suites (`gaps_v062_cross_platform`, `gaps_v062_i18n_reachability`, `gaps_v062_component_budget`) turn all of this into measurements that fail loudly.
- **G-SCP-R01 / G-SCP-R02** `scp-transfer` gained `mtime_preserved` and `durable`. Both are additive and default to `true`, so events written by earlier versions still validate. The product documented mtime preservation as a guarantee while discarding the failure at two nesting levels; the parent-directory fsync after the atomic rename was invisible the same way. Failures are now logged and reported instead of being resolved by silence — transfers onto filesystems that cannot represent a timestamp (FAT32, exFAT, WSL interop paths) still succeed, and say so.
- **C3 — the component-size gate was calibrated to pass, not to a target.** The first cut of `gaps_v062_component_budget` used a single 830-line ceiling, chosen so `src/ssh/sftp_session.rs` (810) would fit, and its doc comment named that one file as "a deliberate exception". The constant granted ten: nine other files sat between 601 and 810 and passed without ever being declared, while the test's own failure message told the reader not to raise the ceiling. That is the B1 failure mode recurring inside B1's own remediation — a gate whose threshold is derived from the current measurement records the present and calls it conformance. The budget is now a hard **600** plus a `DECLARED_EXCEPTIONS` ratchet: every file above it is named with a frozen cap and a mandatory reason, an entry may never grow, and an entry whose file has fallen to budget fails the suite so the ledger can only shrink. The ratchet failed the very commit that introduced it, catching `src/i18n.rs` growing from 757 to 778 lines. Rather than edit the number, the file was split — `en()` and `pt()` became `src/i18n/en.rs` and `src/i18n/pt.rs`, dropping it from 778 to 577 and off the ledger, with exhaustiveness unchanged because each `match` must still cover every variant.
- **C3b — that split disarmed three path-pinned gates, which are now contract-pinned.** `gaps_v062_i18n_reachability` would have counted the relocated tables as production call sites, making every variant look reachable — green measuring nothing. `gaps_v040_integration::gap_scp_020_i18n_mensagens` went red looking for a translated string in the wrong file. `scripts/check_en_identifiers.sh` raised a false positive because its allowlist named only `src/i18n.rs`. This is the fourth occurrence of the class after `gaps_v057_sftp` and `gaps_v061_error_taxonomy`: a source assertion tied to a path passes vacuously when code moves in and fails spuriously when code moves out, because it tests layout rather than contract. The allowlist is now `src/i18n(\.rs|/)`, the integration test reads the concatenated subsystem, and the reachability suite gained `TRANSLATION_TABLES` plus a test that fails if an excluded path stops existing or the tables move back in. Both gates were re-verified by deliberate falsification: an unused probe variant turned reachability red, and a probe file with a Portuguese literal exited the identifier gate with 1.

### Fixed
- **The published crate did not compile, and no local command could tell you.** Renaming the target-resolution module to `src/cli/target/` collided with `.gitignore`'s bare `target/`. A gitignore pattern without a leading slash matches at every depth, so git ignored a source directory, and `cargo package` — which lists files through git inside a checkout — dropped both of its files from the tarball. `cargo build`, `cargo clippy`, 902 tests and all eleven gate scripts stayed green, because every one of them reads the disk. Only `cargo publish --dry-run` failed, with `E0583: file not found for module `target``. The ignore rule is now anchored to `/target/`, which is what it always meant — this crate is a single member and its build output lives at the root — and the module was renamed again, to `targeting`, so correctness no longer depends on one line of `.gitignore` staying right.
- **The localized `no_active_vps` text taught an argv the surface rejects.** `health-check --use-active` reaches `SshCliError::NoActiveVps` too, and the message answered `run \`connect <NAME>\` or pass \`<VPS> <COMMAND>\`` — but `health-check` takes no command, so a caller following the instruction got a second usage error. The wording is now surface-agnostic in both languages. This is the localization arriving after the surface changed: the `--use-active` opt-in gave an old message a second caller, and nothing pairs a message with the set of surfaces that can emit it. Two doc comments went stale the same way and were corrected — `HealthCheckJson::host_source` and `HealthOptions::host_source` still explained *why* a nameless probe may inherit the marker, which the same release had made a usage error. Justifications age worse than descriptions, because the behaviour they defend can be removed without the paragraph noticing.
- **C1 — the English-message contract never reached the machine-readable artifact.** B2 established that the `--json` envelope's `message` stays English regardless of `--lang`, and documented it in `docs/AGENTS.md` and `docs/schemas/README.md` — but not in `docs/schemas/error-envelope.schema.json`, whose `message` description was still the generic "human-readable error text". The one artifact an agent would generate a client from was the one that did not state the contract. The description now declares the invariant, directs consumers to branch on `error_code`, and scopes localization to human text mode.
- **C2 — half the error emitter was never localized.** B2 routed every typed `SshCliError` through i18n and left the last branch of `resolve_exit_code` untouched: an `anyhow` chain that downcasts to neither `SshCliError` nor `DomainError` printed raw English under `--lang pt-BR`. The failure a user is least equipped to interpret was the only one never translated, and it looked covered because the typed path — the tested one — was correct. `localized_error_text` takes `&SshCliError` by signature, so that branch had no way to call it. `Message::ErrorUnexpected` and `i18n::localized_unexpected_text` close it; the helper returns `String` rather than `Option` because there is no alternative rendering to fail open to. Only the label is translated — the `anyhow` chain is preserved verbatim so no diagnostic is lost — and the JSON envelope keeps `error_code` `"unexpected"`.
- **B1 — the Windows target did not compile.** `cargo check --target x86_64-pc-windows-msvc --no-default-features` failed with six errors while `fmt`, `clippy`, 818 tests, `deny` and `doc` were all green, and `docs/CROSS_PLATFORM.md` advertised Windows as supported. Five errors came from `#![forbid(unsafe_code)]` on `src/platform/mod.rs`: an inner attribute on a module file also governs its children, the `windows` child is the product's only Win32 FFI surface, and `forbid` — unlike `deny` — cannot be lifted by an inner `#[allow]`. The sixth was `windows-sys` 0.61 redefining `HANDLE` from an integer to `*mut c_void`, which broke a `handle == 0` guard. The module now uses `deny`, `windows.rs` carries a file-scoped `allow` referencing the audited G-UNSAFE allowlist, and the guard is `handle.is_null()`. The structural cause was that no gate cross-compiles: `#[cfg(target_os = ...)]` code for a foreign target is discarded before type-check, so it is invisible to every gate run on the host. `scripts/check_cross_targets.sh` now type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`, and is required by `CONTRIBUTING.md` and the release checklist.
- **B4** `SCP_IO_CHUNK = 32_768` was declared twice as a function-local `const` inside `src/ssh/client_real_scp.rs`, and `SCP_HEADER_MAX_BYTES` once in `src/ssh/scp_wire.rs`, each carrying a marker admitting it belonged in `crate::constants`. The stated reason was an edit allowlist from a round that had already closed, so the justification expired while the duplication stayed — and `src/constants.rs` already hosted `SFTP_IO_CHUNK` with the identical value. Both constants moved, and a gate fails if the marker reappears.
- **B5** `gaps.md` declared "OPEN residual: 0" in its header while eighteen sections still carried `**OPEN**` titles in the body. All eighteen were verified closed in code across two audits; only the titles had never been updated. Traceability is the single thing that file delivers, so the divergence is a product defect rather than a formatting one.
- **A2** `docs/schemas/error-envelope.schema.json` enumerated `error_class` as `transient|permanent|cancelled` while the CLI has emitted `partial` since 0.5.4 (G-ERR-R02). A strict agent validator rejected the exact envelope the product produces for a partial fan-out. `error_code`, emitted on every failure, was likewise never contracted; both are now declared.
- **A4** `sftp ls` and `sftp stat` wrote to stdout with `println!`, bypassing the `output` facade that `src/lib.rs` documents as mandatory — so `--quiet` was ignored and piping into `head` aborted on EPIPE instead of exiting 141. The `mkdir` / `rmdir` / `rm` / `rename` success lines were built with an inline English `format!`, so `--lang pt-BR` produced English for the most-read SFTP output. Both now route through `output` and `i18n`.
- **A6** `cargo build --no-default-features` — the diagnostic configuration documented in `Cargo.toml` — failed with **62 errors**. The SFTP subsystem was never gated behind `ssh-real` despite calling `russh_sftp` directly. The module, its emitters and the dispatch arm are now gated, and the subcommand answers with a typed refusal instead of failing to link.
- **G-QA-R02** `run_tunnel`'s resolution range (registry lookup → credential overrides → `ConnectionConfig`) is extracted into `resolve_tunnel_connection`, which takes an already-loaded record and touches no disk. This is the range where the E3 agent-auth bug lived, and it had no offline coverage at all.
- **D5 — two published schemas were unreachable through the CLI.** `docs/schemas/` carried 22 documents while the embedded `SCHEMAS` catalogue in `src/cli/schema_cmd.rs` listed 20, so `ssh-cli schema dry-run` and `ssh-cli schema tunnel-closed` both exited **64** with `unknown schema` — while `docs/schemas/README.md` tells agents to discover schemas by running `ssh-cli schema` and documents those exact two names. The contract was written where it was comfortable to write it and not where the consumer reads it. Both entries are in the catalogue now and both commands exit **0** with the document. The new `catalog_and_disk_agree` compares the two sets in both directions and validates that each name is the file leaf minus its suffix, so a schema published on disk without a catalogue entry — or a catalogue entry with no file — fails the suite instead of surfacing as `unknown schema` to an agent.
- **D10 — the only discriminator of a successful tunnel shutdown was discarded.** `src/tunnel.rs` emitted the closing event as `let _ = output::print_tunnel_closed_json(…)`. The three endings that share exit **0** — `deadline`, `signal` and `accept_error` — are distinguishable *only* by that event, so a failed emission handed an agent a successful exit with no way to learn which of the three had happened. The failure is now reported through `tracing::warn!`, the same pattern R10 applied to the two discarded `Result`s in `copy_bidirectional`. The exit code is deliberately unchanged — the shutdown really is a success — but the emission failure is visible on stderr, where the agent-native contract puts diagnostics.
- **K4 — commit provenance silently degraded to `unknown` in the published crate.** An external hygiene tool ran `git rm --cached .commit_hash`, and `cargo package --list` went from 245 entries to 244 with the file gone. Inside a git checkout Cargo builds the package file list from the index, so an untracked file is dropped from the tarball no matter that it still sits on disk. `.commit_hash` is step 2 of the `build.rs` precedence chain (`SSH_CLI_COMMIT_HASH` → `.commit_hash` → `git` → `unknown`) and the only provenance source that survives into a crates.io source build, where no checkout exists — so its removal made every published build report `unknown` without any gate objecting. The existing `gap_rel_007_build_rs_precedence` asserted the file exists on disk, which is true and insufficient, so it stayed green throughout. It now also requires the file to be present in the git index, skipping that check when `.git` is absent — which is precisely the crates.io build where the file is the surviving evidence rather than something to re-verify. The first attempt asserted that the last matching `.gitignore` rule must be a negation; that was measured wrong and reverted, because re-adding the file with `git add -f` while the ignore rule still won by last-match restored the packaged 245, proving tracking is load-bearing and the ignore rule is not.

### Internal
- **`src/cli/exec_target/` is now `src/cli/targeting/`.** It arrived serving the three exec surfaces and was named after them; it now also resolves `health-check`. A module named narrower than its job invites the next caller to copy it rather than extend it, which is how the duplication it was extracted to remove comes back. The module is private and every item is `pub(crate)`, so nothing outside the crate can notice. The functions keep their `_exec_target` suffixes: they name an operation, not a location. The first attempt at this rename used the shorter `target` and produced an unpublishable crate — see the packaging entry under Fixed.
- **`tests/gaps_v067_packaging_reach.rs` asserts that every source file survives `cargo package`.** The gate has two arms: a literal parse of `.gitignore` that never skips, and a `git check-ignore` probe that is exact because it is the same matcher Cargo consults, and that announces its skip out loud when git is absent. A denylist of names like `target` was rejected: once the ignore rule is anchored the name is harmless, so the denylist would be superstition guarding the last defect rather than the class.
- **The Portuguese 0.5.4 `### Corrigido` heading read `### Segurança`.** Its four bullets are identical to the English ones under `### Fixed`, so the English twin proves the intent. A Portuguese reader scanning 0.5.4 for fixes found none, and found fixes filed as security. Correcting a wrong label restores the record; it is not rewriting history.
- **The changelogs are now gated on structure, not only on tokens.** Release 0.5.5 carried `### Security` three times, `### Added` twice and `### Changed` twice, and English had nine section headings against Portuguese's eight. The missing one mattered: four security entries, including a `russh` CVE, sat under `### Adicionado`, so a Portuguese reader scanning for security work would not have found them. Every existing gate compared *tokens*, and every word was present, so all of them stayed green — placement was the property that failed. `the_newest_release_has_one_section_per_name_in_both_languages` now asserts allowed names, uniqueness, canonical order and EN↔pt-BR parity through an explicit pair table, scoped to the newest release so shipped history is not rewritten to satisfy a rule added after it.
- **Source comments are gated on being English, and the gate now exempts citations instead of files.** Forty-five comment lines across nineteen files were Portuguese prose, the English changelog carried the word `desvio` in four places, and about thirty test functions had Portuguese identifiers. Nothing caught any of it because nothing looked. The first version, `source_code_is_written_in_english`, scanned for Latin diacritics against a per-file allowlist; it grew to thirteen entries, each honestly earned because the accents really were the payload. The cost was structural: once `tests/docs_conformance.rs` is exempt, all twelve hundred of its lines are exempt. `tests/gaps_v068_source_language.rs` replaces it and strips string, char and backtick spans per line, so a comment may *quote* localized data and may not *be* Portuguese — which keeps those same thirteen files under the rule. The allowlist is now **empty**, and that is a measurement: every accent in all thirteen, `src/i18n/pt.rs` included, already sits inside a literal. Choosing the right unit to exempt removed the need to exempt anything.
- **The language scanner had to learn that `\` escapes nothing in a raw string.** Its first draft read `r"\\?\"` in `src/paths.rs` as an escaped quote, swallowed the terminator and desynchronised the rest of the file. The symptom was camouflage rather than silence: three `assert!` lines of legitimate Portuguese *test data* were reported while a genuinely bare `// é precomposed` eleven lines below was swallowed and never reported. A scanner that mis-parses accuses the innocent and shelters the guilty in the same run, so raw strings are now parsed as raw strings and both literal kinds carry across lines.
- **Two Portuguese test names survived every gate, and the extension names why.** `gap_sec_001b_todo_sidecar_pontuado_esta_barrado_do_pacote` and `gap_sec_002_e2e_recusa_grok_config_dentro_do_repo` carried no listed connective and no accent, so neither the shell gate nor the diacritic gate could see them; they were found by reading. `check_en_identifiers.sh` gains `_da_`, `_dos_`, `_das_`, `_dentro_` and `_esta_`. It deliberately does **not** gain `_do_` or `_no_`: measured on this tree, `_do_` appears four times with two of them real English (`rejections_do_not`) and `_no_` thirty-nine times, essentially all English. A signal that fires on correct code gets the whole gate switched off, so the homographs are excluded and the exclusion is written down where the next author will reach for them.
- **Seventeen Portuguese `assert!` messages in `tests/gaps_v039_integration.rs` are now English.** They sat inside string literals, which is exactly where the citation rule declines to look, because that is where test *data* lives. An assertion message is prose wearing data's clothes; the rule cannot separate them mechanically, so this class stays a reading task and the limit is stated here rather than implied.
- **The `install-resolve` gate went red because of the host, not the code.** It is the only gate that compiles the whole dependency tree from scratch, and it did so inside `mktemp -d` — which follows `TMPDIR` onto a `tmpfs` under a per-user quota. Run after the `test` gate had already filled that `tmpfs`, it died with `Disk quota exceeded (os error 122)` building `aws-lc-sys`. A gate that cries wolf gets skipped, and a skipped gate protects nothing. Only `CARGO_TARGET_DIR` moved, to a nested directory under `target/`: what the gate proves is that the graph re-resolves without the repository lockfile, and that proof lives entirely in the `Cargo.lock` generated in the scratch dir, so where the objects land is irrelevant to it. Reusing artefacts between runs cut the gate from 53s to 34s as a side effect.
- **The component-budget ratchet had no opinion on a cap that outran its file.** `DECLARED_EXCEPTIONS` refuses growth and refuses rows whose file has fallen to budget, but a cap left behind by a *shrinking* file kept authorizing the difference: three entries held 13, 25 and 31 lines of credit nobody had argued for — room for a whole new function with the gate still green. That is the failure this ledger's own doc comment names, arriving from the other direction, and two of the three were opened by this release's own extractions. The caps are now at the measured sizes and `assert_ledger_is_current` fails above `MAX_LEDGER_SLACK` (8 lines). Equality was rejected deliberately: it would turn deleting one line red, making the cheapest way to stay green *not tidying up*, and a gate that punishes cleanup trains the opposite of what it wants.
- **The component-budget ledger paid off an entry.** `json_wire/vps_export.rs` was frozen at 606 lines and now measures 586, inside `HARD_BUDGET`, so `the_exception_ledger_contains_no_stale_entries` failed until the row was deleted. That is the ratchet working in the direction it was designed for: it records repayment as loudly as it records debt, and raising the cap to keep the row alive is the move it exists to block. The ledger is down to six entries.
- **Agent-facing prose taught the alias instead of the canonical field.** `docs/schemas/exec.schema.json` says to bind new consumers to `target_resolved` and `target_source`, with `host_resolved` and `host_source` as 0.5.5 read aliases written from the same value. Both `SKILL.md` files and both `AGENTS.md` files named only the aliases, so an agent following the skill bound to the compatibility spelling the schema steers away from. All four now teach the canonical pair first and name the aliases as what they are.
- **The gate battery is eleven gates, and five documents said ten.** `check_all_gates.sh` grew `exec-target` without any of its readers noticing. Both `RELEASE_CHECKLIST`s, both `CONTRIBUTING`s and both `TESTING`s are corrected, and the checklist now tells the reader to count with `--list` rather than trust the sentence — a number in prose is a fact with no gate behind it.
- **`docs/MIGRATION.md` had no 0.5.5 section.** The release with two breaking changes to how every verb finds its host documented neither of them where an upgrader looks. Both languages now carry the three legal forms, the 64/66 split and the provenance trio.
- **`docs/schemas/README.md` listed `telemetry`, `active_vps` and the secrets fields flat when the schema nests them under `local`.** An agent binding to `.telemetry` reads `null`. The Portuguese half was further behind still, describing the error envelope as three fields when it has twelve. Both halves now match the schema, including the provenance trio and its absent-never-empty rule.
- **`SECURITY.md` documented the 0.5.4 fixes and stopped there, while 0.5.5 is itself a security release.** The policy that readers consult first said nothing about GAP-SSH-EXEC-ARGC-001, the highest-severity defect of this line — a command that rewrote a machine nobody named. Both halves gained a `Security Fixes in 0.5.5` section, the threat model gained a fifth adversary (ambiguous argv from a *legitimate* operator, which needs no attacker at all), and the STRIDE row for argv stopped writing `—` under Repudiation: before the provenance trio existed, a run genuinely could not be tied to a host, which is what Repudiation names.
- **The 0.5.5 release blockquote taught the alias in twenty-two places.** It announced that the envelope "gained `host_resolved`, `host_source` and `active_fallback`" while `docs/schemas/exec.schema.json` says to bind new consumers to `target_resolved` and `target_source` and calls the other pair a 0.5.5 read alias. The same sentence is replicated across twelve English and ten Portuguese documents, so a single wrong noun propagated to every entry point at once. `docs/COOKBOOK.md` was worse than wrong prose: its copy-paste recipe *parsed* `.host_source`, so every reader who followed it bound new automation to the compatibility field.
- **Four table rows rendered with phantom columns on crates.io.** A pipe inside a cell ends that cell even inside a code span, so ``| `scp upload|download` |`` is a three-column row in a two-column table; the `secrets` row carried two such pipes. Neighbouring rows escaped theirs correctly, so this was drift between rows rather than a convention. `tests/gaps_v069_markdown_tables.rs` now counts unescaped pipes per row against the delimiter row that opens each table, across all twenty-five rendered documents — structural rather than lexical, so it also catches a cell lost to a stray pipe in prose.
- **Both changelogs claimed Keep a Changelog 1.1.0 and opened without `[Unreleased]`, and a gate enforced the omission.** The format requires that section; `top_release_sections` in `tests/docs_conformance.rs` took the first `## [` heading unconditionally, so adding it would have made the parser return empty and fire its own "the parser is broken, not the changelog" assertion. The gate forbade conformance with the format the changelog header claims to follow. The parser now skips `[Unreleased]` explicitly, `the_release_parser_skips_the_unreleased_section` pins that behaviour, and both files open with the section. The footer was stale in the same way: `[Unreleased]` compared against `v0.5.3` two releases late, and neither `[0.5.5]:` nor `[0.5.4]:` existed.
- **`README.md` forbade in one section exactly what it did in another.** The `Changelog` section said "Do not paste release notes into this README" while line 8 pasted a multi-release historical note — and `gap_doc_004_product_line_040_e_file_only` *requires* that note, so the two rules could not both be obeyed as written. The rule is now stated as the project actually practises it: the banner and the one-line product-line note are the history this README carries, everything per-release lives in the changelog.
- **The three `llms.*` files opened with a release note where the format puts the project description.** A consumer of the `llms.txt` convention reads the first blockquote after the H1 as what the project *is*; it got the 0.5.5 security note instead. The taglines are now the first blockquote in all three, with the release banners below them.
- **`docs/TESTING.md` catalogued 24 of the 40 test suites that existed then.** The document exists to tell a contributor what each suite is for, and it stopped at `gaps_v064` — omitting `docs_conformance.rs` itself, `test_quality.rs`, the whole `gaps_v061`–`v063` band, and every suite this release added, including `gaps_v065_exec_target_designation.rs`, which guards the defect the release is named after. A catalogue that covers 60% is worse than none: the reader stops looking after consulting it and concludes the coverage does not exist. Both halves now list all forty-one, and `tests/gaps_v070_doc_reference_reach.rs` now fails the build if that ever drifts again — in either direction.
- **Both READMEs named all 47 commands and none of them were findable.** The `Commands` table folded them — `sftp upload|download|ls|…` in one cell — which reads fine and defeats retrieval: `rg -F 'sftp ls' README.md` returned nothing while the command sat right there. Unfolding into separate spans fixed the phantom columns at the same time, and `README.md` / `README.pt-BR.md` joined `FULL_INVENTORY_DOCS` so the folding cannot come back. Added the missing `Target designation (0.5.5)` row, plus rows for `--hosts` / `--tags` and `--use-active`: the breaking change of this release had no line in the table that describes the product surface.
- `.atomwrite/` is now barred from the published crate in `.gitignore`, in the manifest `exclude` and in `.cargoignore`. The pre-publish package gate measured `cargo package --list` shipping `.atomwrite/scratch/old.txt` and `.atomwrite/scratch/new.txt`, staging left behind by an earlier editing round. Being untracked by git was no protection: Cargo packages any file that is neither tracked nor ignored. The `.gitignore` already covered `.serena/`, `.claude/`, `.setting.cyber/` and `.cursor/`, and missed the one sidecar the agent's own tool creates.
- Added `gap_sec_001b_every_dotted_sidecar_is_barred_from_the_package`, which discovers dotted directories at the repo root instead of enumerating them. Its sibling `gap_sec_001` pins one sidecar by name, and naming them one at a time is precisely how `.atomwrite/` slipped past four listed neighbours. Whatever appears at the root must now be either deliberately packaged, which is `.git` and `.cargo`, or barred on all three surfaces, so a new sidecar turns the suite red on the day it appears. Proven negatively: removing the `.gitignore` line fails the gate naming that exact file.
- Corrected a false claim about `vps export` in both skills and both skill eval sets. They said the export body stays TOML unless `--json` is passed. It does not: the body follows the resolved output format, and that resolves to JSON whenever stdout is not a TTY, which is every agent invocation. Measured, `vps export -o /tmp/hosts.toml` writes a JSON envelope into a file named `.toml`; a TOML body needs `--output-format text`. `export_import_toml_roundtrip` gave this the appearance of coverage while asserting only that `sshcli-enc:` is absent and that import accepts the file — and since import accepts TOML *and* JSON envelopes, the roundtrip closes with the wrong body. The same claim was then corrected across the long-form surfaces and is now held by a gate.
- The false `vps export` TOML claim was corrected in the twenty long-form surfaces that still carried it: `README` and `llms`/`llms-full` in both languages, `INTEGRATIONS`, `docs/AGENTS`, `docs/COOKBOOK`, `docs/MIGRATION`, `docs/HOW_TO_USE`, `docs/TESTING`, `docs/RELEASE_CHECKLIST` and `docs/schemas/README`. Several stated the inverse of the measured behaviour outright, telling the reader that auto JSON on non-TTY does *not* apply to export. The `docs/COOKBOOK` and `docs/HOW_TO_USE` examples now show both paths, because a copyable command that writes JSON into a `.toml` file teaches the wrong lesson twice.
- New `no_document_claims_export_defaults_to_toml` in `tests/docs_conformance.rs`, phrase-based over the skills, the full-inventory documents and the fifteen surfaces that describe the export body. It caught a line in `docs/HOW_TO_USE.md` that a hand-written regex sweep had missed on its first run. The CHANGELOG history is exempt on purpose: those entries record what a past release claimed, and rewriting them would erase the evidence that the claim was made.
- `export_import_toml_roundtrip` was renamed `export_body_follows_resolved_format_and_both_import` and now asserts what its old name promised. It pins the JSON envelope from the default non-TTY path, pins a TOML body from `--output-format text`, pins that TOML says `username` where the envelope says `user`, and imports both. The old version asserted only that `sshcli-enc:` was absent and that import accepted the file, and since import accepts TOML *and* JSON envelopes it closed on the wrong body while looking green. `tests/gaps_v051_integration.rs` held `export_pipe_defaults_to_json_when_non_tty` the whole time: the suite proved the truth in one test and advertised the falsehood in the name of its neighbour. A test whose name asserts what it never verifies is worse than an absent test, because the gap is invisible exactly where someone would look for it.
- Both skills were rebuilt against the installed binary and cut to the 4000-word product budget, from 4906 and 5008 words. Nothing was dropped from the contract: all 47 commands, every flag and every wire token survive. The words came out of duplicated prose — `FORBIDDEN` bullets that merely mirrored a `REQUIRED` bullet in the same section, and an `Absolute Prohibitions` section that restated in full the prohibitions each section had already stated.
- New `skills_stay_within_the_word_budget` in `tests/docs_conformance.rs`. The budget was a standing written rule that nothing counted, so both files ran a fifth over it with 13 gates green. A rule with a number and no gate is indistinguishable from no rule.
- New `skills_name_every_command`: `LEAF_COMMANDS` is now asserted over both skills. `FULL_INVENTORY_DOCS` deliberately lists the seven long-form documents and never included the skills, even though each skill opens its catalog claiming to be the whole surface — true when measured, held true by nothing.
- Both skills now separate global flags from subcommand-local ones. `--i-accept-network-exposure` was documented as global and is local to `tunnel`; placed before the subcommand it exits 2 at parse time. The new section names the four that read like globals and are not.
- Both skills now document the two meanings of `--disable-sudo`. Before the subcommand it suppresses elevation for one invocation and touches no disk; on `vps edit` it writes `disable_sudo` into the config and disables elevation on that host permanently. `--enable-sudo`, the only way to undo the persistent form, appeared in neither skill, so an agent could enter a state the skill did not teach it to leave. `--max-chars` was likewise unnamed and is now marked a legacy alias of the command cap.
- Removed version narrative from both skills, which product rules forbid there: two `0.5.4+` headings, a `since 0.5.4/A3` clause, the gap identifiers `G-SCP-R01/R02` and `G-ERR-R01`, and a sentence describing how an earlier release behaved.
- Published documentation now covers the 0.5.4 surface it had only announced. `tunnel --reverse`, `--socks5` and `--remote-socket` shipped in this release and were mentioned in zero of `docs/HOW_TO_USE`, `docs/COOKBOOK`, `INTEGRATIONS`, `docs/MIGRATION`, `docs/CROSS_PLATFORM`, `SECURITY` and `llms-full.txt`, in either language: the three flags existed only in the release banner and the skill package. `HOW_TO_USE` gained a Tunnel modes section with a runnable example per mode, `COOKBOOK` gained four recipes, `MIGRATION` gained a `Since 0.5.4` section covering both BREAKING changes, `SECURITY` gained a fixes section for A1/A2/A3 plus the exposure guard, and `CROSS_PLATFORM` gained the portability rule that the client may be Windows while the socket may not.
- New `the_054_surface_reaches_every_user_facing_document` in `tests/docs_conformance.rs`. That suite already asserted these tokens, but only over the two skills and the two changelogs, so it stayed green through all of the above — a gate aimed at the wrong subset is indistinguishable from an absent one on the scoreboard. The contract is per document rather than one blanket list: the cookbook owes a recipe, the migration guide owes an upgrade note, the security policy owes the acknowledgement.
- New `every_command_appears_in_every_full_inventory_document`: all 47 leaf commands must appear as literal full paths in every file claiming a complete inventory. `llms-full.txt` and the `docs/AGENTS` tables used brace notation (`tls acme {account create,…}`), so a retriever searching `tls acme account create` found nothing and would conclude the command does not exist. Compact notation suits a short index a human skims; it does not suit a file whose stated purpose is machine ingestion.
- New `every_schema_is_indexed_in_the_full_llm_map`: 22 schemas exist on disk and `llms-full.txt` indexed 14, omitting `dry-run` and every `sftp-*` and `*-batch` contract — exactly the envelopes needed to parse fleet and SFTP work. `docs/schemas/README.md` had a directory-walking gate; the discovery map did not, so it drifted behind the disk.
- Corrected a cookbook example that the CLI would reject today: `--bind 0.0.0.0` was shown without `--i-accept-network-exposure`, which G-TUN-R13 made mandatory. No gate executes cookbook examples, so the recipe aged into invalid without a signal.
- Twelve stale product-line declarations moved from 0.5.3 to 0.5.4. The drift was symmetric across both languages, so no parity gate could see it; roughly sixty remaining `0.5.3` mentions are legitimate version floors (`prefer 0.5.3+` for SFTP) or historical migration sections and were classified before editing rather than replaced blindly.
- The SFTP permission mask is now documented as what it is — directional. `SFTP_PERM_MASK` (`0o7777`) applies outbound on upload and deliberately keeps setuid/setgid/sticky on a file the caller owns; `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) applies inbound on download, where the mode comes from the server. Twelve lines across ten files named the first as if it were the only mask, and `docs/MIGRATION` announced A3 fifteen lines above contradicting it — a reader concludes the fix does not exist. New `permission_mask_claims_are_directional` walks `docs/` and rejects any file citing `0o7777` without the inbound constant, so the class cannot recur rather than just this instance.
- That mask gate initially walked `docs/` alone — the directory being edited — and went green while the identical claim stood uncorrected in `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt` and both `SKILL.md` files. A gate written to catch wrong-subset targeting was itself scoped to the author's diff rather than to the claim. Widening it to a hand-written list of four directories then missed `docs/schemas/README.md`, because `read_dir` does not recurse — a hand-written list of places to look is itself something that falls behind the disk. The walk is now recursive from the root over `.md` and `.txt`, skipping `target/` and `.git`, so a document added in a new subdirectory is covered without anyone remembering to register it; six more files were corrected.
- `--tags` is documented. The selector is real on `exec`, `sudo-exec` and `su-exec`, and it had zero occurrences across all fifteen files under `docs/` while `--all` and `--hosts` were documented everywhere — an agent reading those files concludes tag fan-out does not exist and opens one process per host. New `fleet_documents_name_every_selector` asserts every selector clap accepts appears where the fleet is described; it also caught `--hosts` missing from both `docs/HOW_TO_USE` files, which no hand-written search had looked for.
- Corrected the description of A1 in six places. The 512-character banner log cap predates this release (G-SSH-14); what A1 fixed is that the cut was applied by byte index, which panics inside a multi-byte character, and `panic = "abort"` turns that into process death with no unwind — a remote peer could kill a multi-host fan-out with one non-ASCII character. `SECURITY.md` titled the issue "unbounded server-sent pre-auth banner" and claimed the fix bounds what the peer can make the client hold; russh materialises the whole banner before the callback, so the residual limit is now stated explicitly: A1 bounds the panic, not the allocation.
- Corrected the claim that `tunnel --bind` is IP-validated by clap in all cases. That holds for the local bind. Under `--reverse` the exposed end is the positional `<remote_host>`, guarded by `guard_remote_exposure`, which compares text instead of parsing an IP because RFC 4254 assigns meaning to names and to the empty string — a typo there is exit 64 from the guard, not exit 2 from clap. Four lines said otherwise.
- Documented that `--bind` is accepted and then silently discarded under `--reverse`: reverse delivery is forced to loopback and `ReverseServe` never receives the address. The docs correctly said the acknowledgement guards the server's bind, but none warned that the flag itself does nothing there, so `--reverse --bind 192.168.1.10` changed nothing and warned about nothing.
- `docs/RELEASE_CHECKLIST` gained item 26, the 0.5.4 honesty block. Measured before the fix: both checklists and both `docs/TESTING` files mentioned `tunnel_closed`, `--select` and `--count-only` on line 3 only — the release banner — and named none of the three tunnel modes anywhere, with zero mentions of `--dry-run` or `--tags`. A checklist exists to be a contract, and this one announced the release in its header while verifying nothing from it in its body. The four files are now under `SURFACE_054`, and `gaps_v060_tunnel_modes.rs` is named in the suite inventory.
- One terminology line corrected rather than nine. `docs/MIGRATION` line 63 used master-key as the product term in both languages; the other eight mentions across `docs/` explicitly describe it as the legacy keyring alias accepted on read, which is true, so a blind replacement would have destroyed eight correct statements.
- `TransferResult` carries the two durability flags; `print_transfer_json` takes it by reference rather than eight positional arguments.
- `build_tunnel_closed` / `build_tunnel_listening` split payload construction from emission. `tunnel_closed`, `forwards_served` and `capacity_waits` previously appeared outside the emitter in exactly one place: a test asserting the CHANGELOG mentions them. Deleting the emission would not have failed the suite — prose validating prose, one step removed from the tautology G-QA-R01 was written to stop.
- New `tests/gaps_v061_error_taxonomy.rs` and `tests/gaps_v061_scp_durability.rs`; new tunnel tests covering agent-auth forwarding, the deliberately non-overridden registry timeout, every `close_reason` branch, the `bind` field, and `forwards_served` driven through a real loopback accept with an injected client.
- `gap_deny_002_deny_toml_sem_ignore_cve` pinned `multiple-versions = "warn"` literally, so tightening the policy broke it while loosening it to `allow` would have passed. It now rejects `allow` and accepts anything stricter.
- `known_hosts` persistence pre-sizes its buffer from the entry count instead of growing from empty on every TOFU write.
- Two Portuguese doc-comments in `src/ssh/client_real_scp.rs` and `src/retry.rs` translated to English, per the English-source rule.
- **`scripts/check_all_gates.sh`** runs the whole mandatory battery — `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers`, `install-resolve` — in one invocation, with a TSV or NDJSON record per gate on stdout, per-gate logs on stderr and a non-zero exit if any gate is red. It exists because `cargo clippy` and `cargo test` both abort on the first unbuildable target, so a single broken test file hid the state of every gate behind it — which is exactly how the local inventory came to declare 835 green with four gates red and one target that did not compile. It caught a rustfmt violation on its first run. Not CI: no workflow, no runner, no network. The battery is sequential by design, because the cargo gates contend for one `target/` lock. `scripts/check_advisory_freshness.sh` had no other caller, so it was only reachable through a script nothing documented; the runner and that gate are now declared in `CONTRIBUTING`, `docs/TESTING` and the release checklist in both languages.
- **`tests/gaps_v064_gate_runner.rs`** turns the runner's coverage into a contract. Every `scripts/check_*.sh` must be a gate, and every `scripts/*.sh` must be either a gate or named — with a reason — in a `Declared non-gates` block in the runner's header. Coverage was previously complete only by coincidence of naming: a future `scripts/check_foo.sh` would have slipped in unnoticed, and a battery that omits a script in silence reads as total coverage. The suite also pins `--locked` on the test gate and asserts the runner is documented in both language versions of all three maintainer documents. Directory listings are sorted before assertion, because `std::fs::read_dir` documents its order as platform-dependent and free to change between calls.
- `tests/docs_conformance.rs` now asserts the `CONTRIBUTING`, `docs/TESTING` and `docs/RELEASE_CHECKLIST` tokens over **both** language files. Checking only English is how `CONTRIBUTING.pt-BR.md` came to omit `gaps_v040`, the explicit suite list and the entire cross-target gate section while the suite stayed green — the reader who does not read English was told strictly less, and the parity gate could not see it. The pt-BR file also carried an elided suite range (`v038 … v051`), the exact pattern its English counterpart warns against because a `contains` check silently drops the suites in the middle.
- **D1** `tests/gaps_v040_integration.rs` called `tunnel_subsystem()` and `output_subsystem()`, neither of which existed, so the target did not compile and `clippy` and `test` aborted before a single test ran. Both are implemented as thin wrappers over a shared `concat_subsystem` helper, alongside the `i18n_subsystem()` the original author had written. Test code is excluded from the concatenation on purpose: including it would let an assertion pass by matching the text of the test itself.
- **D4** `tests/gaps_v063_secret_stdin.rs` required exit **64** from a clap parse error. Confirmed against clap 4.6.6, whose `error::Error::exit` documents "exits with a status of `2`": the sysexits codes in `src/errors.rs` apply only to product errors raised *after* the parse succeeds. The companion `assert_ne!` was passing vacuously, because 64 is never what the parser emits; it now excludes 2, which is the code the supported form genuinely cannot produce.
- **D7** `tunnel::tests::accepting_a_connection_increments_forwards_served` passed alone and failed in the suite at a rate tracking `--test-threads`. `serial_test::serial` serializes only the tests that carry it, so a non-serial reader of the process-global stop flag still raced marked writers. Fixed with both halves — `#[serial_test::serial]` plus `crate::signals::reset_flags_for_tests()` — and verified by repeated runs at 8, 32 and 72 threads rather than by one green execution, since a single pass cannot distinguish a fix from a race.
- The component ratchet then bit the fix itself: `src/tunnel.rs` grew past the hard 600-line budget, and `DECLARED_EXCEPTIONS` entries may only shrink. `TunnelStats` and its `close_reason` unit test moved to a new `src/tunnel/stats.rs`, dropping `tunnel.rs` to 570 and removing its ledger entry entirely. Raising the cap was the dishonesty the ratchet exists to block.
- **D3** `scripts/e2e_real_ssh.sh` replaced a non-executable `--bin` with the default and continued, printing `PASS E01` through `PASS E16` for a binary the operator never named — in a harness whose own usage text promises that an unusable environment is a failure and never a silent skip. A `BIN_EXPLICIT` intent flag, set by `--bin` and by the harness-only `SSH_CLI_E2E_BIN`, now makes that case exit **2** with `FAIL E00`; the auto-build survives only when nothing was named. Verified negatively.
- **D9** `tests/gaps_v062_i18n_reachability.rs` searched `src/i18n.rs` for `fn en(msg: &Message)`, which C3 had moved to `src/i18n/en.rs`, so the count was unconditionally the whole file and the guarded branch could never be taken. The dead search and its condition are gone and the doc comment now describes what the code does.
- **D6** the `scp-transfer` field list in `skills/ssh-cli-en/SKILL.md` gained `ok`, matching the pt-BR skill that already had all four. The parity gate's token list gained `ok`/`direction`, `mtime_preserved` and `durable` — the three that actually discriminate, since the bare field names already appear in both skills through other contexts and the check is `contains`.
- **E1** `docs/TESTING.md` and its pt-BR twin catalogued `tests/gaps_v063_gate_falsification.rs`, a suite this repository never had. The bullet promised that every check of `scripts/check_en_identifiers.sh` had a standing falsifiability proof, and the trail ended at a filename. The forward sweep — is every suite on disk listed? — returned a clean zero, because a phantom is invisible from that side. Both halves now name the artefacts that exist, and `tests/gaps_v070_doc_reference_reach.rs` sweeps in both directions so the class cannot return.
- **E2** `exec`, `sudo-exec` and `su-exec` schemas described `host_resolved` as a field of its own — "VPS the command ran on" — while `scp-transfer`, `sftp-transfer` and `health-check` in the same directory correctly labelled it a compatibility alias of `target_resolved`. Three schemas of one release taught two different answers to "which field is canonical". Worse, they promised the field goes `empty` on old events, while `error-envelope.schema.json` promises `Absent — not empty` and argues why: an empty string asserts a host was chosen and has no name. A consumer written against the wrong one reads absence as a hostname or a hostname as absence.
- **E3** `active_fallback` in `error-envelope.schema.json` anchored its meaning in `host_resolved` and `host_source`, never naming the canonical pair — the alias explaining the canonical field instead of the reverse.
- **E4** `docs/MIGRATION.md` titled its wire section "0.5.3 wire format — current" while the line below it read "Product line: 0.5.5", and listed no wire change from this release at all. The schema version is still 3, so the heading was defensible and still misread as "0.5.3 is the current release". The heading now dates the format and states its reach, and the section carries the 0.5.5 additions: the provenance trio, its absence rule, the exit 64/66 split and the fleet selectors.
- **E5** fourteen command groupings folded several leaves into one token — `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` in both `docs/AGENTS.md`, `ssh-cli exec|sudo-exec|su-exec` in the agent docs and in `docs/schemas/README.md`, plus the `locale`, `tls` and `secrets` families. The folded form satisfies a human reader and returns nothing to a literal search for `ssh-cli sftp download`, which is how an agent looks a command up: the substring is simply not in the file. Unfolded, the searchable-with-prefix count in `docs/AGENTS.md` went from 16 of 47 to 38 of 47, with the remainder carried by the inventory table that `every_command_appears_in_every_full_inventory_document` already gates.
- **E6** `SECURITY.pt-BR.md` had lost five technical items its English twin carried, in a section about transport crypto: the rustls CVE window (`0.23.13–0.23.17`) a reader must avoid, the four `tls mtls` subcommands, the absolute-path alternative on the VPS record, the `instant-acme` crate, and the `0o600` mode under `tls/acme/`. mTLS and ACME had been folded into one bullet — 8 bullets in English against 7 in Portuguese — and the reason compression is disabled (side-channel reduction) was gone. A reader in Portuguese could not tell which rustls versions to avoid.
- **E7** the security document contradicted itself about the release it documents. The supported-versions table marked all of `0.5.x` as current with no caveat, while the section 60 lines below explains that `0.5.0`–`0.5.4` carry GAP-SSH-EXEC-ARGC-001; the assets list named "the ability to run remote shell / SCP / local port forwards" without the **identity of the host reached**, although the new section argues the critical dimension is not what runs but *where*; and accepted residual risk (2) still read that the remote host is trusted "for the one-shot command the operator requested", which presumes the host *is* the request — the exact premise this release broke. All three now state the qualification.
- **E8** `docs/RELEASE_CHECKLIST.md` was missing one policy rule its pt-BR twin carried: never publish `gaps.md` nor assert its FIXED text from product tests (G13/G15). Ninety bullets against ninety-one — the kind of gap a reading pass does not surface and a per-section count does immediately. All seven bilingual pairs under `docs/` now match on H2, H3, bullet and fence counts.
- **E9** the exec-family schemas listed `host_resolved` and `host_source` in `required`, and `health-check.schema.json` listed `host_source` but not `host_resolved` — an asymmetry that marks drift rather than a decision. A compatibility alias inside `required` stops being an alias: the whole point of one is that a later major can drop it without breaking validation, and requiring it closes that door. `scp-transfer` and `sftp-transfer` from the same release already required only the canonical pair, and `tests/gaps_v061_scp_durability.rs` states the underlying rule for a different field: a name in `required` invalidates every event that lacks it. All six now require `target_resolved`/`target_source` and leave the aliases optional; the CLI keeps emitting both, so no existing consumer sees a change.
- **E10** `health-check.schema.json` escaped the alias correction entirely, and the reason is instructive: that pass was driven by searching for the word `empty`, the false promise the exec family carried, and this file never used it. Its defect was structural instead. `HealthCheckJson` flattens `TargetEcho`, whose four fields are all unconditional, so the success envelope always carries `host_resolved` — and the schema never declared it. The canonical `target_source` pointed at the alias `host_source` as though the alias held the meaning, the reverse of the pattern every other schema follows. Nothing failed validation, because `additionalProperties` is `true`: the defect was silent. The lesson is that auditing an alias means diffing the fields a DTO *serializes* against the schema's `properties`, not grepping for suspicious phrases.
- **E11** three further schema-versus-emitter gaps, all silent for the same reason: `error_code` was described as "Present on every failure envelope" while sitting outside `required`, though `print_error_envelope` writes it as an unconditional `&str`; `vps-show.schema.json` declared none of the five conditional fields `MaskedVpsJson` emits (`tags`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`), and `vps-list.schema.json` `$ref`s that item, so the gap propagated to the listing; and eleven of the twenty-two schemas had no top-level `description`, which matters because `ssh-cli schema <name>` hands back the raw body and half of them did not say which command they belonged to.
- **E12** `docs/schemas/README.md` told parsers to branch on `event` without listing the payloads that carry none. Measured against the binary: `exec`, `sudo-exec`, `su-exec` and `vps-show` emit an object with no `event` key, and `vps list` emits a bare array with no envelope at all. A parser treating a missing `event` as malformed rejects five valid payloads. The note about CRUD events without schemas also hid four of them behind an ellipsis — `vps-import` most consequentially, since it already appears in the `operation` enum of `dry-run.schema.json`, which sends the reader hunting for a schema of the real event.
- **E13** `docs/schemas/README.md` carried the one command grouping in the repository with no individual table in the same document to compensate: `ssh-cli sftp mkdir|rmdir|rm|rename|stat` collapsed five leaves into a token that matches none of them, and the batch lines did the same for `scp`/`sftp upload|download`. Elsewhere a grouping sits beside a table listing each leaf, so search still finds it; here nothing did. All nine transfer leaves are now literally searchable with the `ssh-cli` prefix. `docs/RELEASE_CHECKLIST.md` also ended on a blank line, alone among the fourteen files of the seven pairs — the kind of drift a line-count check reports as parity because two errors cancel: the English file's trailing blank exactly offset the extra bullet the Portuguese carried.


## [0.5.4] - 2026-08-06

### Security
- **A1** Fixed a remote pre-authentication denial of service. `auth_banner` sliced the server-sent banner at byte 512 (`&banner[..512]`); a multi-byte character on that boundary panicked, and because the release profile sets `panic = "abort"` the whole process died — taking any `--all` fan-out with it. Truncation is now on character boundaries.
- **A2** ACME/mTLS private keys are no longer briefly world-readable. `write_secret_file` used `std::fs::write` (creating at `0644` under the default umask) and narrowed permissions afterwards with the error discarded. It now delegates to a shared helper that creates at `0600` via `O_EXCL` and propagates permission failures.
- **A3** `setuid`/`setgid`/`sticky` bits sent by the server are no longer reproduced on downloaded files. Inbound modes are masked with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) in both SFTP and SCP; outbound modes from local files keep the full mask.
- **A6** `parse_hex_key` rejects non-ASCII before slicing, so a `secrets.key` containing multi-byte UTF-8 totalling 64 bytes returns a typed error instead of panicking.

### Added
- **C1** Agent-native payload shaping as global flags: `--select`/`--fields`, `--filter` (repeatable, AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`. Applied at the single JSON serialization funnel *before* the envelope is built, so the oversized payload never exists. Measured: `vps list` drops from 943 to 19 bytes with `--select name --limit 1`. A malformed `--filter` fails at parse time (exit 64) rather than silently matching nothing.
- **C2** `--no-input` refuses stdin declaratively instead of blocking forever on an absent human.
- **C2** `--dry-run` prints the plan for a destructive operation and exits without executing it: `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`. The plan is JSON even in text mode (`docs/schemas/dry-run.schema.json`), because a preview exists to be diffed. On any other command the flag is **rejected with exit 64** rather than accepted and ignored — the failure mode `--no-input` shipped with. Preconditions run first, so previewing the removal of an absent host still exits 66 instead of promising a success the real run would not deliver.
- **G-TUN-R01** `tunnel --reverse` asks the server to listen and delivers connections back to a local port. `REMOTE_PORT 0` is accepted in this mode only: the server allocates and reports the port it bound, which is what reaches `tunnel_listening.local_port`. A remote bind outside loopback requires `--i-accept-network-exposure` — in this direction the exposed end is the server's listener, so guarding the local `--bind` would check the wrong side.
- **G-TUN-R02** `tunnel --socks5` serves a SOCKS5 proxy (RFC 1928, no-auth + `CONNECT`); every accepted connection becomes one `direct-tcpip` channel. Host names are forwarded unresolved so they mean what they mean on the *remote* side. `BIND` and `UDP ASSOCIATE` are answered with reply code `0x07` instead of a bare close. The handshake is capped at 1024 bytes, above the 519 the RFC permits and far below anything usable to make the proxy buffer at will.
- **G-TUN-R03** `tunnel --remote-socket <PATH>` forwards a local port to a Unix domain socket on the remote host (`direct-streamlocal@openssh.com`). The path is validated as absolute and NUL-free; it is deliberately *not* checked against the local filesystem, which has nothing to do with the server's.
- **Tunnel wire** `tunnel_listening` and `tunnel_closed` gained `mode` (`local` / `socks5` / `streamlocal` / `reverse`). It is the discriminator that says how to read the sibling fields: under `reverse` the listener is the server's, and under `socks5` there is no single destination. It defaults to `local`, so events written before 0.5.4 still validate.
- **G-TUN-R07** New `tunnel_closed` event (`docs/schemas/tunnel-closed.schema.json`) with `reason` (`deadline` / `signal` / `accept_error`), `forwards_served`, `capacity_waits` and `duration_ms`. Those three endings previously shared exit 0 and were indistinguishable.
- **G-TUN-R06** `tunnel_listening` gained the `bind` field, so an agent can audit from the contract alone whether a service was published beyond loopback (additive; existing consumers unaffected).

### Changed
- **BREAKING — G-ERR-R02** Partial multi-host failure now exits **1** with `error_code: "partial_failure"` and `error_class: "partial"`, carrying `failed`/`total`. It previously exited **65**, the same code as malformed TOML, so an agent could not tell "1 of 10 hosts failed" from "the config is corrupt". Exit 65 is again reserved for genuine data errors.
- **BREAKING — G-TUN-R13** A non-loopback `--bind` now requires `--i-accept-network-exposure`; without it the tunnel exits **64** before any network I/O. `--bind 0.0.0.0` previously published the forwarded remote service to the local network in silence.
- **G-TUN-R08** `--bind` is validated as an IP address by clap, so a typo fails at parse time (exit 2) instead of after a full SSH handshake.
- **G-TUN-R09** Bind failures map by `ErrorKind`: `AddrInUse`/`PermissionDenied` → 74 (retryable), `AddrNotAvailable`/`InvalidInput` → 64. All of them previously collapsed into 65 classified permanent.

### Fixed
- **C2** `--no-input` now refuses stdin on `vps add` and `vps edit`. The refusal was implemented in the exec/scp/tunnel override path only, so both registry commands — the ones most likely to run unattended — accepted the flag and read the password anyway. The guard moved into `read_secret_stdin`, so every caller inherits it. Covered by `tests/gaps_v059_agent_native.rs`, which did not exist when the flag shipped.
- **Docs gate** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` is green again: 56 broken intra-doc links across 14 sites are now fully qualified, and links to `pub(crate)` items (`take_utf8_capped`, `EXEC_CAPTURE_HARD_MAX_BYTES`, `lock_config`) became inline code, since private items have no public page to point at.
- **E3** `tunnel` honours `--use-agent` and `--agent-socket`. clap accepted both and the dispatch discarded them, so a host registered for agent auth could not open a tunnel at all.
- **G-TUN-R10/R11** Forward copy uses `tokio::io::copy_bidirectional`, which reports per-direction byte counts and the error. The previous two-`copy` `join!` discarded both `Result`s, making a mid-transfer failure indistinguishable from a clean finish at any verbosity.
- **G-TUN-R12** Concurrency saturation is announced once per run and reported as `capacity_waits`; it was previously invisible and surfaced only as unexplained latency.
- **E4** The human banner prints the effective bind address instead of a hard-coded `localhost:`.
- **E7** `open_tunnel_channel` preserves the underlying error via `channel_src` instead of flattening it into a string.
- **E6** `tunnel-listening.schema.json` matches the clap range for `remote_port`. With the modes added later in this release the shared minimum is 0, since `socks5` and `streamlocal` have no single remote port and `reverse` uses 0 to mean "server allocates"; `local_port` keeps `minimum: 1`, which is the field an agent connects to.
- **B2** SFTP verifies byte counts in **both** directions. SCP had checked `sent != size` and `received != size` since 0.5.2 while SFTP checked neither, so a truncated SFTP transfer reported `ok` with a plausible `bytes` field — the loop counted what it wrote, never what survived. Uploads now compare against the announced local size *and* re-`stat` the remote (the destination-effect proof: a quota or a full filesystem is invisible otherwise); downloads verify before the atomic rename, so a short read never reaches the final path. Servers that omit `size` are accepted rather than failed, since the attribute is optional in the protocol.

### Internal
- **G-QA-R01** New `tests/test_quality.rs` fails the build on assertions between two literals or `assert!(true)`. `src/tunnel.rs` shipped `assert_eq!(0_u64, 0)` named `timeout_zero_conceptually_rejected`, which exercised no product code while implying the one-shot guard was covered.
- **G-QA-R03** Documentation assertions moved to `tests/docs_conformance.rs`, so editing prose no longer turns the behavioural gate red without a functional regression.
- **G-DOC-R01/G-DOC-R02** Closed alongside the split above: `CONTRIBUTING.md` now lists every test suite by name instead of naming `gaps_v040` as the whole surface, and the pt-BR skill documents the `scp-transfer` fields (`ok`, `direction`, `bytes`, `duration_ms`) its English twin already did. Both were fixed in this release without being cited in it.
- **G-TUN-R04/R05, E1, E2** Tunnel tests now exercise real behaviour: the `timeout_ms == 0` guard is called, `run_tunnel_with_client` is actually invoked, and the ephemeral-port assertion checks the bound port instead of `include_str!`-matching its own source.
- **C3** The eight-line "N/M failed" block duplicated across five batch paths is replaced by `errors::finish_batch`.
- **G-TUN split** `src/tunnel.rs` is now a module root over `local`, `reverse`, `socks` and `streamlocal`. The three local-listener modes share one accept loop parameterised by destination: copying it would have meant maintaining the signal handling, admission gate, drain and saturation accounting in triplicate, which are exactly the parts that go subtly wrong in one copy and not the others. `run_tunnel` takes a `TunnelRequest` instead of fifteen positional arguments, where a transposition compiled cleanly and only surfaced as a tunnel pointing somewhere unintended.
- **G-QA-R01** `gap_tun_003_source_local_addr` no longer greps `src/tunnel.rs` for the strings `local_addr()` and `effective_port`. That assertion passed when the text sat in a comment, kept passing after the behaviour was deleted, and failed for the one reason that is not a regression — the module split moved the code. It now drives the real accept loop with an injected client and reads the port the product published.
- Unsolicited `forwarded-tcpip` and `forwarded-streamlocal` channels are rejected with `AdministrativelyProhibited` instead of russh's default of accepting and dropping them. The inbound queue is bounded, so a server cannot grow client memory by opening channels faster than they are drained; a full queue applies backpressure over the wire.

## [0.5.3] - 2026-07-30

### Fixed
- **G1** SFTP upload no longer truncates destination to zero bytes (`FileAttributes::empty()` instead of `Default`).
- **G2** `-v` filter is crate-scoped (`warn,ssh_cli=…`); never bare global `debug` (no password leak via `russh::client::encrypted`).
- **G3** SFTP SETSTAT sends `atime`+`mtime` together (no epoch atime).
- **G4** SFTP `set_metadata` Result is propagated (mutating SETSTAT is not best-effort).
- **G5/G17** Multi-file SCP/SFTP cancel fills cancelled remainder; `results.len() == input.len()`.
- **G6/G11** Signal-flag tests stay serial; `reset_flags_for_tests` for isolation; cancel cardinality tests.
- **G7** E2E real SSH covers SFTP upload/download checksum matrix + recursive tree (E17/E18).
- **G8** `exec --json` single-step emits exactly one NDJSON object (multi-step guard on JSON path).
- **G9** SCP download propagates `sync_data` failure before atomic rename.
- **G10** Formatting debt addressed with release gate (`cargo fmt --check`).
- **G12** Permission bits masked with `SFTP_PERM_MASK` (`0o7777`).
- **G13** Removed circular test that asserted FIXED text in `gaps.md`.
- **G14** Graduated verbosity: `-v`/`-vv`/`-vvv` via `ArgAction::Count` (info/debug/trace, always allowlisted).
- **G15** Inventory criteria require destination effect proof (checksum), not structural self-cert.
- **G16** English identifiers and channel errors in `client_real_scp.rs`.
- **G18** SFTP download local `set_permissions` failures are surfaced.
- **G19** Named `SFTP_PERM_MASK` constant (no bare magic mask).

### Changed
- Version **0.5.3**.

## [0.5.2] - 2026-07-19

### Added
- **Global `--json`** (G-AUD-01): agent-friendly alias that forces JSON output (clap `from_global` on subcommands).
- **`exec` / `sudo-exec` / `su-exec` active VPS** (G-AUD-04): one positional = COMMAND against `connect` active host.
- **`vps path` JSON envelope** (G-AUD-02): `event: vps-path` when format is JSON.
- **`fs_perm` module** (G-AUD-24): single source for secret file/dir Unix modes.

### Fixed
- **Password argv warning false positive** (G-AUD-08): inspect real `Option` fields, not `Debug` strings.
- **TLS missing PEM files** (G-AUD-05): `FileNotFound` / permanent class (not exit 74 retryable).
- **`vps export` honors global JSON format** (G-AUD-03).
- **ACME account create requires `--contact mailto:…`** (G-AUD-06/28).
- **Primary auth mutual exclusion** on write (G-AUD-07): exactly one of password / key / agent.
- **Secrets error messages** no longer advertise env key stores (G-AUD-21).
- **Log filter CLI-only** (G-AUD-22): ambient `RUST_LOG` ignored; use `-v`.
- **Concurrency hard cap single source** (G-AUD-19/23): `constants::MAX_CONCURRENCY`.
- **Skill description ≤1024** chars (G-AUD-15).
- **ACME validation permanent** (G-E2E-01): `invalidContact` / 4xx problem types → exit 64 non-retryable (`tls/acme_error_map.rs`).
- **Single JSON on `vps add` auto-key** (G-E2E-04): fold `secrets_key_auto_created` into `vps-added` (one document).
- **Root `schema` + `doctor`** (G-E2E-02/03): agent discovery parity.
- **Version `-dirty` with `.commit_hash`** (G-E2E-06): honest provenance on dirty trees.
- **clap `env` feature removed** (G-E2E-08); help no longer teaches env stores (G-E2E-07).
- **Export redacted mask** (G-E2E-10): `***` via `FIXED_MASK`, not empty string.
- **`vps add --use-agent`** (G-E2E-19): registry auth triplo complete.
- **E2E harness offline SKIP** + release default bin (G-E2E-05); EN test idents (G-E2E-13).

### Removed
- **`.github/workflows`** (G-AUD-11): local gates only; no product CI/GH Actions in tree.
- **`src/erros.rs` PT shim** (G-AUD-14).
- **Product env config stores** `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` reads (G-AUD-12).

### Changed
- Version **0.5.1 → 0.5.2**.
- Integration tests aligned to CLI/XDG-only config (no env secrets/format store).

### G-SFTP residual harden R01–R15

### Security
- **Entry-name validation** + `ensure_local_under` on recursive/multi-file download (malicious SFTP server cannot escape local dest).
- **Partial cleanup** on every SFTP download error (parity with SCP).
- **Upload tree root** uses `symlink_metadata` (no-follow).

### Changed
- **Outer wall-clock timeout** (`under_timeout`) on multi-file and FS ops (`ls|mkdir|rm|stat|rename`).
- **`cli/scp_args.rs`** extracted (SRP; monólito `cli/mod` shrink).
- Docs/skills/llms: SCP = regular files; trees/FS = **`sftp`**.

### G-SFTP: SFTP subsystem

### Added
- **`russh-sftp` 2.3** (feature `ssh-real`) — SFTP v3 client subsystem.
- **`ssh-cli sftp`:** `upload|download` (optional `--recursive`), `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`.
- **Modules:** `src/ssh/sftp_session.rs`, `sftp_path.rs`, `sftp_types.rs`, `src/sftp/`, `src/cli/sftp_args.rs`.
- **JSON schemas:** `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`.
- **Gates:** `tests/gaps_v057_sftp.rs`.
- Multi-host `--all`/`--hosts` for SFTP upload/download (`map_bounded`).

### Changed
- **ScpOptions / SftpOptions:** `use_agent` + `agent_socket` (G-SFTP-17/18; CLI/XDG only).
- Stream transfers in 32 KiB chunks; partial download + atomic rename; symlink no-follow on trees.

### Security
- Remote path validation (control chars / empty segments).
- Recursive depth cap; listing entry cap; no full-file heap via `SftpSession::read/write`.

### G-SSH: SSH client rules / russh

### Added
- **`src/ssh/client_handler.rs`:** TOFU `check_server_key` + `HostKeyOutcome` typed recovery (G-SSH-01/09/14).
- **`src/ssh/client_connect.rs`:** dial + auth chain file key → agent → password (G-SSH-04/16).
- **`src/ssh/key_material.rs`:** Unix 0600 private-key perms + RSA ≥2048 / DSA reject (G-SSH-03/07).
- **CLI/XDG agent:** `--use-agent`, `--agent-socket`; `VpsRecord.use_agent` / `agent_socket` (no env store).
- **Gates:** `tests/gaps_v056_ssh.rs`.

### Changed
- **SSH client id:** `SSH-2.0-ssh-cli` (no russh version banner fingerprinting).
- **Config policy:** explicit rekey 1h/1GiB, window 2MiB, packet 32KiB, TCP SO_KEEPALIVE via `socket2`.
- **deny.toml:** ban `ssh2`, `thrussh`, `libssh-rs`.
- SRP split of connect/handler/key load from session/SCP path.

### Security
- Host-key change returns typed `HostKeyChanged` (exit EX_NOPERM) to agents.
- Fail-closed when `known_hosts_path` is missing outside tests.
- Password fallback remains opt-in via non-empty inventory secret (documented).

### G-UNSAFE: unsafe code e FFI

### Added
- **`src/test_util/env.rs`:** encapsulated `set_var`/`remove_var` with `// SAFETY:` (ed2024-ready).
- **`src/vps/config_io.rs`:** config path/load/save/permissions split (SRP / G-UNSAFE-10).
- **Gates:** `tests/gaps_v055_unsafe_ffi.rs` (allowlist, forbid, main order, no plaintext env).

### Changed
- **`main`:** `signals::register_handler()` **before** Tokio `new_multi_thread` (G-UNSAFE-13 / signal-hook first-hook).
- **SIGTERM SAFETY** multi-bullet (async-signal-safe atomics only; double-hit justified vs `flag::register`).
- **Windows console FFI** module safety docs; one op per `unsafe` block unchanged.
- VPS tests use `set_runtime_flags` for plaintext opt-out (no `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` env).
- Secrets docs: fail-closed env key material; plaintext only via CLI flag.
- Concurrency docs: CLI `--max-concurrency` + auto formula only (no env store).
- `forbid(unsafe_code)` on `ssh/mod`, `vps/mod`, `vps/model`, `vps/config_io`.

### Security
- Product `unsafe` allowlist: `platform/windows.rs` + `signals.rs` only; test env mutation centralized.

### G-ERR: tratamento de erros

### Added
- **`SshCliError::Domain`**, **`Crypto`**, **`Config`** variants; TLS/channel structured errors with optional `source`.
- **Helpers:** `tls_msg` / `tls_src` / `channel_msg` / `channel_src` / `error_code()` (G-ERR-08/16).
- **JSON envelope `error_code`** field for agents (stable snake_case).
- **Gates:** `tests/gaps_v054_error_handling.rs`.
- **SSH client split (G-ERR-12):** `client.rs` facade + `client_real.rs` / `client_stub.rs` / `client_tests.rs`.

### Changed
- Display messages lowercase, no trailing period (G-ERR-01 / thiserror convention).
- `paths` validation returns `SshCliResult` (no `anyhow`/`bail!`).
- `VpsRecord::validate*` → `Result<(), DomainError>`.
- Secrets: env key material rejected (XDG + CLI only); concurrency `resolve_limit` ignores env store.
- `xdg_config_dir` uses `SshCliError::XdgDirectory`.

### Security
- No secret material in error Display for crypto ops; fail-closed on `SSH_CLI_SECRETS_KEY*` env store.

### G-DOM: tipos de domínio chrono/uuid/rust_decimal/url

### Added
- **Four domain crates (coordinated):** `chrono` 0.4.45 (`serde`+`clock`), `uuid` 1.24 (`v4`+`v7`+`serde`), `rust_decimal` 1.42 (`serde-with-str`+`macros`), `url` 2.5 (`serde`).
- **`src/domain/` split (SRP):** `error`, `names`, `ports`, `limits`, `command`, `time`, `ids`, `http_url`, `money`.
- **`Rfc3339Utc` / `AddedAt` / `CreatedAt`:** `DateTime<Utc>` newtype; VPS `added_at` + ACME timestamps.
- **`HttpsUrl` / `AcmeOrderUrl`:** HTTPS-only parse for ACME order resume (XDG).
- **`CorrelationId` (v4) / `BatchRunId` (v7):** multi-host batch JSON field `batch_run_id`.
- **`Money<C: Currency>`:** library-ready decimal money (not wired into VPS).
- **Gates:** `tests/gaps_v053_domain_types.rs` + proptest roundtrips (G-DOM-07/09).

### Changed
- Batch schemas (`health-check-batch`, `exec-batch`, `scp-batch`) require `batch_run_id` (UUID v7).
- Import `added_at` validates RFC 3339 (no length-only check).

### Security
- No `Local::now` in product sources; no `serde-float` on decimals; ACME URLs reject `http`/`data`/`javascript`.

### G-TLS product: rustls / SSH-over-TLS / mTLS / ACME

### Added
- **Feature `tls` (default):** `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`.
- **`CryptoProvider::install_default`** in binary `main` (aws_lc_rs only; libraries never reinstall).
- **`ClientConfig` builder** (`src/tls/client_config.rs`) with webpki-roots + optional mTLS client cert.
- **SSH-over-TLS:** `ConnectionConfig.tls` / VPS fields `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`; dial TCP → rustls → `russh::connect_stream`.
- **CLI `tls`:** `provider`, `paths`, `mtls import|list|show|remove`, `acme account create|show`, `acme issue --print-challenge`, `acme complete`, `acme status|list`.
- **XDG layout:** `tls/mtls/<name>/`, `tls/acme/account.json`, `tls/acme/<domain>/` (0o600 secrets).
- Residual suite updates in `tests/gaps_v052_tls_policy.rs`.

### Changed
- `deny.toml`: allow product `rustls`; keep bans on `openssl*`, `native-tls`, `libssh2-sys`, `ring`; allow license `CDLA-Permissive-2.0` (webpki-roots).
- PEM load via `rustls-pki-types::PemObject` (no unmaintained `rustls-pemfile`).

### G-TLS / rustls policy — prior session

### Added
- **`src/ssh/connect.rs`:** single `build_ssh_client_config` + Happy Eyeballs dial helpers (G-TLS-07/09).
- **Residual suite** `tests/gaps_v052_tls_policy.rs` — lockfile/deny bans, no flate2, SECURITY/README policy (G-TLS-03).
- **SECURITY Transport & crypto policy (G-TLS)** — SSH ≠ TLS; aws-lc-rs; future rustls-only if HTTP; local gates only.

### Changed
- **SSH compression `none` only** — preferred algorithms never offer zlib (G-TLS-04).
- **russh features:** drop `flate2` (G-TLS-05); keep `aws-lc-rs` only.
- **`deny.toml`:** ban `openssl`, `ring`, `rustls` in addition to `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02).
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: crypto policy surfaces (G-TLS-01/06/08/11/12).

### Security
- No product TLS stack; no OpenSSL/`native-tls`/`ring`/`rustls` in the dependency graph.
- No product OTEL. Secrets remain `SecretString` + XDG AEAD.

### Sistema de Tipos

### Added
- **Domain newtypes (G-TYPE-01…20):** `src/domain/` with `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort` (ephemeral ≠ SSH).
- **`ssh/session_io.rs`:** extracted `truncate_utf8` / capped UTF-8 helpers (G-TYPE-14).
- Zero-cost layout tests for `SshPort` / `Option<SshPort>` niche.

### Changed
- **`VpsRecord` / `ConnectionConfig`:** fields are domain-proven; `try_new` replaces infallible `new`.
- **`HostSelection`:** `VpsName` / `HostTag` (parse at CLI boundary).
- **`ExecOptions` / `ScpOptions`:** `Option<TimeoutMs>`; steps are `Vec<RemoteCommand>`.
- **CLI ports:** `value_parser!(u16).range(1..=65535)` on VPS add/edit and tunnel remote port; local bind still allows `0`.
- **`paths::validate_and_normalize` → `Result<VpsName>`** (proof retained).
- **Import JSON:** empty host/user rejected; builds via `VpsRecord::try_new`.

### Security
- Auth material check centralized (`secret_nonempty`); no secrets in domain error messages.
- No product OTEL.

### Session notes (validation / serde / componentization)

### Added
- **Validation pipeline (G-SERDE-01…14):** `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`; module `src/validation.rs` (parse → serde → validate → domain).
- **Host tags on agent JSON wire (G-SERDE-06):** `tags` on list/export/import DTOs with round-trip tests.
- **Structural config validation on load (G-SERDE-04):** empty host/user, port 0, limit/char limits, tags charset.
- **Fuzz target** `import_envelope` for import deserialize (G-SERDE-12).
- **`ssh/connection.rs`:** ConnectionConfig extracted (G-COMP-R).
- **`cli/tests.rs`:** CLI unit tests extracted from monólito (G-COMP-R).

### Changed
- **deny_unknown_fields** on TOML `ConfigFile` / `VpsRecord` (G-SERDE-05); import JSON remains Must-Ignore with warn (G-SERDE-14).
- **SCP multi-host fan-out** uses `Arc<ScpOptions>` (G-MEM-SCP); `apply_scp_options` takes `&ScpOptions`.
- **CI Actions pinned by commit SHA** (G-PROC-PIN).
- Serde deps caret: `serde = "1"`, `serde_json = "1"`.

### Security
- No product telemetry (OTEL still forbidden). Secrets stay in `SecretString`; validation never logs secrets.


### Prior closeout / process notes

### Changed
- **O1–O6 + process (mandatory):** global `--fail-fast` (`map_bounded_with`); host `tags` + `--tags` / `vps --tag`; multi-cmd `--step` one session; SCP `--scp-file-concurrency` parallel channel windows; Arc `ExecOptions` fan-out; proptest packing + fuzz targets; CI miri/geiger/sbom/proptest; `scripts/release_attest.sh`. Zero product telemetry.
- **Deep componentization (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01):** extract `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`); `ssh/scp_wire.rs`; `scp/{mod,batch}.rs`; `output/{mod,batch}.rs`; `cli/{mod,dispatch}.rs`; populate `commands/*` thin reexports. Residual EN renames (`metadata`, `config_path`). OPEN product security remains 0; monólitos inventoriáveis fatiados.
- **Componentization (G-COMP-02…04):** split `vps/doctor.rs`, `vps/import_export.rs`, and `vps/health.rs` out of the `vps` monolith (`mod.rs` ~2428 → ~1698 LOC); re-export `HostHealthResult` / `run_health_check` for existing callers.

### Security
- **Closeout meta-audit (G-CLOSE):** doctor/concurrency/SCP integer casts use `TryFrom` (no truncating `as`); remaining pure modules `forbid(unsafe_code)`; extract `vps/selection.rs` (SRP); re-ran context7-cli + docsrs-cli + duckduckgo for skill compliance.
- **Security development audit (G-SECDEV):** secrets cross the CLI boundary as `SecretString` (`read_secret_stdin` + `ExecOptions`/`ScpOptions`/tunnel/health overrides); pure modules `#![forbid(unsafe_code)]`; deny `clippy::mem_forget` + undocumented/multi-op unsafe; STRIDE map + CVSS v4 preference in `SECURITY.md` (+ pt-BR).
- **Defensive security audit (G-SEC):** deny `unsafe_op_in_unsafe_fn`; release
  `overflow-checks`; TOFU fingerprint constant-time compare; product paths
  free of `.unwrap`/`.expect`/`unreachable!` in CLI parsers, concurrency admit,
  and single-host selection arms; import port via `u16::try_from`;
  `SshCliError` is `#[non_exhaustive]`; threat model in `SECURITY.md` (+ pt-BR);
  CI job `cargo deny check` (`deny.toml`).

### Added
- **Retry audit (G-RETRY):** typed error classification (`ErrorClass` / `ErrorLayer` /
  `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) on `SshCliError`;
  named `retry::RetryConfig` with full-jitter backoff and agent defaults (max 2
  retries on exit 74); JSON error envelope fields `error_class`, `retryable`,
  `suggestion` + schema update. In-process auto-retry of non-idempotent remote
  ops remains **off** (agent re-invokes the process).

### Fixed
- **Network audit (G-NET):** SSH dial uses async DNS + Happy Eyeballs multi-address race
  (`net::dial_tcp` + `russh::client::connect_stream`); enable `TCP_NODELAY` and SSH
  keepalives (`15s` / max `3`); private-key load and known_hosts TOFU run in
  `spawn_blocking`; tunnel accept continues on transient errors and sets nodelay on
  local forwards.


### Changed
- **Hardcode audit (G-HC):** central `constants` module for XDG file names, env keys,
  app identity, network defaults (`DEFAULT_SSH_PORT`, tunnel bind/origin), process
  timing, AEAD/keyring sizes; single `paths::xdg_config_dir()` helper. No product
  secrets/URLs were present; hosts remain registry/CLI data.

### Fixed
- **External process audit (G-PROC):** `build.rs` git probes set explicit `Stdio`
  (null/piped); remote commands reject NUL before SSH exec packing; test
  `ssh-keygen` fixtures use direct argv + explicit stdio and skip if missing.
- Docs: CROSS_PLATFORM / AGENTS process-boundary policy (no local OpenSSH spawn;
  MSRV ≥ 1.77.2 BatBadBut; remote `sh -c` packing only on target host).

### Added
- **Multi-host bounded concurrency (modus operandi):** `health-check|exec|sudo-exec|su-exec|scp --all` fans out with `Semaphore` + `JoinSet` (cap from `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / auto CPUs×RAM formula, clamp 1..=64). Batch JSON: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`). Tunnel accept forwards use the same admission gate.
- **Selective multi-host `--hosts a,b,c`:** same bounded fan-out and batch JSON as `--all` (even for one name); unified via `HostSelection` + `resolve_host_jobs`.
- **Multi-file SCP (single-host, G-PAR-47):** `scp upload VPS f1 f2 … REMOTE_DIR` / download symmetric — **one SSH session**, serial transfers (auth once).
- **Multi-host × multi-file SCP (G-PAR-48):** `scp upload --all f1 f2 … REMOTE_DIR` / `--hosts a,b` — outer bounded fan-out per host session; files serial on each session; download writes under `LOCAL_DIR/<host>/`.
- **TOFU flock (G-PAR-49):** `known_hosts` mutations exclusive-lock + reload-merge under multi-host first-connect.
- **`vps doctor --probe-ssh [--hosts a,b]`:** single JSON root `event: vps-doctor` with `local` + optional `ssh_probe` (no dual roots).
- **`map_bounded` cancel:** stops admission on SIGINT/SIGTERM; `force_exit` aborts in-flight JoinSet (timer poll); debug logs `available_permits`; span `fan_out_unit` per task (G-PAR-52).
- Agent docs/skills: multi-host fleet + multi-file / cartesian SCP + doctor envelope contract.

### Changed
- SCP upload/download path validation and post-transfer metadata use `tokio::fs` / `spawn_blocking` (non-blocking Tokio workers under multi-host fan-out).
- `scripts/dist_multiarch.sh` supports `PARALLEL_JOBS` (default 2) via `xargs -P`.

## [0.5.1] - 2026-07-17

### Fixed
- **Import/export agent roundtrip**: `vps export` default body is **TOML** even on non-TTY; JSON only with `--json`. Import accepts TOML (EN+PT keys) and JSON `vps-export` envelopes (GAP-AUD-001/022).
- **Wire dual-read**: deserialize EN + legacy PT aliases; serialize English keys; schema **v3**; default `added_at` when missing (GAP-AUD-002/021). Supersedes 0.5.0 wire note (PT keys via `serde(rename)` only).
- **`secrets init` / `reencrypt` JSON** envelopes (`event: secrets-init|secrets-reencrypt`) via `--json` or `--output-format json` (GAP-AUD-003).
- Empty command error is English technical (`empty command`) under any locale (GAP-AUD-004).
- CRUD/connect/import success paths emit structured JSON when format is JSON (GAP-AUD-008).
- SCP remote-missing message normalized to `file not found: <path>` (GAP-AUD-025); EC 66 retained.
- Import TOML parse errors map to sysexits **65** (`TomlDe`) (GAP-AUD-012).
- `SshAuthentication` exit code aligned to **77** (GAP-AUD-020).
- Timeout values `< 1000` ms emit a stderr warning (GAP-AUD-009).
- `--include-secrets` to pipe/non-TTY requires `--output` or `--i-understand-secrets-on-stdout` (GAP-AUD-011).
- Doctor `secrets_plaintext_opt_out` is JSON **bool** (GAP-AUD-013).

### Added
- CLI flags: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (env layers deprecated, still work) (GAP-AUD-006).
- Event `secrets-key-auto-created` when primary key is provisioned on first write (GAP-AUD-007).
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018).
- Password-on-argv stderr warning (GAP-AUD-010).

### Changed
- Version **0.5.0 → 0.5.1**.
- Tracing / residual identifiers standardized to English (GAP-AUD-005).
- Portuguese type aliases in `erros` module marked deprecated (GAP-AUD-017).

### Notes
- No crates.io/GitHub publish without explicit maintainer OK.
- SCP real transfer contracts from 0.5.0 §1.1 must not regress.

## [0.5.0] - 2026-07-15

### Fixed
- **CRITICAL**: `secrets init --force` now re-encrypts existing host secrets under the new primary key and writes `secrets.key.bak` (GAP-AUD-SEC-001).
- Doctor `permissions` field uses English `"missing"` instead of Portuguese `"ausente"`.
- Technical error messages, clap help residual PT, and product identifiers standardized to English.
- VPS names with internal whitespace are rejected (GAP-AUD-VAL-001).

### Changed
- Semver **0.5.0**: English renames of public/lib identifiers and residual API surface (`generate_completions`, `plaintext_allowed`, `ENC_PREFIX`, `verify_tofu`, etc.). Wire TOML keys were still Portuguese via `serde(rename)` in this release (**superseded in 0.5.1** by English serialize + dual-read EN/PT aliases, schema v3).
- `secrets init` / `secrets reencrypt` success lines go through `Message` i18n.

### Notes
- No crates.io/GitHub publish in this change set without explicit maintainer OK.

## [0.4.2] - 2026-07-15

### Fixed
- **Tunnel ephemeral port** (`local_port=0`): after bind, JSON/banner report the **OS-assigned** port via `local_addr()` (never `0` post-bind) (GAP-SSH-TUN-003). Schema `local_port.minimum` is 1.
- **SCP remote missing** now exits **66** `ArquivoNaoEncontrado` (parity with local missing) instead of **74** `CanalFalhou` when OpenSSH reports `No such file` / `not found` (GAP-SSH-IO-010). Protocol/permission errors remain 74.

### Added
- `vps export --json` agent-first envelope: `event: "vps-export"`, redacted hosts by default, no `sshcli-enc:` for empty secrets (GAP-SSH-UX-001 / EXP-001 parity); schema `docs/schemas/vps-export.schema.json`
- Commit hash embed for crates.io: `build.rs` precedence `SSH_CLI_COMMIT_HASH` → `.commit_hash` pack file → git → `unknown` (GAP-SSH-REL-007)
- Official e2e **E15** (tunnel port 0) + **E16** (symlink) + E13 asserts exit **66**; ENV-001 fail2ban policy in script header (GAP-SSH-ENV-001, SCP-024)
- Suite `tests/gaps_v042_integration.rs`

### Changed
- Version 0.4.1 → **0.4.2**
- Product-line docs + skills: tunnel remains **positional** args; port `0` = ephemeral; trust JSON `local_port` after bind; never invent `--local-port` (GAP-SSH-DOC-042)

### Security / honesty
- VPS TCP ban after audit e2e was **fail2ban** from intentional wrong-password tests (ENV-001), **not** TUN-003. Whitelist/ignoreip is ops; CLI one-shot does not open sshd permanently.
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)


## [0.4.1] - 2026-07-15

### Fixed
- **Export redacted empty secret** no longer emits `sshcli-enc:v1:…` ciphertext for password `""` (GAP-SSH-EXP-001). Empty secrets serialize as empty strings so cross-machine import of skeletons stays honest.
- **Tunnel one-shot deadline** after local bind no longer returns exit **74** `TimeoutSsh` when the agent already received `tunnel_listening` (GAP-SSH-TUN-002). Pre-bind timeout remains exit 74.

### Added
- `tunnel` auth flag parity with exec/scp: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- `health-check` auth flag parity: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- SCP success JSON field `event: \"scp-transfer\"` + schema required (GAP-SSH-IO-009)
- Suite `tests/gaps_v041_integration.rs` (AUD-POST regression)
- `health-check` honors global `--replace-host-key` and enables JSON error envelope with `--json`

### Changed
- Version 0.4.0 → **0.4.1**
- Product-line docs + skills document tunnel/health auth parity and scp-transfer event

### Security / honesty
- **If you installed 0.4.0 from crates.io:** redacted `vps export` could show fake empty-password ciphertext; tunnel agents could see `ok:true` then exit 74. Upgrade to **0.4.1**.
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)

## [0.4.0] - 2026-07-15

### Fixed
- **SCP wire protocol** was broken on crates.io **0.3.9** (header used literal `\\n` instead of real newline `0x0a`; ACK/EOF sent empty data instead of byte `0x00`; status not validated; download header/terminator incorrect) — SCP-010..013
- Remote SCP path shell-escape for spaces and meta-characters (SCP-014)
- Unit tests no longer crystalize the broken header form (SCP-015)
- Download no longer leaves a partial final file on failure: write `{path}.ssh-cli.partial` then atomic rename (SCP-022); mode/times applied on the **partial** before rename (SCP-022b)
- Upload no longer loads the entire file into RAM (`fs::read`); streams in 32 KiB chunks (SCP-018)
- `scp --json` enables the JSON error envelope on stderr (parity with tunnel; IO-007b)
- SCP file-only validation messages are i18n EN/PT (SCP-020b)

### Added
- Official e2e E10–E14 SCP coverage in `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, missing remote, mode/mtime preserve) (SCP-016, SCP-023)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- Structured SCP success JSON + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-directional: remote `scp -tp`/`-fp`, `T` line + `C` mode parse, set_permissions + set_times (SCP-023/023b; e2e E14)
- `tunnel --json` emits structured `tunnel_listening` event after local bind (IO-008)
- i18n EN/PT success messages for SCP (SCP-020)
- Suite `tests/gaps_v040_integration.rs` (TEST-004)

### Changed
- Version 0.3.9 → **0.4.0**
- Product-line docs document **regular files only** (no `-r` / no SFTP subsystem) and the 0.3.9 SCP wire regression (DOC-004, SCP-019, REL-004)
- Root docs honesty (SECURITY 0.4.x current, INTEGRATIONS real 0.4.0 surface, CONTRIBUTING gaps_v040) (DOC-004b)
- `docs/*` honesty: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + schemas index cover SCP file-only, partial, 32 KiB stream, preserve, `scp --json`, `tunnel --json` / `tunnel_listening`, and 0.3.9 wire warning (DOC-004c)
- `skills/*` honesty: bilingual agent skills + evals teach SCP file-only, scp-transfer JSON, `.ssh-cli.partial`, 32 KiB stream, mtime/mode preserve, tunnel `--json` / `tunnel_listening`, timeout flag matrix (DOC-004d)
- Added `docs/schemas/tunnel-listening.schema.json` for IO-008 agent contract
- `scp` honors global `--replace-host-key` and global `--output-format json`

### Security / honesty
- **If you installed 0.3.9 from crates.io and used `scp`:** that release advertised SCP but the wire implementation was inoperant (upload often produced 0-byte remote files or timed out). Upgrade to **0.4.0**.
- No telemetry

### Notes
- One-shot CLI: connect → transfer → disconnect → exit
- Large files: raise `--timeout` (covers connect + full transfer)

## [0.3.9] - 2026-07-15

### Fixed
- Post-0.3.8 audit residuals: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Default tracing level is **error** (agent-first); `-v` enables debug (LOG-001)
- Tunnel/JSON stderr no longer emits INFO progress banners by default (LOG-001)
- Key-only VPS JSON: empty password serializes as `null` instead of `"***"` (JSON-001)
- `health-check --timeout <ms>` override aligned with exec (CLI-004)
- Product-line docs bumped to **0.3.9** and residual behaviors documented across README, `llms*.txt`, INTEGRATIONS, `docs/*` (HOW_TO_USE, COOKBOOK, MIGRATION, TESTING, CROSS_PLATFORM, AGENTS, schemas), and skills (DOC-003 deep audit)
- CHANGELOG compare anchors for 0.3.8/0.3.9 (CHG-001)
- `deny.toml` documents expected multi-version warns without CVE ignore (DENY-002)
- `docs/schemas/vps-show.schema.json` allows `password` type `string | null` (JSON-001 contract parity with runtime)
- Portuguese cross-language openers in `docs/*.pt-BR.md` use Portuguese narrative ("Leia este documento em inglês")
- Bilingual `docs/RELEASE_CHECKLIST.md` + `docs/RELEASE_CHECKLIST.pt-BR.md` with residual gates LOG/JSON/CLI/DOC/DENY/REL/CHG
- DOC-003 tests cover checklists and schema password null window
- Skills EN/PT consolidated as imperative operational formulas (LOG/JSON/CLI, error envelope, quiet, key-passphrase-stdin, port, full completions) without version changelog stories
- Workspace secret-hygiene residuals SEC-001..003: ignore `.setting.cyber/` fully, E2E refuses grok config inside the repo, docs use `demo-password-not-real` (not `s3cret`)

### Added
- Suite `tests/gaps_v039_integration.rs` for residual audit gaps (incl. SEC-001..003)

### Changed
- Version 0.3.8 → 0.3.9
- Cargo package `exclude` adds `.setting.cyber/` and enrich-queue sqlite sidecars

### Notes
- No telemetry
- Real credentials stay outside the tree (`~/.config/ssh-cli/`, `$HOME/.grok/config.toml`)

## [0.3.8] - 2026-07-15

### Fixed
- Residual gaps post-0.3.7 audit (IO-006, EXIT-002, VAL-004, TEST-004, DOC-001, REL-001/002, DENY-001, PROC-001, E2E-001)
- Tunnel human banners no longer pollute agent stdout (JSON/non-TTY/quiet) (IO-006)
- No active VPS returns sysexits 66 via typed `ErroSshCli::NenhumaVpsAtiva` (EXIT-002)
- OpenSSH private key parse on VPS write-path after `is_file` (VAL-004)
- Full named regression suite `tests/gaps_v038_integration.rs` (TEST-004)
- Version string reports `-dirty` when working tree is dirty (REL-002)
- Inventory `gaps.md` versioned (DOC-001); release checklist `docs/RELEASE_CHECKLIST.md` (PROC-001)

### Security
- Upgrade **russh 0.62.2** (security floor ≥0.60.3); remove crypto COMPAT RC pins (DEP-002)
- `cargo deny`: `yanked=deny`, empty ignore list; drop dead `Unicode-DFS-2016` allow (DENY-001)
- Install resolve gate requires patched russh; allows stable primefield
- crossbeam-epoch ≥0.9.20 (RUSTSEC-2026-0204 / criterion dev)

### Changed
- Version 0.3.7 → 0.3.8
- `scripts/verify_install_resolve.sh` policy inverted (stable crypto allowed; patched russh required)

### Notes
- No telemetry (doctor reports `telemetry: false` only)
- Product fixes from uncommitted 0.3.7 ship in this release commit


### Added
- Full bilingual documentation framework (README, CONTRIBUTING, SECURITY, INTEGRATIONS, docs guides, schemas, skills)
- Dual license files `LICENSE-MIT` and `LICENSE-APACHE` with MIT OR Apache-2.0

## [0.3.7] - 2026-07-15

### Fixed
- All 23 gaps from `gaps.md` (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Domain write-path: `validar_e_normalizar`, port 1..=65535, key file exists (VAL-001..003)
- I/O: global `--output-format` on VPS CRUD, `health-check --json`, JSON error envelope, `--quiet` silences human success, `println!` only in `output` (IO-001..005)
- Tunnel `--timeout-ms` covers SSH connect + loop (TUN-001)
- SCP validates local file before connect (SCP-001)
- `vps remove` clears orphan `active`; lock file `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`; clap conflicts for password/*_stdin; completions broken-pipe safe (CLI-001..003)
- Signals tests `#[serial]`; help snapshot; non-tautological abort pattern test (TEST-001..003)
- Remote command failure uses process exit `EX_GENERAL` (not remote code) (EXIT-001)
- sudo/su password on channel stdin, not remote argv; mask always `***` (SEC-001, SEC-002)
- Import redacted UX + `--allow-incomplete` (IMP-001)
- `cargo deny` green with dated russh/crypto pin policy (DEP-001)

### Changed
- Version 0.3.6 → 0.3.7
- **Breaking (agent contracts):** long secrets no longer show 12+4 chars (always `***`); remote non-zero exit maps to process exit `1` with `remote_exit_code` in JSON error envelope
- `SSH_CLI_FORCE_TEXT=1` forces text output format (test/scripts)

### Security
- No sudo/su password in remote process list (`ps`)
- No password prefix leak in `vps list`/`show`

## [0.3.6] - 2026-07-15

### Added
- Default at-rest encryption: auto-create XDG `secrets.key` (0o600) on first secret write
- CLI `secrets status|init|reencrypt` (never prints master key)
- `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` opt-out for tests
- Doctor fields: `secrets_key_file`, `secrets_plaintext_opt_out`
- Script `scripts/e2e_real_ssh.sh` for real SSH E2E without logging credentials
- Auth failure message teaches `--password-stdin` / `--key` / passphrase stdin

### Changed
- Version 0.3.5 → 0.3.6
- GAP-009 residual: encryption is default (not merely optional)
- Pin freeze documentation for russh 0.60.0 + crypto RC pins (R-PINS)

### Security
- Secrets in `config.toml` encrypted by default (`sshcli-enc:v1:…`)
- E2E protocol forbids printing host/user/password and uses `/tmp` + password-stdin

## [0.3.5] - 2026-07-15

### Fixed
- Residual GAP-007: atomic `vps export` (tempfile + fsync + 0o600)
- Residual GAP-006: remote abort uses TERM then KILL with longer sanitized pattern
- Residual GAP-009/012: optional at-rest secret encryption (ChaCha20-Poly1305) via env/file/keyring
- README no longer recommends install without `--locked`
- gaps.md parity matrix updated for 0.3.4/0.3.5 reality

### Added
- `--key-passphrase` / `--key-passphrase-stdin` runtime overrides on exec/sudo-exec/su-exec
- Auto JSON when stdout is not a TTY (unless `--output-format` set)
- `vps doctor` reports `secrets_at_rest` and `secrets_key_source` (never prints secrets)
- Integration tests `tests/gaps_v035_integration.rs` (fake secrets only)

### Changed
- Version 0.3.4 → 0.3.5

## [0.3.4] - 2026-07-15

### Fixed
- `cargo install` crypto graph: pin `primefield`, `primeorder`, `ecdsa`, `pkcs5`, exact `russh = 0.60.0` (GAP-014)
- `sudo-exec` packing with `sh -c`  (GAP-005)
- Atomic `config.toml` write with tempfile + fsync + flock (GAP-007)
- Host key TOFU via XDG `known_hosts` (GAP-008)
- Dual `max_command_chars` / `max_output_chars` (GAP-004)
- Timeout remote abort best-effort (GAP-006)
- Credential validation: password or key required (GAP-011)

### Added
- Auth by private key (`--key`, `key_path`) via russh `load_secret_key` (GAP-002)
- `su-exec` one-shot consuming `senha_su` (GAP-003)
- `--password-stdin` / sudo / su stdin secrets (GAP-009)
- `vps doctor`, `vps export`, `vps import` (GAP-012)
- Tunnel mandatory `--timeout-ms` (GAP-010)
- `--disable-sudo`, `--description`, `--replace-host-key`
- Schema v2 multi-host XDG fields
- Install resolve gate: `scripts/verify_install_resolve.sh`

### Changed
- Default timeout 60000 ms 
- `directories` 5 → 6 (GAP-013)
- Version bump 0.3.3 → 0.3.4
- Dual license MIT OR Apache-2.0

## [0.3.3] - 2026-07-15

### Changed
- Migrated crate ownership and repository to `danilo-aguiar-br` after previous GitHub account ban (crates.io owner was `ghost_*`).
- `repository` / `homepage` now point to `https://github.com/danilo-aguiar-br/ssh-cli`.
- Author metadata updated to `Danilo Aguiar <daniloaguiarbr@proton.me>`.
- Removed GitHub Actions CI/CD workflows and CI badges — new repository ships without Actions.

### Note
- crates.io already had versions through `0.3.2` from the previous owner account; this release is the first under the new owner and repository URL.

## [0.2.1] - 2026-04-16

### Fixed
- Pin `elliptic-curve = "=0.14.0-rc.30"` to fix `cargo install ssh-cli` failure caused by incompatible `elliptic-curve 0.14.0-rc.31+` being resolved for `p256/p384/p521 0.14.0-rc.8`

## [0.2.0] - 2026-04-15

### Added
- Fix sudo-exec stdin password piping with `printf '%s\n'`
- Runtime overrides: --password, --sudo-password, --timeout flags on exec/sudo-exec/scp/tunnel
- LLM-friendly camelCase aliases (--sudoPassword, --suPassword)

## [0.1.0] - 2026-04-14

Initial release.

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.9...v0.4.0
[0.3.9]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.3.3
[0.2.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.1.0
