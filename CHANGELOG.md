# Changelog


- Read this document in [Portuguese (pt-BR)](CHANGELOG.pt-BR.md)
- All notable changes to this project are documented in this file
- The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)
- This project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
- Each release lists its sections in this order: Security, Added, Changed, Deprecated, Removed, Fixed, Internal


## [Unreleased]
- Nothing yet


## [0.5.6] - 2026-11-12

### Security
- `russh` 0.62.5 is now 0.65.1, which closes RUSTSEC-2026-0492 (HIGH, `client::Handler` callbacks fired for a channel that was never opened, fixed in 0.63.1) and RUSTSEC-2026-0495 (the agent server never called `Agent::confirm`, fixed in 0.65.0)
- The same upgrade closes GHSA-fw5c-8223-6h42, where a zero maximum packet size stalled every channel write, and GHSA-3hm4-p7h4-rvwp, a stack overflow before authentication triggered by a nested OpenSSH certificate
- `Handler::check_server_key` now receives `&PublicKeyOrCertificate`, and the certificate branch is unreachable because `Preferred::DEFAULT` leaves `host_key_certificates` empty, so the embedded key goes through the same TOFU check
- `rustix` 1.1.4, flagged unsound by RUSTSEC-2026-0486, is now 1.1.6, and the yanked `async-executor` 1.14.0 is now 1.14.1
- `deny.toml` skips `num-bigint@0.4.8` with its reason, because russh 0.65 moved to num-bigint 0.5 while secret-service 5.2.0, the newest release, still reaches 0.4
- The russh security floor was hand-written in three places that still passed the vulnerable 0.62 line, and `gap_dep_002_russh_patched_no_lock` and the `install-resolve` gate now both require `>= 0.65.0`, taken from RUSTSEC-2026-0495
- `webpki-roots` 1.0.12 drops eight more distrusted roots: emSign Root CA - C1 and emSign ECC Root CA - C3 in 1.0.11, then Buypass Class 2, Buypass Class 3, DigiCert Trusted Root G4, QuoVadis Root CA 1 G3, QuoVadis Root CA 3 G3 and Certigna Root CA in 1.0.12
- Gap 50: `ssh-cli secrets init --keyring` reported `{"ok":true,...,"key_source":"none"}` and stored nothing, because `keyring = "3"` resolved 3.6.3, which has no default feature and fell back to an in-process `mock` store
- The mock kept the value in the memory of one process, so the following `secrets status` reported `encryption_active:false` and the agent received a success it could not rely on
- `keyring` is now `4.2.0` with the `v1` feature, which enables the Keychain store on macOS, the Credential Manager store on Windows and the zbus Secret Service store on the remaining unix targets, and it fails closed with exit 69 when no store answers instead of writing to a mock
- Second cause found by measurement: the leaf flag `secrets init --keyring` never reached the runtime state, because `dispatch` forwarded only the global `--use-keyring`
- `secrets status` then re-resolved the key from flags alone and reported `none` over a key that had just been written
- Third cause: a key stored by a previous `secrets init --keyring` was invisible without repeating the flag, so `load_primary_key` now falls back to the store when no file and no flag exist
- The keyring entry is scoped by a digest of the resolved config directory, because the OS store is machine-wide while `secrets.key` and `config.toml` are per profile
- Without the scope an isolated `--config-dir` run overwrote the real profile's key
- D2: `--force` no longer writes `secrets.key.bak`, which violated the project rule against backup copies and left key material behind on every rotation
- The error that told the user to restore `secrets.key.bak` now points at `secrets reencrypt` with the previous key supplied by `--secrets-key-file`
- Gap 53: `secrets init --keyring` on a profile that already held `secrets.key` answered `key_source: keyring` and minted a second primary key instead of refusing
- The file kept answering first, so the next `secrets status` reported `xdg_file`, and every secret already at rest failed to decrypt with the key the command had just reported as active
- The same defect existed in the other direction: rotating from the store back to the file left the store entry alive and shadowing the file
- The guard now asks the profile, not the destination, so any existing key refuses the write without `--force`, and `--force` retires the other source so exactly one key survives the rotation
- A1: the TLS root store shipped a distrusted authority: `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 and `webpki-root-certs` 1.0.9 still carried `emSign ECC Root CA - G3`, whose eMudhra intermediates have a public private key (Mozilla Bug 2083449, CVE-2026-111943, RUSTSEC-2026-0381/0382/0383)
- `cargo deny check` was red on `advisories` and the client ACME and outbound TLS paths built their `RootCertStore` from that list
- The minimum is now declared in the manifest, `webpki-roots = "1.0.10"`, so a generic `cargo update` cannot walk the tree back to the compromised version without failing the build
- The lock resolves `rustls-webpki` 0.103.17 and `webpki-root-certs` 1.0.10, and `cargo deny check` reports advisories, bans, licenses and sources all ok
- A4: a new gate compares the advisory database with the resolved tree: `scripts/check_advisory_hits.py` runs `cargo audit --json` and fails on any hit in `vulnerabilities.list` or on a yanked package
- The previous `advisory-freshness` gate only measured the age of the clone, so a stale database could leave the gate green over a live CVE, which is exactly what the 2026-08-07 note in `deny.toml` records
- Text mode escapes terminal control sequences: A host name, an error message or a stored field could carry ESC, BEL or bidi characters, and text mode wrote them raw, so a hostile name could redraw the operator's terminal
- `src/output/sanitize.rs` (`escape_untrusted`, `escape_untrusted_keep_whitespace`) now turns control and formatting characters into `\u{1b}`-style text in the one text funnel: success, error and warning messages, `vps list`, `vps show`, `health-check`, `doctor` and the batch outputs
- A new host name has a strict charset: `VpsName::try_new` accepts ASCII letters, digits, dot, hyphen and underscore, starting with a letter or digit, 1 to 255 bytes, plus the file-name rules (no `..`, no trailing dot or space, no reserved Windows names such as `CON`)
- It guards `vps add`, the destination of `vps rename` and `vps edit --name`, and `vps import`
- Reading is lenient on purpose (see `Changed`), so an old `config.toml` keeps working
- The config directory is born private: A directory the CLI creates gets mode `0700` at every new level, where it used to inherit `0755` from the umask and expose host names and file sizes next to a `0600` `config.toml`
- An existing directory is tightened to `0700` only when it is the default XDG one, not a symlink, and owned by the current user
- An existing directory the CLI does not own by default is never chmodded: A `--config-dir` pointing at a directory that already exists keeps its mode, writes a log warning (visible with `-v`, because the default level is `error`) and shows up in `doctor` as `local.config_dir_exposed` and `local.config_dir_permissions`
- Names that become identities or remote fields are escaped or refused too: `tls mtls import --name` refuses the same control, formatting and bidi characters as host names (the ranges are in `Changed`) with exit 64 and creates no directory, while ordinary Unicode such as `serviço-1` is still accepted
- `sftp ls` and `sftp stat` escape `\n` and `\t` in the remote name in text mode
- Limits declared, not hidden: Remote `exec` stdout and stderr stay raw by design
- Windows has no chmod, so the directory relies on the profile ACL
- `--select` and `--truncate-content` reduce a single object such as the `doctor` report, and the six collection flags `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` stay inert there and are named in `agent_shape.ignored`
- JSON keeps every character of that set raw (U+202E and ZWSP, for example), because JSON is for machines, while ESC and BEL already leave `serde_json` escaped
- Error bodies stay in English by design, so agents get stable text
- A legacy name does not log a warning on load: `doctor` lists it

### Added
- `scripts/check_markdown_format.py` turns the formatting contract into an executable gate, because Gaps 25 and 37 named "the formatting linter" as their acceptance criterion and no such linter existed anywhere in the tree
- The gate runs as `markdown-format` in `scripts/check_all_gates.py`, and with the `doc` gate the battery now runs fourteen gates
- Thirteen rules cover bold, italic, horizontal rules, star bullets, bullets ending in a period or comma, H4 and deeper, H1 count, H1 and H2 spacing, the gap between a header and its first bullet, em dashes outside an H3, decorative blockquotes and emoji
- `--selftest` proves every rule fires on its own fixture, because a regex that matches nothing reports a clean tree for the same reason a broken smoke alarm reports no fire
- `docs_rules/` and `docs_prd/` are declared out of scope as imported source material, rather than skipped in silence
- `tests/gaps_v075_markdown_format_gate.rs` keeps the gate reachable from the battery, executable on the real tree, and free of third-party imports
- `src/secrets/keyring_store.rs` writes and reads on a dedicated OS thread, because the Linux store parks on an `async-io` reactor and the process runs a multi-thread Tokio runtime
- `secrets::delete_key_from_keyring` removes the entry of the current profile, so a deleted profile does not leave its primary key in the machine-wide store forever
- `AcmeDirectory::from_staging_flag` is the single decision point for `--staging`, replacing the two copies of the same conditional in the ACME leaves
- E2E cases E19, E20 and E21 cover the reduced `vps list --select` item against the published schema, `--select` composed with `--limit` plus its stderr accounting, and the keyring round-trip across two processes
- `docs/schemas/vps-list.schema.json` publishes two item shapes with `anyOf`: the full record through `vps-show.schema.json`, and the reduced record `--select` produces
- A2: `--select` and `--truncate-content` now reduce a single record, which used to be inert on `vps show`, `doctor`, `commands` and every other payload with no collection
- `ssh-cli commands --select tree.about` returns just the product name instead of 52699 bytes
- The reduced record carries `agent_shape` with `selected`, `content_truncated` and `ignored`, so a consumer can tell which of the two published shapes it holds
- `docs/schemas/vps-show.schema.json` and `docs/schemas/vps-doctor.schema.json` publish both shapes through `anyOf`, in the same two-variant spirit `vps-export` has used since Gap 35, instead of loosening the schema `additionalProperties`, and `anyOf` is used instead of `oneOf` because `--truncate-content` and the collection-only flags keep the record whole, so a `oneOf` would match both variants
- A6: every one of the 48 leaves now shows an `Examples:` block in `--help` while `-h` stays short, and a test runs every published argv with `--help` appended to prove it parses against the real flag set
- `vps rename <NAME> <NEW_NAME>`, a native rename: Before this, the only way to change a host name was `vps export`, edit the file, `vps import`
- That is a data-loss trap disguised as a procedure: the export redacts every secret, so the import either refused the file with exit 64 or, with `--allow-incomplete`, wrote back a host with no password, and the operator had to retype a credential the tool already held
- It was worse for agent-authenticated hosts, which lost their authentication entirely (see the `Fixed` entry below)
- A name is a label on a record, not a reason to touch its secrets
- `vps edit <NAME> --name <NEW>` reaches the same code path from the verb an operator was already using, so both spellings exist and neither is a second implementation
- Renaming never asks for the plaintext, and every credential stays readable under the new name: `VpsRecord` serializes secrets through the unbound `sshcli-enc:v2:` variants, so the AEAD associated data carries no host name and a rename has nothing to re-seal
- The ciphertext token itself does change, because every save seals again with a fresh nonce: that is normal AEAD behavior and it never touches the plaintext
- The test asserts the real property rather than byte equality: after the rename, `vps export --include-secrets` still returns the original credential under the new key
- If a future version binds the ciphertext to the host, that test is what fails first
- Eight clear flags, one per field: `--clear-tags`, `--no-key`, `--no-agent-socket`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert` and `--no-tls-client-key` remove a value
- Until now every optional field could be overwritten and none could be removed, so an operator could set a TLS SNI and never unset it
- The clear flags use dedicated booleans rather than sentinel values, because `--tls-sni ""` cannot distinguish "clear it" from "I meant to pass a name and the shell ate it"
- `--no-key` also clears the passphrase: Clearing only the key path would leave a secret on disk that nothing reads and nothing can reach: an orphan
- `vps edit --no-key` on a host whose only authentication was that key is refused with exit 64, because the result would authenticate nowhere
- `--no-agent-socket` fails closed on Unix: Removing the explicit socket from a host that authenticates through the agent leaves it with no endpoint the platform provides, so the edit is refused with exit 64 and a message naming the alternatives
- On Windows the default endpoint exists, so the same command is allowed
- The predicate lives beside `resolved_agent_socket` in `ssh::connection`, where the knowledge already was
- `--dry-run` now covers `vps edit` and `vps rename`: Both previews run every validation the real run runs, name parsing, duplicate detection, TLS pairing, record validation, and then stop before the write
- The `vps-rename` plan carries `previous_name` and `active_rewritten`, so a caller can see whether the marker would move without moving it
- `commands --json` publishes every argument: The tree carried `name`, `about` and `subcommands`, which told an agent that `vps edit` exists and nothing about what it accepts
- Each argument now appears with `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` and `hidden`, and each command carries its `aliases`
- Ten new flags would otherwise have shipped invisible on the one surface built for discovery
- `--lock-wait-ms` bounds the config lock: Default 5000 ms, ceiling 300000 ms, both from `crate::constants`
- See the `Fixed` entry on contention
- `use_agent` and `agent_socket` are visible on `vps show`, `vps list`, `vps export` and `vps import`: The wire types now carry the two fields, `print_details_text` prints the agent state, tags, key passphrase and all five TLS fields, and an import sets them before validating
- The `vps-renamed` event exists as its own envelope: A rename is not an edit that happened to change a key: the operator needs to know which name resolves now, and `vps-edited` alone would not say
- The envelope carries `name` (after), `previous_name` (before), `active_rewritten`, `updated` and the target echo (`target_resolved`, `target_source`)
- `vps edit <NAME> --name <NEW>` reaches the same code path and emits the same `vps-renamed` event, so a caller gets one answer whichever spelling it used
- `vps-edited` is emitted only when nothing was renamed and never carries `previous_name`
- `vps edit` and `vps rename` report `updated`, the fields whose stored value actually changed: A flag can be accepted and still leave the value identical, setting `--port 22` on a host already on 22, and without this field the envelope claimed a change that did not happen
- `updated` lists the fields in application order, each at most once, with `tags` last
- It is empty when every requested value already matched or when the invocation only renamed the host, and an invocation with no field, rename or clear flag is refused with exit 64
- The field is always present in the JSON, so a consumer tests for an empty list rather than for a missing key
- `vps-import` reports `replaced`: The success event lists the host names the import overwrote, sorted, and the field is always present, empty when nothing was overwritten
- The dry-run keeps `replaces_existing`, which is a prediction
- `replaced` is the record of what the real run did
- `docs/schemas/vps-edit.schema.json` and `docs/schemas/vps-rename.schema.json`: Both are in the `schema` catalog, and `dry-run.schema.json` now lists eight operations
- `vps doctor --json` reports the config directory and the bad names: `local` gains `config_dir_exposed` (boolean, always present), `invalid_host_names` (array of escaped strings, always present, empty when every name is valid) and `config_dir_permissions` (a four-digit octal string such as `0700`, absent off Unix or when the directory does not exist, and off Unix only this one is missing while `config_dir_exposed` reads `false`)
- The text report prints `Config dir mode:`, `Config dir exposed:` and `Invalid host names:`
- `permissions` still describes `config.toml` only
- `fs_perm::ensure_config_dir` is the one place that creates the config directory: A source scan in `gaps_v073_config_dir_mode` fails if another module creates it
- Four new suites keep the 0.5.6 fixes from coming back: `gaps_v073_vps_auth_switch`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` and `gaps_v073_config_dir_mode`
- `--help` shows the exit codes and examples: The root `--help` ends with an `Exit status:` section for 0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 and 143
- All 48 leaves carry an `Examples:` section through `after_long_help`, which `--help` shows and `-h` leaves out
- The help text lives in `src/cli/help_text.rs`
- The rustdoc says when a function fails: It gains 78 `# Errors` sections, one `# Panics` section and 7 doctests, so the doctests go from 25 to 32
- The six batch, listing and preview schemas gain the optional `agent_shape` and `count` fields: `exec-batch`, `scp-batch`, `sftp-batch`, `health-check-batch`, `sftp-list` and `dry-run` now describe the shaped output that the global flags produce
- `vps-rename.schema.json` lists the 19 field names that `updated` may carry

### Changed
- BREAKING for builds, MSRV is now Rust 1.89, because `russh` 0.65 declares `rust-version = "1.89"`, while `keyring` 4 alone needed 1.88
- `scripts/check_all_gates.py` runs fourteen gates, and the new `doc` gate runs `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` right after `clippy`
- `scripts/check_advisory_freshness.py` declares its 7-day ceiling as an operational exception, with the reason written beside it
- `skills/ssh-cli-pt/SKILL.md` and the copy kept in `~/Dropbox/ai/ai-llms/skills-nao-usadas/ssh-cli/SKILL.md` are byte-identical again: 63 lines that existed only in the installed copy were measured against the binary and merged, and the one rule that taught the behaviour from before this release was replaced by what `vps show a1 --select name,host` actually returns
- MSRV was raised from 1.85.0 for `keyring` 4.2.0, which declares `edition = "2024"` and `rust-version = "1.88.0"` and the zbus store follows
- The E2E matrix is E01 to E21, up from E01 to E18
- `docs/CROSS_PLATFORM.md`, `docs/AGENTS.md`, `docs/COOKBOOK.md` and `INTEGRATIONS.md`, in both languages, now name all 89 published flags, where they named 39, 73, 85 and 68 before
- A3: a reduced bare array reports its count on stderr as one JSON line with `"event":"agent-shape"`, carrying `input_count`, `output_count` and `dropped`
- Before this the count went only to `tracing::info!`, which the default `error` level discards, so `ssh-cli vps list --limit 2 --json` over 60 hosts returned two records and said nothing about the other 58 while the module promised "truncation is never silent"
- stdout keeps the array's type, because consumers index it, and `--quiet` suppresses the stderr line like every other diagnostic
- A5: the `clippy` gate now passes `-W clippy::doc_markdown`, so a flag doc that loses its backticks fails the battery instead of scrolling past
- `vps edit` and `vps rename` publish target provenance: Both call `set_resolved_target` after the registry lookup succeeds and before any mutation, and both echo `target_resolved`, `target_source: argv` plus the `host_*` read aliases
- Publishing before the lookup would have inverted the existing contract: `vps_not_found` must keep reporting no target at all, because no host was chosen
- The `vps edit` arm in `crud.rs` is now a builder call: The 186-line block that read stdin, took the lock, mutated the record, validated TLS and emitted the envelope moved to `vps::edit`, where `EditRequest` names every input and `apply_fields` is pure apart from the key-path existence check
- `cli/mod.rs` sheds the dry-run policy and the command tree: Both moved to `cli/dry_run.rs` and `cli/discovery.rs`
- The file sits on a frozen component budget, and a budget file that only grows stops meaning anything
- BREAKING for library users, `SshCliError::Unavailable` carries an enum, not a string: The variant is now `Unavailable { service: UnavailableService }` and the constructor is `SshCliError::unavailable(UnavailableService)`
- `UnavailableService` is `#[non_exhaustive]` with `Keyring` and `ConfigLock`
- A free-form string let the keyring and the config lock share one variant while meaning different things, and it let the 0.5.6 lock failure ship with a hint about the keyring
- The enum makes each caller name its own cause and lets the hint follow it
- The wire text is unchanged (`service unavailable: config_lock`), so CLI and JSON consumers see nothing
- Migration: replace the string argument with the enum value and add a wildcard arm where the variant is matched
- BREAKING for release wrappers and local gate callers, the nine `scripts/*.sh` are now `scripts/*.py`: `check_all_gates`, `check_advisory_freshness`, `check_cross_targets`, `check_en_identifiers`, `verify_install_resolve`, `generate_sbom`, `dist_multiarch`, `release_attest` and `e2e_real_ssh` became Python 3.9+ programs that use only the standard library, with the helpers `_gatekit.py`, `_e2e_sshd.py` and `_e2e_cases.py` beside them
- On Windows they run as `py -3 scripts/<name>.py`
- A shell script runs on Linux and macOS and nowhere else, and this project ships a Windows target
- Their environment variables became flags: `CHECK_TIMEOUT_SECS` is `--timeout-secs` (also accepted by the scripts that call cargo), `TARGETS`, `OUT_DIR` and `PARALLEL_JOBS` are `--targets`, `--out-dir` and `--jobs`, `REQUIRE_COSIGN` is `--require-cosign`, `CARGO_TARGET_DIR_INSTALL_GATE` is `--target-dir`, and `check_advisory_freshness` takes `--db-root`
- `e2e_real_ssh` takes `--host --port --user --password-file --sudo-password-file --from-grok-config [PATH] --bin --local-sshd` and no longer reads `SSH_CLI_E2E_*` or `--config-dir`
- `--from-grok-config` needs Python 3.11+, and `--local-sshd` is Unix-only and fails on Windows with `local_sshd=requested_but_unusable`
- Migration: `bash scripts/<name>.sh` becomes `python3 scripts/<name>.py`
- The old `.sh` files are deleted (see `Removed`)
- `--version` provenance reads a stamped file, and `build.rs` stops rerunning: `.commit_hash` now holds `<short hash> <version>`, and the resolution order is env, then git, then the file, then `unknown`
- Without env and git the file is exact only when its version is the package version, so a hash left over from an earlier release can no longer be reported as the current one
- A file recorded for another version is printed as `<hash>-dirty`, because the version bump proves the tree changed after that commit
- `build.rs` no longer reruns on every build when there is no `.git`
- Until the release refresh step in `docs/RELEASE_CHECKLIST.md` runs, 0.5.6 prints `08462e2-dirty`: the base commit of this tree plus the local changes, because the release commit does not exist yet
- BREAKING for scripts and agents, `vps edit` switches the authentication method: `--key` clears `password`, `use_agent` and `agent_socket`
- `--password` and `--password-stdin` clear `key_path`, `key_passphrase`, `use_agent` and `agent_socket`
- `--use-agent` clears `password`, `key_path` and `key_passphrase`
- Before, the same calls exited 64 with `primary auth methods are mutually exclusive`, and the only way from a password to a key was to go through `--use-agent` first
- Code that relied on that refusal now gets a changed record and exit 0
- `updated` lists what an auth switch cleared: Only fields that held a value appear, after the new method, so `vps edit prod --key K` on a password host reports `key_path` and `password`
- `--use-agent` follows the same rule
- A different `--key` drops the old passphrase: When the call carries no new passphrase, `key_passphrase` is cleared and listed in `updated` if it existed
- The same key path changes nothing
- `--agent-socket` alone implies `--use-agent`: It clears `password`, `key_path` and `key_passphrase`, and `updated` lists the ones that held a value
- A passphrase needs a private key: `--key-passphrase` or `--key-passphrase-stdin` on a host that ends with no key exits 64 (`a key passphrase needs a private key`) and leaves the record byte-identical
- Conflicting auth flags in one call exit 2 at parse time: `--key` with `--password*`, `--agent-socket` with `--key`, `--password*` or a passphrase flag, and a passphrase flag with `--use-agent` or `--password*` are rejected by clap before any work
- They used to reach the validator and exit 64 later
- `vps edit X --use-agent --agent-socket S` stays valid, and `vps add` is unchanged
- BREAKING for hosts you create, `VpsName::try_new` is strict: `vps add`, the destination of `vps rename` and `vps edit --name`, and `vps import` refuse names outside `[A-Za-z0-9._-]` (see `Security`), where spaces, accents and control characters used to pass
- Reading stays lenient through `VpsName::from_stored`: NFC, not empty, at most 255 bytes, no control character (`char::is_control`) and none of the formatting or bidi characters U+00AD, U+061C, U+180E, U+200B to U+200F, U+202A to U+202E, U+2060 to U+2064, U+2066 to U+206F, U+FEFF and U+E0000 to U+E007F (U+2065 is unassigned and stays out)
- A legacy host named with a space or an accent still loads, lists, shows, resolves and can be renamed to a strict name
- `vps import` lists every bad name at once: It validates all items first and exits 64 with `invalid VPS names in import (N): 'a', 'b'; a name is 1..=255 ASCII letters, digits, '.', '_' or '-' and starts with a letter or digit; nothing was imported`
- Nothing is written, and `--dry-run` gives the same error
- A legacy host whose name fails the strict charset can be exported, but its export restores only after `vps rename` gives it a strict name
- A legacy `config.toml` never blocks the CLI: A stored name with control, formatting or bidi characters loads
- `doctor` lists it escaped in `local.invalid_host_names`, `list`, `show`, `remove` and `edit` escape it in text mode, and `vps rename '<legacy name>' <new>` repairs it because the source is not validated
- Lookups that become a target refuse control characters with exit 64: `exec`, `health-check`, `--hosts` and `connect` reject such a name as a usage error, and `connect` writes no marker
- `show`, `remove`, `rename` and `edit` on a source that does not exist still exit 66
- `vps edit`, `vps rename` and `vps remove` echo the target in `--dry-run`: The plan carries `target_resolved`, `target_source` (`argv`), `host_resolved` and `host_source` after the lookup succeeds, and `dry-run.schema.json` lists them
- A failed lookup still carries no target
- A missing registry exits 66 and creates nothing: `vps edit`, `vps rename` and `vps remove`, real or `--dry-run`, on a config that does not exist create no directory and no lock file
- The config map is keyed by the NFC record name: Two tables that normalize to the same name exit 65 with `config has two hosts named ...` (names escaped), and nothing is lost silently
- `--config-dir something.toml` makes the parent the config directory for everything: `secrets.key`, `known_hosts`, `active`, `lang` and the TLS files follow `config.toml` there
- `vps export -o new/deep/e.json` creates the missing directories `0700`: An existing directory is never changed
- BREAKING for library users, `paths::validate_and_normalize` returns `String`: It returned `VpsName`
- Only the file-name rule remains in that function (the mTLS path did not change), and the host-name rule lives in `VpsName::try_new`
- Migration: take the `String` and call `VpsName::try_new` where a host name is meant
- The `test` gate runs `cargo test --locked --all-features --no-fail-fast`: One red test binary no longer hides the others (`scripts/check_all_gates.py`)
- Tests use temp directories and three hosts: `apply_tests.rs` uses `tempfile`, `gap_doc_003` reads the schema with `serde_json`, the `vps_not_found` snapshot was re-recorded, and the import round trip covers three hosts (one with a password, one with a key, one with an agent)

### Removed
- `serde_with` is gone from the dependencies because nothing used it, and 14 crates left `Cargo.lock` with this round of updates
- `package.homepage` is gone from `Cargo.toml`, because cargo 1.100 reported it as redundant
- No hosted-runner config in tree, stated without naming the vendor
- BREAKING for library users, `Money`, `Brl`, `Usd`, `Currency` and the `rust_decimal` dependency are gone: `src/domain/money.rs` no longer exists
- The product never handled money, so nothing in the CLI or the JSON changes
- Migration: delete the imports, or copy the module into your own crate
- The nine `scripts/*.sh`, 1236 lines, are deleted: The `.py` programs of the same names are the only entry points (see `Changed`)
- A shell script runs on Linux and macOS and nowhere else, and this project ships a Windows target
- The stray file `json` in the crate root is deleted: It was a leftover output file
- The suites `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v062_cross_platform.rs` and `tests/gaps_v073_package_hygiene.rs` and the test module of `src/tls/provider.rs` are deleted by the owner's decision

### Fixed
- `scripts/check_advisory_hits.py` read `advisory.patched_versions`, while `cargo audit` publishes `versions.patched` at the entry level, so the gate said "no patched release" for advisories that had one
- `use_keyring_is_wired_and_fails_closed` in `tests/gaps_v074_flag_reach.rs` and the E2E case E21 in `scripts/_e2e_cases.py` left one `secrets-primary-key:<digest>` entry in the OS store on every run
- The test now cleans up with `set_config_dir` plus `delete_key_from_keyring`, and E21 runs `secrets init --force` back to the file, which deletes the entry through `src/secrets/init_key.rs`
- The 7 orphan entries those runs had left in the OS store were deleted
- `impl Display for SshPort` used `write!` and ignored the width, so the PORT column of `vps list` in text mode was misaligned, and it now delegates to `u16`
- `vps list` in text mode sizes every column from its widest cell instead of the fixed 20/30/6/15 widths, which a 17-character user name pushed out of line, and no line ends in spaces
- `vps show` in text mode printed `Name:`, `Port:`, `User:` and `Added at:` with one extra space, and all 22 labels now align
- cargo 1.100 reported `human-panic` as unused in debug builds, because its `cfg(not(debug_assertions))` gate was redundant: human-panic 2.0.8 `src/panic.rs:60` is already a no-op in debug, so the gate is gone
- Gap 6 proved its own count with a hand-written `43` that had aged into `53`, so the proof now derives the number instead of asserting it
- Gap 31's proof claimed zero forbidden terms while `CHANGELOG.md` and `CHANGELOG.pt-BR.md` held four, and the two files were outside the scan, but the terms are gone and the proof now covers them
- Gap 49 recorded a `sha256` from a different source state, so the proof now derives the hash from the installed artefact
- `docs/COOKBOOK.md` still called an unpublished intermediate number the current product line, and the build that produced its outputs is 0.5.6, so the line and the version claim now say what they mean
- `CHANGELOG.md`, `CHANGELOG.pt-BR.md` and `skills/ssh-cli-pt/SKILL.md` carried three `h2-spacing` hits that the new gate caught on the first run, because the release sections sat one blank line apart
- The declared limits in `gaps.md` contradicted Gap 39 about the tunnel signal exits, and both now state the measured 130 and 143
- Gaps 39 and 42 recorded the E2E matrix as E01 to E18 while it has run E01 to E21 since the matrix grew
- `.commit_hash` was frozen at `a691fc7 0.5.5`, and the release refresh step in `docs/RELEASE_CHECKLIST.md` stamps it with the release it belongs to
- `vps list --select name --json` returned `[{"name":"a"}]`, an item that satisfied neither shape of the published list schema because the reduction report of a bare array goes to stderr and never becomes an item field
- The schema conformance suite validated only the unshaped list, which is why the deviation was invisible
- It now validates both forms
- `a_document_without_the_inventory_points_at_one` accepted any mention of `HOW_TO_USE`, `README.md`, `llms-full.txt` or `commands --json` as a substitute for the flag inventory, which is how the four documents above shipped incomplete and green
- Eight flag tests asserted only that the parser did not reject the flag: `--fail-fast`, `--use-keyring`, `--secrets-key-file`, `--replace-host-key`, `--scp-file-concurrency`, `--description`, `--contact` with `--staging`, and `--keyring` on `secrets init`
- `--description` with a line break could end the shell comment and make the rest of the text a second remote command
- The sanitization of line breaks is now asserted directly in `src/ssh/packing.rs`
- `--scp-file-concurrency` installed its value once through a `OnceLock`, so the value the batch window read could not be measured
- It is an atomic now, clamped at the installer
- `ssh-cli --use-keyring secrets init --json` answered `key_source: xdg_file`, because the subcommand received only the leaf `--keyring` and the global flag never reached the write path
- The init path now reads the runtime state that the dispatcher already installs, so both flags reach the same store
- A single-record payload that ignored a collection flag now names it under `agent_shape.ignored` instead of accepting it in silence, including when that flag is the only one passed: `ssh-cli doctor --limit 2` used to return the body byte identical to an unshaped run because the early return fired whenever neither `--select` nor `--truncate-content` was present
- `vps-show` and `vps-doctor` match their two shapes with `anyOf` rather than `oneOf`: `--truncate-content` and the collection-only flags keep the record whole, so both variants described the same body and a `oneOf` would have rejected the real output
- `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` and `--max-output-bytes` remain inert on a single record on purpose, because there is no collection to reduce, and the report says so per flag
- Agent authentication no longer disappears through export and import: `MaskedVpsJson`, `ExportHostJson` and `ImportHostEntry` had no `use_agent` or `agent_socket`, and `into_record` never set them, so a host that authenticated through the agent round-tripped into a host with no password, no key and no agent, one that could not connect at all
- Measured on the wire before the fix: the export of an agent host carried no trace of the agent
- The config lock no longer hangs: `flock` blocks indefinitely by default, so a peer holding `config.toml.lock` made every other one-shot wait forever: no output, no error, no exit
- It now retries with full-jitter backoff until `--lock-wait-ms` expires and then returns `Unavailable` (exit 69, retryable)
- A signal received during the wait wins over the deadline and exits 130/143, because the operator asked to stop and a retryable code would invite a retry
- `vps import` takes the lock before it loads: It loaded the registry unlocked and saved under a lock, so an import committing between a peer's load and save discarded the peer's host
- The guard now spans load → merge → save, the same shape the rest of the CRUD follows
- `--dry-run` on a missing host still exits 66: The precondition outranks the preview in both new verbs, so a rehearsal never reports a plan for an operation that would fail
- An edit that changes nothing is an idempotent no-op: It exits 0, emits `vps-edited` with `updated: []` and does not rewrite `config.toml`, so it spends no temp file, no fsync and no rename
- Exit 64 is reserved for an invocation that carries no field, rename or clear flag at all, which is the only case where the caller asked for nothing
- An agent that repeats an edit after a lost response now gets the same success instead of an error that looks like a failure
- Rename preconditions have a fixed order: `vps rename ghost ghost` exits 66, because "the host exists" is checked before "the names differ"
- An identical name for a host that does exist exits 64 and the error carries `target_resolved`, since by then a host was chosen
- Rename cannot leave the marker pointing at a name that no longer exists: The `active` marker is written first and `config.toml` second
- If the save fails the old marker is restored, and if the marker write fails nothing is renamed
- A crash between the two steps leaves an orphan marker, which is fail-closed: `--use-active` exits 66 instead of acting on the wrong host, and `vps list` shows the real state
- The config lock reacts to a signal within 100 ms, is created `0600`, and its hint no longer misleads: The wait runs in 100 ms slices that check the signal, so SIGINT and SIGTERM exit 130 and 143 within one slice
- Before, a blocked sleep could hold them for about 5 s. `config.toml.lock` is created with mode 0600 and the marker write fsyncs its directory
- The exit 69 `unavailable` hint now says to repeat the argv or raise `--lock-wait-ms` and no longer mentions the keyring, which has nothing to do with this failure
- `VpsNotFound` (exit 66) says that an earlier rename or remove may have worked: The hint tells the caller to check `vps list`
- `vps rename` is not idempotent: repeating it after a success exits 66, because the old name is gone, and `vps show <NEW>` is how to confirm the result
- Text mode no longer swallows a stdout write error: Every verb now propagates it: a broken pipe exits 141 and a full disk becomes an I/O error, where before both exited 0 after the output was lost
- On `exec` with stdout closed the write error wins over the remote command failure, because the caller cannot read the result of a command whose output it could not receive
- `vps import` validates TLS like `vps edit`: A certificate without a key exits 64 at import time instead of producing a record that fails later at connect time
- The two paths shared a rule and only one enforced it
- A host could not move from a password to a key: `vps edit --key` on a password host, and `--password` on a key host, exited 64 as mutually exclusive
- The edit now switches the method and clears the old secrets (see `Changed`)
- A host name with control characters could be created and printed raw: The strict charset refuses it at creation, and text mode escapes whatever is already stored
- ESC and BEL reached the terminal in text mode: Messages, errors, `list`, `show`, `health-check`, `doctor` and the batch outputs now escape them
- The config directory was created `0755`: It is `0700` now, and `doctor` reports an existing exposed directory
- `locale set --config-dir new/config.toml` created a directory named `config.toml`: When the path ends in a missing `.toml` file, the parent is the directory that gets created
- An encrypted secret behind a `.toml` `--config-dir` exited 74: `secrets.key` was written to a directory named after the file, and the run left that directory behind
- The parent is the directory now
- `known_hosts.lock` and `lang` were born with the umask: Both are created `0600`, with no window
- Red tests are green: Five tests were out of date with the code: `snapshot_error_vps_not_found` (old hint), `gap_doc_003_residual_behaviors_documentados` (sliced schema text), `every_suite_on_disk_is_catalogued_in_testing` (`docs/TESTING.md` lacked five suites), and `json_export_then_import_round_trips_every_field` plus `toml_export_then_import_round_trips_every_field` (edited a password host with `--key`)
- `clippy::manual_find` in `docs_conformance` is fixed, and `cargo fmt --check` is clean again
- `apply_tests.rs` no longer writes to a fixed path in `/tmp`: Parallel runs could collide on it
- It uses `tempfile`
- `cargo audit` no longer reports RUSTSEC-2026-0235: `rkyv` entered the lock only through the optional `rust_decimal`, and both left with it
- `docs/TESTING.md` and `docs/TESTING.pt-BR.md` list every suite on disk: They now catalog the five `gaps_v072_*` suites that were missing and the four new `gaps_v073_*` suites, and `every_suite_on_disk_is_catalogued_in_testing` fails when a suite is left out
- The two repository skills regained four safety rules that the 4000-word cut had removed: The SANITIZE step, the raw `sudo` rule, the TOFU rule and the persistent form of `--disable-sudo` are back, paid for by cutting duplicated formulas
- The installed copy of the agent skill was stale: It was an old copy with 47 leaves and did not know `vps rename`, `vps edit --name`, `--lock-wait-ms`, the clear flags or the auth switch, so it was replaced by the current text with 48 leaves
- `cargo clippy --lib --no-default-features -- -D warnings` passes again: A `needless_return` in `src/cli/dispatch.rs` was removed
- Documents that taught the wrong behavior were corrected after measuring the binary: The ACME domain is `--domain <DOMAIN>` and never a positional, so `tls acme issue example.com` exits 2
- A redacted `vps export` emits an empty `password` and null for the other secrets: The three-asterisk mask appears only in `vps list` and `vps show`
- The results of `health-check --all` carry `status` with `ok` or `error`, so the filter is `--filter status!=ok`
- Other corrected claims: `rustls` is allowed on purpose in the release checklist, macOS arm64 is not covered by any gate, and the texts about parsing only stdout, `--count-only` in batches, the shape of `exec --step`, `--no-input` with `--*-stdin` and the scope of `--disable-sudo` now match the binary
- Code comments that described a mask in the export were corrected
- Published documents no longer carry a stale gap count: They state which classes of gap the release closes, and every gap of this release is resolved except the pure Rust cryptography attempt, which was tried and reverted
- Published documents no longer point to the unpublished `gaps.md`: `llms.txt`, `llms-full.txt`, `llms.pt-BR.txt` and both INTEGRATIONS stopped linking or citing it
- Usage errors are documented as plain text: The exit 2 of the argument parser and the exit 64 of a malformed `--filter` print plain text on stderr even with `--json`, and only execution errors carry the envelope
- `vps-export.schema.json` describes the real output of `vps export -o`: A `oneOf` accepts the body (`schema_version` and `hosts`) or the summary (`path` and `format`)
- The changelog history is aligned between the two languages: The English header pasted inside the Portuguese 0.5.2 section and an empty section are gone, titles are in the language of each file, and every version has the same number of bullets in both files, with no fact of a published entry changed
- Em dashes outside H3 titles are gone from the published documents: They became colons or commas with the words unchanged, and the formatting linter now counts them
- `vps import --file` on a missing file exits 66 `file_not_found` as a permanent error: It used to exit 74 `io` with `retryable: true`, which sent an agent into a retry loop for a path that cannot appear
- Fourteen schemas were realigned with the output the binary emits: `dry-run`, `error-envelope`, `exec`, `sudo-exec`, `su-exec`, `exec-batch`, `vps-edit`, `vps-rename`, `secrets-init`, `secrets-reencrypt`, `vps-doctor`, `sftp-fs-op`, `tunnel-closed` and `health-check-batch`
- `vps list` in text mode shows `(not set)` in the PASSWORD column of a key-only host: It used to print a masked value for a password the host does not have
- The command inventories in `docs/` quote the full invocation: HOW_TO_USE, AGENTS, COOKBOOK and CROSS_PLATFORM listed 28, 19, 20 and 43 leaves as a bare `vps add`, and the coverage test now requires `ssh-cli vps add` in a code span
- Documented behaviour that the binary contradicted was corrected: a global `--disable-sudo` also persists before `vps add` and `vps edit`, a `tunnel` stopped by a signal exits 130 or 143, only the two secret `SSH_CLI_*` variables are refused, exit 69 also covers a registry lock, and the written prefix is `sshcli-enc:v2:`
- A secret-key environment variable is refused as a usage error on every command that reads or writes a stored secret: `vps list`, `vps show`, `vps export`, `doctor` and `vps add` with `SSH_CLI_SECRETS_KEY` or `SSH_CLI_SECRETS_KEY_FILE` exited 65 `config` with a message that blamed the TOML, because serde flattened the typed error and the loader wrapped it again

### Internal
- `Cargo.toml` declares `keyring = { version = "4.2.0", features = ["v1"] }` instead of relying on a default that does not exist, and the MSRV comment that still said 1.85 was corrected
- `tests/gaps_v074_flag_reach.rs` replaces the pointer test with a count: eight documents in two languages are read against the 89 flags `commands --json` publishes, and a shortfall names the missing flags per file
- `tests/gaps_v063_e2e_harness.rs` fixes the matrix at 21 cases, and `scripts/_e2e_cases.py` schedules all three new ones
- The keyring tests clean up through `delete_key_from_keyring`, and the one test that still leaked an entry is listed under `Fixed`
- The stale `.commit_hash` (`a691fc7 0.5.5`) is declared instead of updated, because there is no git metadata in this directory and running git was not requested
- `--version` therefore shows the base commit with `-dirty`, not a release hash
- `src/agent_shape.rs` split its single-record paths into `src/agent_shape_record.rs`, so both files stay under the 600-line component budget
- `tests/gaps_v074_flag_reach.rs` adds 21 assertions over the 18 flags that had no behavioural test and over the documents that claim the full inventory
- `tests/docs_conformance.rs` adds the Examples coverage and parse test for the 48 leaves
- The schema conformance validator now enforces `pattern`, a keyword five published schemas already carried unchecked, and `dev-dependencies` gains the `regex` crate for it
- New direct dependency `rustix` (features `process` and `std`, Unix only): It was already in the lock through `tempfile`
- `src/output/sanitize.rs` and `src/vps/edit/auth.rs` are new modules: The first owns text escaping, the second owns the auth switch and its `updated` list
- `gaps.md` is rewritten as the record of the gaps of 0.5.6: Each gap has problem, consequences, root cause, solution, benefits and proof, and the file stays local because it is not published
- `Cargo.toml` `exclude` now lists `/json`, `*.sh`, `__pycache__/`, `*.pyc` and `*.snap.new`, and `cargo package --list` shows none of them
- Files were split by responsibility: `src/cli/help_text.rs`, `src/cli/tunnel_mode.rs`, `src/cli/process_flags.rs`, `src/cli/locale_cmd.rs`, `src/i18n/error_text.rs`, `src/secrets/tests.rs`, `src/vps/model/tests.rs` and `src/vps/config_io/tests.rs` are new
- The size ledger shrank: `secrets.rs`, `cli/mod.rs`, `vps/model.rs` and `cli/dispatch.rs` now measure under the 600-line budget and left the ledger
- `FULL_INVENTORY_DOCS` in `tests/docs_conformance.rs` also lists `llms.txt`, `llms.pt-BR.txt`, both INTEGRATIONS and both COOKBOOK: `every_command_appears_in_every_full_inventory_document` now holds the 48 leaves in 15 files, and it found `sftp download` missing from the COOKBOOK
- A migration to pure Rust cryptography was tried and reverted: the TLS provider stays `aws_lc_rs`, and `aws-lc-sys` still needs a C compiler at build time (the attempt and its measured reason are kept in the local audit record)


## [0.5.5] - 2026-08-10

### Security
- Translated text no longer reaches the JSON wire: Nineteen call sites in SCP and SFTP built a failure out of `i18n::t(…)`, either as the body of `SshCliError::InvalidArgument` or straight into the `error` field of a batch entry, so under `--lang pt-BR` the envelope carried Portuguese
- `docs/schemas/error-envelope.schema.json` states that `message` is "always English, regardless of `--lang`", because an agent branching on prose that moves with the operator's locale has no stable discriminator at all
- Two of those sites wrote to stdout: a cancelled `scp --all` put `Operação cancelada pelo usuário.` in the batch payload
- The three phrases now live in `crate::constants` as plain `&str`, and `Message::OperationCancelled`, `ScpUploadFileOnly` and `ScpDownloadLocalNotDirectory` were removed: a `Message` variant is a promise the string is safe to localize, and for these three it never was
- The human-facing label around each body is still translated
- The existing gate checked the emitter, which did nothing wrong
- The new one checks the constructors, because an invariant that can be broken before the checkpoint has to be tested at the source
- BREAKING, `ssh_cli::telemetry` is now `ssh_cli::tracing_setup`: The module never emitted telemetry, its own documentation has ruled out OTLP, metrics backends and log files since the first commit, but a reader auditing the tree against this project's telemetry ban found a module by that name and had to read it to learn it was innocent
- A name that costs every reader a verification is wrong however defensible its history
- No deprecated alias is provided: an alias would preserve the name the rename exists to remove
- The wire field `"telemetry": false` in `vps doctor` keeps its name, where the word is the claim being denied
- BREAKING, `health-check` now demands an explicit target: It used to probe the active marker whenever the name was omitted, defended as safe because a connectivity probe is an idempotent read
- The defence is true about the probe and false about the habit: an operator who learns a target is optional on the read surface carries that expectation to `exec`, which is where GAP-SSH-EXEC-ARGC-001 cost a day of work
- Ergonomic asymmetry between a read and a write is a trap, and the read is the cheaper half to make strict
- The exit taxonomy is now identical across both families: 64 means no target was designated, 66 means the designated target does not exist
- This narrows the older `GAP-SSH-EXIT-002` contract, which mapped a nameless `health-check` to 66 and thereby conflated "you named nothing" with "what you named is absent"
- Migration: `ssh-cli health-check` becomes `ssh-cli health-check --use-active` or, preferably, `ssh-cli health-check <VPS>`
- GAP-SSH-EXEC-ARGC-001 rule 6, exit 127 on step zero now aborts the batch: That code on the first step is the fingerprint of a host name having been promoted to a command: the shell answers `command not found` and, before this, every remaining `--step` then ran against a machine nobody designated while the aggregate exit stayed 0. The abort is scoped to index zero
- A 127 at any later index keeps the existing run-everything contract, because by then the session has already proven it is aimed where the caller believes
- Defence in depth: the argv that produced this is now rejected at parse time, and this is the second lock on the same door
- BREAKING, GAP-SSH-EXEC-ARGC-001: the exec family could run on a host the caller never named: `exec`, `sudo-exec` and `su-exec` declared `num_args = 1..=2` on a single positional vector and used the count to decide meaning: two positionals were `VPS COMMAND`, one was `COMMAND` against whatever host the on-disk `active` marker held
- `--step` is `ArgAction::Append` and therefore never counted as a positional, so `exec <HOST> --step <CMD>` collapsed into the one-positional branch: the host name was executed as a remote binary on a different machine, step zero returned 127, and every remaining `--step` ran there while the aggregate process exit stayed 0. On 2026-08-09 that put a full auto-clean configuration, systemd units and timers included, onto the wrong server
- The intended host was never touched and the operator was told it succeeded
- The class is CWE-441 (Unintended Proxy or Intermediary, "Confused Deputy"): the CLI held authority the caller could not see it exercising, and spent it on a target drawn from ambient state instead of from the request
- Naming the class matters because the fix generalizes from it: every surface that can act on a host must take that host from the argv, never from the environment
- `clap`'s own `Arg::num_args` documentation warns about precisely this shape: a positional with a range keeps consuming values "until it finds another flag or option"
- Arity no longer carries meaning
- The mode is declared by a flag: `<VPS> <COMMAND>` for one host, a selector plus one positional for a fleet, and the new `--use-active <COMMAND>` to reach the marker deliberately
- A lone positional with no flag is now a usage error (exit 64): it used to be the dangerous shortcut, and there is no argv left that reaches a host the caller did not designate
- A second guard rejects `--use-active <registered-name>`, which is the same mistake one flag further along
- The fleet path already failed closed
- This brings the single-host path to parity
- Migration: `ssh-cli exec 'uptime'` becomes `ssh-cli exec --use-active 'uptime'` or, preferably, `ssh-cli exec <VPS> 'uptime'`
- A1: `russh` moved from 0.62.2 to 0.62.5
- CVE-2026-68930 (GHSA-m65r-rprj-r5rg, published 2026-08-03) affects `<= 0.62.4`: channel-scoped handler callbacks are dispatched for recipient channel IDs that were never opened
- The defect is server-side and this CLI is a pure client: there is not one reference to `russh::server` in `src/`, so the reachable impact here was nil
- The pin moved anyway, because `cargo deny check advisories` reported zero findings with the advisory database synchronised the same day
- A green supply-chain gate is not evidence that no published CVE applies
- A3: The at-rest crypto stack is no longer duplicated
- `chacha20poly1305` 0.10 pulled `aead` 0.5 / `chacha20` 0.9 / `poly1305` 0.8 while russh and aws-lc-rs resolved the 0.6 / 0.10 / 0.9 generation, so two independent ChaCha20-Poly1305 implementations were compiled into a security binary
- Bumping to `chacha20poly1305` 0.11, `getrandom` 0.3, `toml` 1 and `windows-sys` 0.61 collapsed 41 duplicate crate versions to 5, all transitive and individually justified in `deny.toml`
- `deny.toml` `multiple-versions` moved from `warn` to `deny`
- The warning was load-bearing and nobody read it
- Every surviving duplicate is now enumerated with a reason, so a new one: the kind that is actually fixable, fails the gate
- A6: `tls mtls import` read operator-supplied PEM paths with an unbounded `fs::read`
- Both reads are now capped at `MAX_PEM_FILE_BYTES` (1 MiB) via the new `paths::read_bytes_capped`

### Added
- GAP-SSH-EXEC-ENVELOPE-002 completed, the error envelope names the host too: The 0.5.5 work added the audit fields to the success envelope
- The error path, where the information matters most, did not carry them
- Measured: a failure reached through argv and one reached through the marker produced byte-identical envelopes, so the misdirection stayed invisible in the one channel an agent actually reads
- `target_resolved`, `target_source` and `active_fallback`: plus their `host_*` read aliases, now appear on the error envelope as well, and are absent rather than empty when the failure predates target resolution, `vps_not_found` is that case
- An empty string would assert that a host was chosen and has no name
- Absence says no host was chosen at all, and only one of those is true
- GAP-SSH-EXEC-ENVELOPE-002, the single-host envelope now names its own target: `ExecutionJson` carried `stdout`, `stderr`, `exit_code`, the truncation flags and `duration_ms`, and nothing that identified the host
- `ExecHostJson` and `ExecBatchJson`, the fleet path, always carried `name`, so the asymmetry ran opposite to the risk: the one path that accepted a silent fallback was the one that could not be audited from stdout
- Detecting a misdirected run required spending an extra SSH round-trip on a `hostname -f` badge command, which is optional and therefore skipped exactly when there is time pressure
- The envelope now carries `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) and `active_fallback`, with `host_resolved` and `host_source` alongside as read aliases: on the success path always, and on the error path whenever the failure happened after the host was resolved, so a misdirected run is a string comparison rather than an investigation
- All three are additive with serde defaults, so events written by earlier versions still deserialize
- `health-check` reports `host_source` for the same reason: naming a host without naming its provenance cannot distinguish a deliberate check from an inherited one
- That surface was defended here as a legitimate place to inherit the marker, because a probe is an idempotent read
- The BREAKING entry above revokes that defence within this same release, and the field is what makes the new opt-in checkable rather than merely claimed
- GAP-SSH-DOCTOR-ACTIVE, `doctor` reports the active host, not just the path to it: `local.active_file` gave the marker's path and no command in the CLI exposed its contents, so an agent had to read the file itself, outside the tool contract, to answer which host an ambient-target command would hit
- `local.active_vps` now carries the name, or `null` when no marker is set

### Changed
- Usage lines stopped teaching an argv the parser rejects: `clap` generated `Usage: ssh-cli exec --use-active <VPS> [COMMAND]...`, advertising a two-positional shape that `--use-active` forbids, printed at exactly the moment the caller had already got it wrong
- The positional stays a vector, because two named slots would bind `exec --all uptime` to a `VPS` slot
- The fix is an explicit `override_usage` on all three exec surfaces and on `health-check`, spelling the three legal forms
- The `su-exec` opt-in doc-comment also stopped describing `sudo`, which it had been copied from
- Commit provenance no longer freezes inside a checkout: `.commit_hash` is tracked in the repository, so it outranked git on every local build: the binary reported `f3b98b7` while `HEAD` was six commits later
- Git now wins whenever a checkout exists, and the file keeps its stated purpose exactly: a crates.io tarball ships without `.git`, so the git probe finds nothing there and the file answers, as designed
- BREAKING, G-ERR-R01: New exit codes 69 (`EX_UNAVAILABLE`) and 70 (`EX_SOFTWARE`), with matching `SshCliError::Unavailable` / `Software` variants
- `SshCliError::Config`: exit 65, classified permanent, had become the destination of every `map_err` without an obvious variant, 16 of them in `src/secrets.rs` alone
- An OS keyring that was locked exited 65 with `retryable: false`, so the one failure in that group a plain retry actually fixes was the one agents were told to abandon
- Keyring failures now exit 69 and are transient
- CSPRNG failures exit 70
- `secrets.key` I/O exits 74
- `Config` in that module dropped from 16 call sites to 3, all of them genuine data errors
- B2: error text is now localized in human mode
- The JSON envelope stays English by contract: 17 of 44 `i18n::Message` variants shipped with complete English and Brazilian Portuguese translations and had zero call sites, so `--lang pt-BR` produced byte-identical English for every failure
- Six were shadowed by `thiserror`'s `#[error("…")]` attribute literals, which render at the real emission site
- Eleven had been superseded by per-mode variants and never deleted
- Nothing catches this: `Message` is `pub`, so `dead_code` never fires on an unconstructed variant, and the exhaustive `match` in `en()` / `pt()` enforces a translation, never a caller
- The superseded variants are gone, the error variants now carry the upstream detail, and `i18n::localized_error_text` maps them by `error_code`: the same stable discriminator the schema publishes
- The `--json` branch deliberately keeps `SshCliError`'s English `Display`: agents branch on `error_code`, and a locale-dependent `message` would silently change the payload they parse when the host locale changes
- Untranslated codes fall back to English, so the localization is fail-open
- B3 / A7, component budget: `#[allow(clippy::too_many_arguments)]` appeared 16 times in production code, concentrated in exactly the files A7 flagged as oversized: the one lint that measures that coupling was silenced where it mattered, which is why `-D warnings` stayed green over it
- Removing all 16 showed only 10 still fired: six were dead attributes whose functions had already dropped below the threshold
- `crate::vps::AuthOverrides`, `HealthCheckRequest`, `tunnel::ServeContext` and `output::TunnelClosedInput` replace the positional lists
- 16 suppressions became 3, each with a comment justifying why the flat shape is correct
- `src/vps/exec_ops.rs` (748 lines) split into `exec_ops/{types,single,elevation,fleet}.rs` and `src/sftp/mod.rs` (630) into `sftp/{setup,dispatch,emit}.rs`, with every public path preserved by re-export
- `src/ssh/sftp_session.rs` stays whole by declared decision: it is one cohesive SFTP v3 state machine
- Three new suites (`gaps_v062_cross_platform`, `gaps_v062_i18n_reachability`, `gaps_v062_component_budget`) turn all of this into measurements that fail loudly
- G-SCP-R01 / G-SCP-R02: `scp-transfer` gained `mtime_preserved` and `durable`
- Both are additive and default to `true`, so events written by earlier versions still validate
- The product documented mtime preservation as a guarantee while discarding the failure at two nesting levels
- The parent-directory fsync after the atomic rename was invisible the same way
- Failures are now logged and reported instead of being resolved by silence: transfers onto filesystems that cannot represent a timestamp (FAT32, exFAT, WSL interop paths) still succeed, and say so
- C3, the component-size gate was calibrated to pass, not to a target: The first cut of `gaps_v062_component_budget` used a single 830-line ceiling, chosen so `src/ssh/sftp_session.rs` (810) would fit, and its doc comment named that one file as "a deliberate exception"
- The constant granted ten: nine other files sat between 601 and 810 and passed without ever being declared, while the test's own failure message told the reader not to raise the ceiling
- That is the B1 failure mode recurring inside B1's own remediation: a gate whose threshold is derived from the current measurement records the present and calls it conformance
- The budget is now a hard 600 plus a `DECLARED_EXCEPTIONS` ratchet: every file above it is named with a frozen cap and a mandatory reason, an entry may never grow, and an entry whose file has fallen to budget fails the suite so the ledger can only shrink
- The ratchet failed the very commit that introduced it, catching `src/i18n.rs` growing from 757 to 778 lines
- Rather than edit the number, the file was split: `en()` and `pt()` became `src/i18n/en.rs` and `src/i18n/pt.rs`, dropping it from 778 to 577 and off the ledger, with exhaustiveness unchanged because each `match` must still cover every variant
- C3b, that split disarmed three path-pinned gates, which are now contract-pinned: `gaps_v062_i18n_reachability` would have counted the relocated tables as production call sites, making every variant look reachable, green measuring nothing
- `gaps_v040_integration::gap_scp_020_i18n_mensagens` went red looking for a translated string in the wrong file
- `scripts/check_en_identifiers.sh` raised a false positive because its allowlist named only `src/i18n.rs`
- This is the fourth occurrence of the class after `gaps_v057_sftp` and `gaps_v061_error_taxonomy`: a source assertion tied to a path passes vacuously when code moves in and fails spuriously when code moves out, because it tests layout rather than contract
- The allowlist is now `src/i18n(\.rs|/)`, the integration test reads the concatenated subsystem, and the reachability suite gained `TRANSLATION_TABLES` plus a test that fails if an excluded path stops existing or the tables move back in
- Both gates were re-verified by deliberate falsification: an unused probe variant turned reachability red, and a probe file with a Portuguese literal exited the identifier gate with 1

### Fixed
- The published crate did not compile, and no local command could tell you: Renaming the target-resolution module to `src/cli/target/` collided with `.gitignore`'s bare `target/`
- A gitignore pattern without a leading slash matches at every depth, so git ignored a source directory, and `cargo package`: which lists files through git inside a checkout, dropped both of its files from the tarball
- `cargo build`, `cargo clippy`, 902 tests and all eleven gate scripts stayed green, because every one of them reads the disk
- Only `cargo publish --dry-run` failed, with `E0583: file not found for module `target``
- The ignore rule is now anchored to `/target/`, which is what it always meant: this crate is a single member and its build output lives at the root, and the module was renamed again, to `targeting`, so correctness no longer depends on one line of `.gitignore` staying right
- The localized `no_active_vps` text taught an argv the surface rejects: `health-check --use-active` reaches `SshCliError::NoActiveVps` too, and the message answered `run \`connect <NAME>\` or pass \`<VPS> <COMMAND>\``, but `health-check` takes no command, so a caller following the instruction got a second usage error
- The wording is now surface-agnostic in both languages
- This is the localization arriving after the surface changed: the `--use-active` opt-in gave an old message a second caller, and nothing pairs a message with the set of surfaces that can emit it
- Two doc comments went stale the same way and were corrected: `HealthCheckJson::host_source` and `HealthOptions::host_source` still explained why a nameless probe may inherit the marker, which the same release had made a usage error
- Justifications age worse than descriptions, because the behaviour they defend can be removed without the paragraph noticing
- C1, the English-message contract never reached the machine-readable artifact: B2 established that the `--json` envelope's `message` stays English regardless of `--lang`, and documented it in `docs/AGENTS.md` and `docs/schemas/README.md`, but not in `docs/schemas/error-envelope.schema.json`, whose `message` description was still the generic "human-readable error text"
- The one artifact an agent would generate a client from was the one that did not state the contract
- The description now declares the invariant, directs consumers to branch on `error_code`, and scopes localization to human text mode
- C2, half the error emitter was never localized: B2 routed every typed `SshCliError` through i18n and left the last branch of `resolve_exit_code` untouched: an `anyhow` chain that downcasts to neither `SshCliError` nor `DomainError` printed raw English under `--lang pt-BR`
- The failure a user is least equipped to interpret was the only one never translated, and it looked covered because the typed path: the tested one, was correct
- `localized_error_text` takes `&SshCliError` by signature, so that branch had no way to call it
- `Message::ErrorUnexpected` and `i18n::localized_unexpected_text` close it
- The helper returns `String` rather than `Option` because there is no alternative rendering to fail open to
- Only the label is translated: the `anyhow` chain is preserved verbatim so no diagnostic is lost, and the JSON envelope keeps `error_code` `"unexpected"`
- B1, the Windows target did not compile: `cargo check --target x86_64-pc-windows-msvc --no-default-features` failed with six errors while `fmt`, `clippy`, 818 tests, `deny` and `doc` were all green, and `docs/CROSS_PLATFORM.md` advertised Windows as supported
- Five errors came from `#![forbid(unsafe_code)]` on `src/platform/mod.rs`: an inner attribute on a module file also governs its children, the `windows` child is the product's only Win32 FFI surface, and `forbid`, unlike `deny`, cannot be lifted by an inner `#[allow]`
- The sixth was `windows-sys` 0.61 redefining `HANDLE` from an integer to `*mut c_void`, which broke a `handle == 0` guard
- The module now uses `deny`, `windows.rs` carries a file-scoped `allow` referencing the audited G-UNSAFE allowlist, and the guard is `handle.is_null()`
- The structural cause was that no gate cross-compiles: `#[cfg(target_os = ...)]` code for a foreign target is discarded before type-check, so it is invisible to every gate run on the host
- `scripts/check_cross_targets.sh` now type-checks `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and `x86_64-apple-darwin`, and is required by `CONTRIBUTING.md` and the release checklist
- B4: `SCP_IO_CHUNK = 32_768` was declared twice as a function-local `const` inside `src/ssh/client_real_scp.rs`, and `SCP_HEADER_MAX_BYTES` once in `src/ssh/scp_wire.rs`, each carrying a marker admitting it belonged in `crate::constants`
- The stated reason was an edit allowlist from a round that had already closed, so the justification expired while the duplication stayed: and `src/constants.rs` already hosted `SFTP_IO_CHUNK` with the identical value
- Both constants moved, and a gate fails if the marker reappears
- B5: `gaps.md` declared "OPEN residual: 0" in its header while eighteen sections still carried bold `OPEN` titles in the body
- All eighteen were verified closed in code across two audits
- Only the titles had never been updated
- Traceability is the single thing that file delivers, so the divergence is a product defect rather than a formatting one
- A2: `docs/schemas/error-envelope.schema.json` enumerated `error_class` as `transient|permanent|cancelled` while the CLI has emitted `partial` since 0.5.4 (G-ERR-R02)
- A strict agent validator rejected the exact envelope the product produces for a partial fan-out
- `error_code`, emitted on every failure, was likewise never contracted
- Both are now declared
- A4: `sftp ls` and `sftp stat` wrote to stdout with `println!`, bypassing the `output` facade that `src/lib.rs` documents as mandatory, so `--quiet` was ignored and piping into `head` aborted on EPIPE instead of exiting 141
- The `mkdir` / `rmdir` / `rm` / `rename` success lines were built with an inline English `format!`, so `--lang pt-BR` produced English for the most-read SFTP output
- Both now route through `output` and `i18n`
- A6: `cargo build --no-default-features`, the diagnostic configuration documented in `Cargo.toml`, failed with 62 errors
- The SFTP subsystem was never gated behind `ssh-real` despite calling `russh_sftp` directly
- The module, its emitters and the dispatch arm are now gated, and the subcommand answers with a typed refusal instead of failing to link
- G-QA-R02: `run_tunnel`'s resolution range (registry lookup → credential overrides → `ConnectionConfig`) is extracted into `resolve_tunnel_connection`, which takes an already-loaded record and touches no disk
- This is the range where the E3 agent-auth bug lived, and it had no offline coverage at all
- D5, two published schemas were unreachable through the CLI: `docs/schemas/` carried 22 documents while the embedded `SCHEMAS` catalogue in `src/cli/schema_cmd.rs` listed 20, so `ssh-cli schema dry-run` and `ssh-cli schema tunnel-closed` both exited 64 with `unknown schema`, while `docs/schemas/README.md` tells agents to discover schemas by running `ssh-cli schema` and documents those exact two names
- The contract was written where it was comfortable to write it and not where the consumer reads it
- Both entries are in the catalogue now and both commands exit 0 with the document
- The new `catalog_and_disk_agree` compares the two sets in both directions and validates that each name is the file leaf minus its suffix, so a schema published on disk without a catalogue entry: or a catalogue entry with no file, fails the suite instead of surfacing as `unknown schema` to an agent
- D10, the only discriminator of a successful tunnel shutdown was discarded: `src/tunnel.rs` emitted the closing event as `let _ = output::print_tunnel_closed_json(…)`
- The three endings `deadline`, `signal` and `accept_error` are distinguishable only by that event, so a failed emission handed an agent a successful exit with no way to learn which of the three had happened
- The failure is now reported through `tracing::warn!`, the same pattern R10 applied to the two discarded `Result`s in `copy_bidirectional`
- The exit code is deliberately unchanged: the shutdown really is a success, but the emission failure is visible on stderr, where the agent-native contract puts diagnostics
- K4, commit provenance silently degraded to `unknown` in the published crate: An external hygiene tool ran `git rm --cached .commit_hash`, and `cargo package --list` went from 245 entries to 244 with the file gone
- Inside a git checkout Cargo builds the package file list from the index, so an untracked file is dropped from the tarball no matter that it still sits on disk
- `.commit_hash` is step 2 of the `build.rs` precedence chain (`SSH_CLI_COMMIT_HASH` → `.commit_hash` → `git` → `unknown`) and the only provenance source that survives into a crates.io source build, where no checkout exists: so its removal made every published build report `unknown` without any gate objecting
- The existing `gap_rel_007_build_rs_precedence` asserted the file exists on disk, which is true and insufficient, so it stayed green throughout
- It now also requires the file to be present in the git index, skipping that check when `.git` is absent: which is precisely the crates.io build where the file is the surviving evidence rather than something to re-verify
- The first attempt asserted that the last matching `.gitignore` rule must be a negation
- That was measured wrong and reverted, because re-adding the file with `git add -f` while the ignore rule still won by last-match restored the packaged 245, proving tracking is load-bearing and the ignore rule is not

### Internal
- `src/cli/exec_target/` is now `src/cli/targeting/`: It arrived serving the three exec surfaces and was named after them
- It now also resolves `health-check`
- A module named narrower than its job invites the next caller to copy it rather than extend it, which is how the duplication it was extracted to remove comes back
- The module is private and every item is `pub(crate)`, so nothing outside the crate can notice
- The functions keep their `_exec_target` suffixes: they name an operation, not a location
- The first attempt at this rename used the shorter `target` and produced an unpublishable crate: see the packaging entry under Fixed
- `tests/gaps_v067_packaging_reach.rs` asserts that every source file survives `cargo package`: The gate has two arms: a literal parse of `.gitignore` that never skips, and a `git check-ignore` probe that is exact because it is the same matcher Cargo consults, and that announces its skip out loud when git is absent
- A denylist of names like `target` was rejected: once the ignore rule is anchored the name is harmless, so the denylist would be superstition guarding the last defect rather than the class
- The Portuguese 0.5.4 `### Corrigido` heading read `### Segurança`: Its four bullets are identical to the English ones under `### Fixed`, so the English twin proves the intent
- A Portuguese reader scanning 0.5.4 for fixes found none, and found fixes filed as security
- Correcting a wrong label restores the record
- It is not rewriting history
- The changelogs are now gated on structure, not only on tokens: Release 0.5.5 carried `### Security` three times, `### Added` twice and `### Changed` twice, and English had nine section headings against Portuguese's eight
- The missing one mattered: four security entries, including a `russh` CVE, sat under `### Adicionado`, so a Portuguese reader scanning for security work would not have found them
- Every existing gate compared tokens, and every word was present, so all of them stayed green: placement was the property that failed
- `the_newest_release_has_one_section_per_name_in_both_languages` now asserts allowed names, uniqueness, canonical order and EN↔pt-BR parity through an explicit pair table, scoped to the newest release so shipped history is not rewritten to satisfy a rule added after it
- Source comments are gated on being English, and the gate now exempts citations instead of files: Forty-five comment lines across nineteen files were Portuguese prose, the English changelog carried the word `desvio` in four places, and about thirty test functions had Portuguese identifiers
- Nothing caught any of it because nothing looked
- The first version, `source_code_is_written_in_english`, scanned for Latin diacritics against a per-file allowlist
- It grew to thirteen entries, each honestly earned because the accents really were the payload
- The cost was structural: once `tests/docs_conformance.rs` is exempt, all twelve hundred of its lines are exempt
- `tests/gaps_v068_source_language.rs` replaces it and strips string, char and backtick spans per line, so a comment may quote localized data and may not be Portuguese: which keeps those same thirteen files under the rule
- The allowlist is now empty, and that is a measurement: every accent in all thirteen, `src/i18n/pt.rs` included, already sits inside a literal
- Choosing the right unit to exempt removed the need to exempt anything
- The language scanner had to learn that `\` escapes nothing in a raw string: Its first draft read `r"\\?\"` in `src/paths.rs` as an escaped quote, swallowed the terminator and desynchronised the rest of the file
- The symptom was camouflage rather than silence: three `assert!` lines of legitimate Portuguese test data were reported while a genuinely bare `// é precomposed` eleven lines below was swallowed and never reported
- A scanner that mis-parses accuses the innocent and shelters the guilty in the same run, so raw strings are now parsed as raw strings and both literal kinds carry across lines
- Two Portuguese test names survived every gate, and the extension names why: `gap_sec_001b_todo_sidecar_pontuado_esta_barrado_do_pacote` and `gap_sec_002_e2e_recusa_grok_config_dentro_do_repo` carried no listed connective and no accent, so neither the shell gate nor the diacritic gate could see them
- They were found by reading
- `check_en_identifiers.sh` gains `_da_`, `_dos_`, `_das_`, `_dentro_` and `_esta_`
- It deliberately does not gain `_do_` or `_no_`: measured on this tree, `_do_` appears four times with two of them real English (`rejections_do_not`) and `_no_` thirty-nine times, essentially all English
- A signal that fires on correct code gets the whole gate switched off, so the homographs are excluded and the exclusion is written down where the next author will reach for them
- Seventeen Portuguese `assert!` messages in `tests/gaps_v039_integration.rs` are now English: They sat inside string literals, which is exactly where the citation rule declines to look, because that is where test data lives
- An assertion message is prose wearing data's clothes
- The rule cannot separate them mechanically, so this class stays a reading task and the limit is stated here rather than implied
- The `install-resolve` gate went red because of the host, not the code: It is the only gate that compiles the whole dependency tree from scratch, and it did so inside `mktemp -d`, which follows `TMPDIR` onto a `tmpfs` under a per-user quota
- Run after the `test` gate had already filled that `tmpfs`, it died with `Disk quota exceeded (os error 122)` building `aws-lc-sys`
- A gate that cries wolf gets skipped, and a skipped gate protects nothing
- Only `CARGO_TARGET_DIR` moved, to a nested directory under `target/`: what the gate proves is that the graph re-resolves without the repository lockfile, and that proof lives entirely in the `Cargo.lock` generated in the scratch dir, so where the objects land is irrelevant to it
- Reusing artefacts between runs cut the gate from 53s to 34s as a side effect
- The component-budget ratchet had no opinion on a cap that outran its file: `DECLARED_EXCEPTIONS` refuses growth and refuses rows whose file has fallen to budget, but a cap left behind by a shrinking file kept authorizing the difference: three entries held 13, 25 and 31 lines of credit nobody had argued for, room for a whole new function with the gate still green
- That is the failure this ledger's own doc comment names, arriving from the other direction, and two of the three were opened by this release's own extractions
- The caps are now at the measured sizes and `assert_ledger_is_current` fails above `MAX_LEDGER_SLACK` (8 lines)
- Equality was rejected deliberately: it would turn deleting one line red, making the cheapest way to stay green not tidying up, and a gate that punishes cleanup trains the opposite of what it wants
- The component-budget ledger paid off an entry: `json_wire/vps_export.rs` was frozen at 606 lines and now measures 586, inside `HARD_BUDGET`, so `the_exception_ledger_contains_no_stale_entries` failed until the row was deleted
- That is the ratchet working in the direction it was designed for: it records repayment as loudly as it records debt, and raising the cap to keep the row alive is the move it exists to block
- The ledger is down to six entries
- Agent-facing prose taught the alias instead of the canonical field: `docs/schemas/exec.schema.json` says to bind new consumers to `target_resolved` and `target_source`, with `host_resolved` and `host_source` as 0.5.5 read aliases written from the same value
- Both `SKILL.md` files and both `AGENTS.md` files named only the aliases, so an agent following the skill bound to the compatibility spelling the schema steers away from
- All four now teach the canonical pair first and name the aliases as what they are
- The gate battery is eleven gates, and five documents said ten: `check_all_gates.sh` grew `exec-target` without any of its readers noticing
- Both `RELEASE_CHECKLIST`s, both `CONTRIBUTING`s and both `TESTING`s are corrected, and the checklist now tells the reader to count with `--list` rather than trust the sentence: a number in prose is a fact with no gate behind it
- `docs/MIGRATION.md` had no 0.5.5 section: The release with two breaking changes to how every verb finds its host documented neither of them where an upgrader looks
- Both languages now carry the three legal forms, the 64/66 split and the provenance trio
- `docs/schemas/README.md` listed `telemetry`, `active_vps` and the secrets fields flat when the schema nests them under `local`: An agent binding to `.telemetry` reads `null`
- The Portuguese half was further behind still, describing the error envelope as three fields when it has twelve
- Both halves now match the schema, including the provenance trio and its absent-never-empty rule
- `SECURITY.md` documented the 0.5.4 fixes and stopped there, while 0.5.5 is itself a security release: The policy that readers consult first said nothing about GAP-SSH-EXEC-ARGC-001, the highest-severity defect of this line, a command that rewrote a machine nobody named
- Both halves gained a `Security Fixes in 0.5.5` section, the threat model gained a fifth adversary (ambiguous argv from a legitimate operator, which needs no attacker at all), and the STRIDE row for argv stopped writing `—` under Repudiation: before the provenance trio existed, a run genuinely could not be tied to a host, which is what Repudiation names
- The 0.5.5 release blockquote taught the alias in twenty-two places: It announced that the envelope "gained `host_resolved`, `host_source` and `active_fallback`" while `docs/schemas/exec.schema.json` says to bind new consumers to `target_resolved` and `target_source` and calls the other pair a 0.5.5 read alias
- The same sentence is replicated across twelve English and ten Portuguese documents, so a single wrong noun propagated to every entry point at once
- `docs/COOKBOOK.md` was worse than wrong prose: its copy-paste recipe parsed `.host_source`, so every reader who followed it bound new automation to the compatibility field
- Four table rows rendered with phantom columns on crates.io: A pipe inside a cell ends that cell even inside a code span, so ``| `scp upload|download` |`` is a three-column row in a two-column table
- The `secrets` row carried two such pipes
- Neighbouring rows escaped theirs correctly, so this was drift between rows rather than a convention
- `tests/gaps_v069_markdown_tables.rs` now counts unescaped pipes per row against the delimiter row that opens each table, across all twenty-five rendered documents: structural rather than lexical, so it also catches a cell lost to a stray pipe in prose
- Both changelogs claimed Keep a Changelog 1.1.0 and opened without `[Unreleased]`, and a gate enforced the omission: The format requires that section
- `top_release_sections` in `tests/docs_conformance.rs` took the first `## [` heading unconditionally, so adding it would have made the parser return empty and fire its own "the parser is broken, not the changelog" assertion
- The gate forbade conformance with the format the changelog header claims to follow
- The parser now skips `[Unreleased]` explicitly, `the_release_parser_skips_the_unreleased_section` pins that behaviour, and both files open with the section
- The footer was stale in the same way: `[Unreleased]` compared against `v0.5.3` two releases late, and neither `[0.5.5]:` nor `[0.5.4]:` existed
- `README.md` forbade in one section exactly what it did in another: The `Changelog` section said "Do not paste release notes into this README" while line 8 pasted a multi-release historical note, and `gap_doc_004_product_line_040_e_file_only` requires that note, so the two rules could not both be obeyed as written
- The rule is now stated as the project actually practises it: the banner and the one-line product-line note are the history this README carries, everything per-release lives in the changelog
- The three `llms.*` files opened with a release note where the format puts the project description: A consumer of the `llms.txt` convention reads the first blockquote after the H1 as what the project is
- It got the 0.5.5 security note instead
- The taglines are now the first blockquote in all three, with the release banners below them
- `docs/TESTING.md` catalogued 24 of the 40 test suites that existed then: The document exists to tell a contributor what each suite is for, and it stopped at `gaps_v064`, omitting `docs_conformance.rs` itself, `test_quality.rs`, the whole `gaps_v061`–`v063` band, and every suite this release added, including `gaps_v065_exec_target_designation.rs`, which guards the defect the release is named after
- A catalogue that covers 60% is worse than none: the reader stops looking after consulting it and concludes the coverage does not exist
- Both halves now list all forty-one, and `tests/gaps_v070_doc_reference_reach.rs` now fails the build if that ever drifts again: in either direction
- Both READMEs named all 47 commands and none of them were findable: The `Commands` table folded them, `sftp upload|download|ls|…` in one cell, which reads fine and defeats retrieval: `rg -F 'sftp ls' README.md` returned nothing while the command sat right there
- Unfolding into separate spans fixed the phantom columns at the same time, and `README.md` / `README.pt-BR.md` joined `FULL_INVENTORY_DOCS` so the folding cannot come back
- Added the missing `Target designation (0.5.5)` row, plus rows for `--hosts` / `--tags` and `--use-active`: the breaking change of this release had no line in the table that describes the product surface
- `.atomwrite/` is now barred from the published crate in `.gitignore`, in the manifest `exclude` and in `.cargoignore`
- The pre-publish package gate measured `cargo package --list` shipping `.atomwrite/scratch/old.txt` and `.atomwrite/scratch/new.txt`, staging left behind by an earlier editing round
- Being untracked by git was no protection: Cargo packages any file that is neither tracked nor ignored
- The `.gitignore` already covered `.serena/`, `.claude/`, `.setting.cyber/` and `.cursor/`, and missed the one sidecar the agent's own tool creates
- Added `gap_sec_001b_every_dotted_sidecar_is_barred_from_the_package`, which discovers dotted directories at the repo root instead of enumerating them
- Its sibling `gap_sec_001` pins one sidecar by name, and naming them one at a time is precisely how `.atomwrite/` slipped past four listed neighbours
- Whatever appears at the root must now be either deliberately packaged, which is `.git` and `.cargo`, or barred on all three surfaces, so a new sidecar turns the suite red on the day it appears
- Proven negatively: removing the `.gitignore` line fails the gate naming that exact file
- Corrected a false claim about `vps export` in both skills and both skill eval sets
- They said the export body stays TOML unless `--json` is passed
- It does not: the body follows the resolved output format, and that resolves to JSON whenever stdout is not a TTY, which is every agent invocation
- Measured, `vps export -o /tmp/hosts.toml` writes a JSON envelope into a file named `.toml`
- A TOML body needs `--output-format text`
- `export_import_toml_roundtrip` gave this the appearance of coverage while asserting only that `sshcli-enc:` is absent and that import accepts the file: and since import accepts TOML and JSON envelopes, the roundtrip closes with the wrong body
- The same claim was then corrected across the long-form surfaces and is now held by a gate
- The false `vps export` TOML claim was corrected in the twenty long-form surfaces that still carried it: `README` and `llms`/`llms-full` in both languages, `INTEGRATIONS`, `docs/AGENTS`, `docs/COOKBOOK`, `docs/MIGRATION`, `docs/HOW_TO_USE`, `docs/TESTING`, `docs/RELEASE_CHECKLIST` and `docs/schemas/README`
- Several stated the inverse of the measured behaviour outright, telling the reader that auto JSON on non-TTY does not apply to export
- The `docs/COOKBOOK` and `docs/HOW_TO_USE` examples now show both paths, because a copyable command that writes JSON into a `.toml` file teaches the wrong lesson twice
- New `no_document_claims_export_defaults_to_toml` in `tests/docs_conformance.rs`, phrase-based over the skills, the full-inventory documents and the fifteen surfaces that describe the export body
- It caught a line in `docs/HOW_TO_USE.md` that a hand-written regex sweep had missed on its first run
- The CHANGELOG history is exempt on purpose: those entries record what a past release claimed, and rewriting them would erase the evidence that the claim was made
- `export_import_toml_roundtrip` was renamed `export_body_follows_resolved_format_and_both_import` and now asserts what its old name promised
- It pins the JSON envelope from the default non-TTY path, pins a TOML body from `--output-format text`, pins that TOML says `username` where the envelope says `user`, and imports both
- The old version asserted only that `sshcli-enc:` was absent and that import accepted the file, and since import accepts TOML and JSON envelopes it closed on the wrong body while looking green
- `tests/gaps_v051_integration.rs` held `export_pipe_defaults_to_json_when_non_tty` the whole time: the suite proved the truth in one test and advertised the falsehood in the name of its neighbour
- A test whose name asserts what it never verifies is worse than an absent test, because the gap is invisible exactly where someone would look for it
- Both skills were rebuilt against the installed binary and cut to the 4000-word product budget, from 4906 and 5008 words
- Nothing was dropped from the contract: all 47 commands, every flag and every wire token survive
- The words came out of duplicated prose: `FORBIDDEN` bullets that merely mirrored a `REQUIRED` bullet in the same section, and an `Absolute Prohibitions` section that restated in full the prohibitions each section had already stated
- New `skills_stay_within_the_word_budget` in `tests/docs_conformance.rs`
- The budget was a standing written rule that nothing counted, so both files ran a fifth over it with 13 gates green
- A rule with a number and no gate is indistinguishable from no rule
- New `skills_name_every_command`: `LEAF_COMMANDS` is now asserted over both skills
- `FULL_INVENTORY_DOCS` deliberately lists the seven long-form documents and never included the skills, even though each skill opens its catalog claiming to be the whole surface: true when measured, held true by nothing
- Both skills now separate global flags from subcommand-local ones
- `--i-accept-network-exposure` was documented as global and is local to `tunnel`
- Placed before the subcommand it exits 2 at parse time
- The new section names the four that read like globals and are not
- Both skills now document the two meanings of `--disable-sudo`
- Before the subcommand it suppresses elevation for one invocation and touches no disk
- On `vps edit` it writes `disable_sudo` into the config and disables elevation on that host permanently
- `--enable-sudo`, the only way to undo the persistent form, appeared in neither skill, so an agent could enter a state the skill did not teach it to leave
- `--max-chars` was likewise unnamed and is now marked a legacy alias of the command cap
- Removed version narrative from both skills, which product rules forbid there: two `0.5.4+` headings, a `since 0.5.4/A3` clause, the gap identifiers `G-SCP-R01/R02` and `G-ERR-R01`, and a sentence describing how an earlier release behaved
- Published documentation now covers the 0.5.4 surface it had only announced
- `tunnel --reverse`, `--socks5` and `--remote-socket` shipped in this release and were mentioned in zero of `docs/HOW_TO_USE`, `docs/COOKBOOK`, `INTEGRATIONS`, `docs/MIGRATION`, `docs/CROSS_PLATFORM`, `SECURITY` and `llms-full.txt`, in either language: the three flags existed only in the release banner and the skill package
- `HOW_TO_USE` gained a Tunnel modes section with a runnable example per mode, `COOKBOOK` gained four recipes, `MIGRATION` gained a `Since 0.5.4` section covering both BREAKING changes, `SECURITY` gained a fixes section for A1/A2/A3 plus the exposure guard, and `CROSS_PLATFORM` gained the portability rule that the client may be Windows while the socket may not
- New `the_054_surface_reaches_every_user_facing_document` in `tests/docs_conformance.rs`
- That suite already asserted these tokens, but only over the two skills and the two changelogs, so it stayed green through all of the above: a gate aimed at the wrong subset is indistinguishable from an absent one on the scoreboard
- The contract is per document rather than one blanket list: the cookbook owes a recipe, the migration guide owes an upgrade note, the security policy owes the acknowledgement
- New `every_command_appears_in_every_full_inventory_document`: all 47 leaf commands must appear as literal full paths in every file claiming a complete inventory
- `llms-full.txt` and the `docs/AGENTS` tables used brace notation (`tls acme {account create,…}`), so a retriever searching `tls acme account create` found nothing and would conclude the command does not exist
- Compact notation suits a short index a human skims
- It does not suit a file whose stated purpose is machine ingestion
- New `every_schema_is_indexed_in_the_full_llm_map`: 22 schemas exist on disk and `llms-full.txt` indexed 14, omitting `dry-run` and every `sftp-*` and `*-batch` contract, exactly the envelopes needed to parse fleet and SFTP work
- `docs/schemas/README.md` had a directory-walking gate
- The discovery map did not, so it drifted behind the disk
- Corrected a cookbook example that the CLI would reject today: `--bind 0.0.0.0` was shown without `--i-accept-network-exposure`, which G-TUN-R13 made mandatory
- No gate executes cookbook examples, so the recipe aged into invalid without a signal
- Twelve stale product-line declarations moved from 0.5.3 to 0.5.4
- The drift was symmetric across both languages, so no parity gate could see it
- Roughly sixty remaining `0.5.3` mentions are legitimate version floors (`prefer 0.5.3+` for SFTP) or historical migration sections and were classified before editing rather than replaced blindly
- The SFTP permission mask is now documented as what it is: directional
- `SFTP_PERM_MASK` (`0o7777`) applies outbound on upload and deliberately keeps setuid/setgid/sticky on a file the caller owns
- `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) applies inbound on download, where the mode comes from the server
- Twelve lines across ten files named the first as if it were the only mask, and `docs/MIGRATION` announced A3 fifteen lines above contradicting it: a reader concludes the fix does not exist
- New `permission_mask_claims_are_directional` walks `docs/` and rejects any file citing `0o7777` without the inbound constant, so the class cannot recur rather than just this instance
- That mask gate initially walked `docs/` alone: the directory being edited, and went green while the identical claim stood uncorrected in `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt` and both `SKILL.md` files
- A gate written to catch wrong-subset targeting was itself scoped to the author's diff rather than to the claim
- Widening it to a hand-written list of four directories then missed `docs/schemas/README.md`, because `read_dir` does not recurse: a hand-written list of places to look is itself something that falls behind the disk
- The walk is now recursive from the root over `.md` and `.txt`, skipping `target/` and `.git`, so a document added in a new subdirectory is covered without anyone remembering to register it
- Six more files were corrected
- `--tags` is documented
- The selector is real on `exec`, `sudo-exec` and `su-exec`, and it had zero occurrences across all fifteen files under `docs/` while `--all` and `--hosts` were documented everywhere: an agent reading those files concludes tag fan-out does not exist and opens one process per host
- New `fleet_documents_name_every_selector` asserts every selector clap accepts appears where the fleet is described
- It also caught `--hosts` missing from both `docs/HOW_TO_USE` files, which no hand-written search had looked for
- Corrected the description of A1 in six places
- The 512-character banner log cap predates this release (G-SSH-14)
- What A1 fixed is that the cut was applied by byte index, which panics inside a multi-byte character, and `panic = "abort"` turns that into process death with no unwind: a remote peer could kill a multi-host fan-out with one non-ASCII character
- `SECURITY.md` titled the issue "unbounded server-sent pre-auth banner" and claimed the fix bounds what the peer can make the client hold
- Russh materialises the whole banner before the callback, so the residual limit is now stated explicitly: A1 bounds the panic, not the allocation
- Corrected the claim that `tunnel --bind` is IP-validated by clap in all cases
- That holds for the local bind
- Under `--reverse` the exposed end is the positional `<remote_host>`, guarded by `guard_remote_exposure`, which compares text instead of parsing an IP because RFC 4254 assigns meaning to names and to the empty string: a typo there is exit 64 from the guard, not exit 2 from clap
- Four lines said otherwise
- Documented that `--bind` is accepted and then silently discarded under `--reverse`: reverse delivery is forced to loopback and `ReverseServe` never receives the address
- The docs correctly said the acknowledgement guards the server's bind, but none warned that the flag itself does nothing there, so `--reverse --bind 192.168.1.10` changed nothing and warned about nothing
- `docs/RELEASE_CHECKLIST` gained item 26, the 0.5.4 honesty block
- Measured before the fix: both checklists and both `docs/TESTING` files mentioned `tunnel_closed`, `--select` and `--count-only` on line 3 only, the release banner, and named none of the three tunnel modes anywhere, with zero mentions of `--dry-run` or `--tags`
- A checklist exists to be a contract, and this one announced the release in its header while verifying nothing from it in its body
- The four files are now under `SURFACE_054`, and `gaps_v060_tunnel_modes.rs` is named in the suite inventory
- One terminology line corrected rather than nine
- `docs/MIGRATION` line 63 used master-key as the product term in both languages
- The other eight mentions across `docs/` explicitly describe it as the legacy keyring alias accepted on read, which is true, so a blind replacement would have destroyed eight correct statements
- `TransferResult` carries the two durability flags
- `print_transfer_json` takes it by reference rather than eight positional arguments
- `build_tunnel_closed` / `build_tunnel_listening` split payload construction from emission
- `tunnel_closed`, `forwards_served` and `capacity_waits` previously appeared outside the emitter in exactly one place: a test asserting the CHANGELOG mentions them
- Deleting the emission would not have failed the suite: prose validating prose, one step removed from the tautology G-QA-R01 was written to stop
- New `tests/gaps_v061_error_taxonomy.rs` and `tests/gaps_v061_scp_durability.rs`
- New tunnel tests covering agent-auth forwarding, the deliberately non-overridden registry timeout, every `close_reason` branch, the `bind` field, and `forwards_served` driven through a real loopback accept with an injected client
- `gap_deny_002_deny_toml_sem_ignore_cve` pinned `multiple-versions = "warn"` literally, so tightening the policy broke it while loosening it to `allow` would have passed
- It now rejects `allow` and accepts anything stricter
- `known_hosts` persistence pre-sizes its buffer from the entry count instead of growing from empty on every TOFU write
- Two Portuguese doc-comments in `src/ssh/client_real_scp.rs` and `src/retry.rs` translated to English, per the English-source rule
- `scripts/check_all_gates.sh`: runs the whole mandatory battery, `fmt`, `build-release`, `build-no-default`, `clippy`, `test`, `deny`, `cross-targets`, `advisory-freshness`, `en-identifiers`, `install-resolve`, in one invocation, with a TSV or NDJSON record per gate on stdout, per-gate logs on stderr and a non-zero exit if any gate is red
- It exists because `cargo clippy` and `cargo test` both abort on the first unbuildable target, so a single broken test file hid the state of every gate behind it: which is exactly how the local inventory came to declare 835 green with four gates red and one target that did not compile
- It caught a rustfmt violation on its first run
- Not CI: no workflow, no runner, no network
- The battery is sequential by design, because the cargo gates contend for one `target/` lock
- `scripts/check_advisory_freshness.sh` had no other caller, so it was only reachable through a script nothing documented
- The runner and that gate are now declared in `CONTRIBUTING`, `docs/TESTING` and the release checklist in both languages
- `tests/gaps_v064_gate_runner.rs`: turns the runner's coverage into a contract
- Every `scripts/check_*.sh` must be a gate, and every `scripts/*.sh` must be either a gate or named: with a reason, in a `Declared non-gates` block in the runner's header
- Coverage was previously complete only by coincidence of naming: a future `scripts/check_foo.sh` would have slipped in unnoticed, and a battery that omits a script in silence reads as total coverage
- The suite also pins `--locked` on the test gate and asserts the runner is documented in both language versions of all three maintainer documents
- Directory listings are sorted before assertion, because `std::fs::read_dir` documents its order as platform-dependent and free to change between calls
- `tests/docs_conformance.rs` now asserts the `CONTRIBUTING`, `docs/TESTING` and `docs/RELEASE_CHECKLIST` tokens over both language files
- Checking only English is how `CONTRIBUTING.pt-BR.md` came to omit `gaps_v040`, the explicit suite list and the entire cross-target gate section while the suite stayed green: the reader who does not read English was told strictly less, and the parity gate could not see it
- The pt-BR file also carried an elided suite range (`v038 … v051`), the exact pattern its English counterpart warns against because a `contains` check silently drops the suites in the middle
- D1: `tests/gaps_v040_integration.rs` called `tunnel_subsystem()` and `output_subsystem()`, neither of which existed, so the target did not compile and `clippy` and `test` aborted before a single test ran
- Both are implemented as thin wrappers over a shared `concat_subsystem` helper, alongside the `i18n_subsystem()` the original author had written
- Test code is excluded from the concatenation on purpose: including it would let an assertion pass by matching the text of the test itself
- D4: `tests/gaps_v063_secret_stdin.rs` required exit 64 from a clap parse error
- Confirmed against clap 4.6.6, whose `error::Error::exit` documents "exits with a status of `2`": the sysexits codes in `src/errors.rs` apply only to product errors raised after the parse succeeds
- The companion `assert_ne!` was passing vacuously, because 64 is never what the parser emits
- It now excludes 2, which is the code the supported form genuinely cannot produce
- D7: `tunnel::tests::accepting_a_connection_increments_forwards_served` passed alone and failed in the suite at a rate tracking `--test-threads`
- `serial_test::serial` serializes only the tests that carry it, so a non-serial reader of the process-global stop flag still raced marked writers
- Fixed with both halves: `#[serial_test::serial]` plus `crate::signals::reset_flags_for_tests()`, and verified by repeated runs at 8, 32 and 72 threads rather than by one green execution, since a single pass cannot distinguish a fix from a race
- The component ratchet then bit the fix itself: `src/tunnel.rs` grew past the hard 600-line budget, and `DECLARED_EXCEPTIONS` entries may only shrink
- `TunnelStats` and its `close_reason` unit test moved to a new `src/tunnel/stats.rs`, dropping `tunnel.rs` to 570 and removing its ledger entry entirely
- Raising the cap was the dishonesty the ratchet exists to block
- D3: `scripts/e2e_real_ssh.sh` replaced a non-executable `--bin` with the default and continued, printing `PASS E01` through `PASS E16` for a binary the operator never named, in a harness whose own usage text promises that an unusable environment is a failure and never a silent skip
- A `BIN_EXPLICIT` intent flag, set by `--bin` and by the harness-only `SSH_CLI_E2E_BIN`, now makes that case exit 2 with `FAIL E00`
- The auto-build survives only when nothing was named
- Verified negatively
- D9: `tests/gaps_v062_i18n_reachability.rs` searched `src/i18n.rs` for `fn en(msg: &Message)`, which C3 had moved to `src/i18n/en.rs`, so the count was unconditionally the whole file and the guarded branch could never be taken
- The dead search and its condition are gone and the doc comment now describes what the code does
- D6: the `scp-transfer` field list in `skills/ssh-cli-en/SKILL.md` gained `ok`, matching the pt-BR skill that already had all four
- The parity gate's token list gained `ok`/`direction`, `mtime_preserved` and `durable`: the three that actually discriminate, since the bare field names already appear in both skills through other contexts and the check is `contains`
- E1: `docs/TESTING.md` and its pt-BR twin catalogued `tests/gaps_v063_gate_falsification.rs`, a suite this repository never had
- The bullet promised that every check of `scripts/check_en_identifiers.sh` had a standing falsifiability proof, and the trail ended at a filename
- The forward sweep: is every suite on disk listed?, returned a clean zero, because a phantom is invisible from that side
- Both halves now name the artefacts that exist, and `tests/gaps_v070_doc_reference_reach.rs` sweeps in both directions so the class cannot return
- E2: `exec`, `sudo-exec` and `su-exec` schemas described `host_resolved` as a field of its own, "VPS the command ran on", while `scp-transfer`, `sftp-transfer` and `health-check` in the same directory correctly labelled it a compatibility alias of `target_resolved`
- Three schemas of one release taught two different answers to "which field is canonical"
- Worse, they promised the field goes `empty` on old events, while `error-envelope.schema.json` promises `Absent — not empty` and argues why: an empty string asserts a host was chosen and has no name
- A consumer written against the wrong one reads absence as a hostname or a hostname as absence
- E3: `active_fallback` in `error-envelope.schema.json` anchored its meaning in `host_resolved` and `host_source`, never naming the canonical pair, the alias explaining the canonical field instead of the reverse
- E4: `docs/MIGRATION.md` titled its wire section "0.5.3 wire format, current" while the line below it read "Product line: 0.5.5", and listed no wire change from this release at all
- The schema version is still 3, so the heading was defensible and still misread as "0.5.3 is the current release"
- The heading now dates the format and states its reach, and the section carries the 0.5.5 additions: the provenance trio, its absence rule, the exit 64/66 split and the fleet selectors
- E5: fourteen command groupings folded several leaves into one token, `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` in both `docs/AGENTS.md`, `ssh-cli exec|sudo-exec|su-exec` in the agent docs and in `docs/schemas/README.md`, plus the `locale`, `tls` and `secrets` families
- The folded form satisfies a human reader and returns nothing to a literal search for `ssh-cli sftp download`, which is how an agent looks a command up: the substring is simply not in the file
- Unfolded, the searchable-with-prefix count in `docs/AGENTS.md` went from 16 of 47 to 38 of 47, with the remainder carried by the inventory table that `every_command_appears_in_every_full_inventory_document` already gates
- E6: `SECURITY.pt-BR.md` had lost five technical items its English twin carried, in a section about transport crypto: the rustls CVE window (`0.23.13–0.23.17`) a reader must avoid, the four `tls mtls` subcommands, the absolute-path alternative on the VPS record, the `instant-acme` crate, and the `0o600` mode under `tls/acme/`. mTLS and ACME had been folded into one bullet, 8 bullets in English against 7 in Portuguese, and the reason compression is disabled (side-channel reduction) was gone
- A reader in Portuguese could not tell which rustls versions to avoid
- E7: the security document contradicted itself about the release it documents
- The supported-versions table marked all of `0.5.x` as current with no caveat, while the section 60 lines below explains that `0.5.0`–`0.5.4` carry GAP-SSH-EXEC-ARGC-001
- The assets list named "the ability to run remote shell / SCP / local port forwards" without the identity of the host reached, although the new section argues the critical dimension is not what runs but where
- And accepted residual risk (2) still read that the remote host is trusted "for the one-shot command the operator requested", which presumes the host is the request: the exact premise this release broke
- All three now state the qualification
- E8: `docs/RELEASE_CHECKLIST.md` was missing one policy rule its pt-BR twin carried: never publish `gaps.md` nor assert its FIXED text from product tests (G13/G15)
- Ninety bullets against ninety-one: the kind of gap a reading pass does not surface and a per-section count does immediately
- All seven bilingual pairs under `docs/` now match on H2, H3, bullet and fence counts
- E9: the exec-family schemas listed `host_resolved` and `host_source` in `required`, and `health-check.schema.json` listed `host_source` but not `host_resolved`, an asymmetry that marks drift rather than a decision
- A compatibility alias inside `required` stops being an alias: the whole point of one is that a later major can drop it without breaking validation, and requiring it closes that door
- `scp-transfer` and `sftp-transfer` from the same release already required only the canonical pair, and `tests/gaps_v061_scp_durability.rs` states the underlying rule for a different field: a name in `required` invalidates every event that lacks it
- All six now require `target_resolved`/`target_source` and leave the aliases optional
- The CLI keeps emitting both, so no existing consumer sees a change
- E10: `health-check.schema.json` escaped the alias correction entirely, and the reason is instructive: that pass was driven by searching for the word `empty`, the false promise the exec family carried, and this file never used it
- Its defect was structural instead
- `HealthCheckJson` flattens `TargetEcho`, whose four fields are all unconditional, so the success envelope always carries `host_resolved`: and the schema never declared it
- The canonical `target_source` pointed at the alias `host_source` as though the alias held the meaning, the reverse of the pattern every other schema follows
- Nothing failed validation, because `additionalProperties` is `true`: the defect was silent
- The lesson is that auditing an alias means diffing the fields a DTO serializes against the schema's `properties`, not grepping for suspicious phrases
- E11: three further schema-versus-emitter gaps, all silent for the same reason: `error_code` was described as "Present on every failure envelope" while sitting outside `required`, though `print_error_envelope` writes it as an unconditional `&str`
- `vps-show.schema.json` declared none of the five conditional fields `MaskedVpsJson` emits (`tags`, `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`), and `vps-list.schema.json` `$ref`s that item, so the gap propagated to the listing
- And eleven of the twenty-two schemas had no top-level `description`, which matters because `ssh-cli schema <name>` hands back the raw body and half of them did not say which command they belonged to
- E12: `docs/schemas/README.md` told parsers to branch on `event` without listing the payloads that carry none
- Measured against the binary: `exec`, `sudo-exec`, `su-exec`, `vps-show` and the single-host `health-check` emit an object with no `event` key, and `vps list` emits a bare array with no envelope at all
- A parser treating a missing `event` as malformed rejects six valid payloads
- The note about CRUD events without schemas also hid four of them behind an ellipsis: `vps-import` most consequentially, since it already appears in the `operation` enum of `dry-run.schema.json`, which sends the reader hunting for a schema of the real event
- E13: `docs/schemas/README.md` carried the one command grouping in the repository with no individual table in the same document to compensate: `ssh-cli sftp mkdir|rmdir|rm|rename|stat` collapsed five leaves into a token that matches none of them, and the batch lines did the same for `scp`/`sftp upload|download`
- Elsewhere a grouping sits beside a table listing each leaf, so search still finds it
- Here nothing did
- All nine transfer leaves are now literally searchable with the `ssh-cli` prefix
- `docs/RELEASE_CHECKLIST.md` also ended on a blank line, alone among the fourteen files of the seven pairs, the kind of drift a line-count check reports as parity because two errors cancel: the English file's trailing blank exactly offset the extra bullet the Portuguese carried


## [0.5.4] - 2026-08-06

### Security
- A1: Fixed a remote pre-authentication denial of service
- `auth_banner` sliced the server-sent banner at byte 512 (`&banner[..512]`)
- A multi-byte character on that boundary panicked, and because the release profile sets `panic = "abort"` the whole process died: taking any `--all` fan-out with it
- Truncation is now on character boundaries
- A2: ACME/mTLS private keys are no longer briefly world-readable
- `write_secret_file` used `std::fs::write` (creating at `0644` under the default umask) and narrowed permissions afterwards with the error discarded
- It now delegates to a shared helper that creates at `0600` via `O_EXCL` and propagates permission failures
- A3: `setuid`/`setgid`/`sticky` bits sent by the server are no longer reproduced on downloaded files
- Inbound modes are masked with `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`) in both SFTP and SCP
- Outbound modes from local files keep the full mask
- A6: `parse_hex_key` rejects non-ASCII before slicing, so a `secrets.key` containing multi-byte UTF-8 totalling 64 bytes returns a typed error instead of panicking

### Added
- C1: Agent-native payload shaping as global flags: `--select`/`--fields`, `--filter` (repeatable, AND), `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- Applied at the single JSON serialization funnel before the envelope is built, so the oversized payload never exists
- Measured: `vps list` drops from 943 to 19 bytes with `--select name --limit 1`
- A malformed `--filter` fails at parse time (exit 64) rather than silently matching nothing
- C2: `--no-input` refuses stdin declaratively instead of blocking forever on an absent human
- C2: `--dry-run` prints the plan for a destructive operation and exits without executing it: `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt`
- The plan is JSON even in text mode (`docs/schemas/dry-run.schema.json`), because a preview exists to be diffed
- On any other command the flag is rejected with exit 64 rather than accepted and ignored: the failure mode `--no-input` shipped with
- Preconditions run first, so previewing the removal of an absent host still exits 66 instead of promising a success the real run would not deliver
- G-TUN-R01: `tunnel --reverse` asks the server to listen and delivers connections back to a local port
- `REMOTE_PORT 0` is accepted in this mode only: the server allocates and reports the port it bound, which is what reaches `tunnel_listening.local_port`
- A remote bind outside loopback requires `--i-accept-network-exposure`: in this direction the exposed end is the server's listener, so guarding the local `--bind` would check the wrong side
- G-TUN-R02: `tunnel --socks5` serves a SOCKS5 proxy (RFC 1928, no-auth + `CONNECT`)
- Every accepted connection becomes one `direct-tcpip` channel
- Host names are forwarded unresolved so they mean what they mean on the remote side
- `BIND` and `UDP ASSOCIATE` are answered with reply code `0x07` instead of a bare close
- The handshake is capped at 1024 bytes, above the 519 the RFC permits and far below anything usable to make the proxy buffer at will
- G-TUN-R03: `tunnel --remote-socket <PATH>` forwards a local port to a Unix domain socket on the remote host (`direct-streamlocal@openssh.com`)
- The path is validated as absolute and NUL-free
- It is deliberately not checked against the local filesystem, which has nothing to do with the server's
- Tunnel wire: `tunnel_listening` and `tunnel_closed` gained `mode` (`local` / `socks5` / `streamlocal` / `reverse`)
- It is the discriminator that says how to read the sibling fields: under `reverse` the listener is the server's, and under `socks5` there is no single destination
- It defaults to `local`, so events written before 0.5.4 still validate
- G-TUN-R07: New `tunnel_closed` event (`docs/schemas/tunnel-closed.schema.json`) with `reason` (`deadline` / `signal` / `accept_error`), `forwards_served`, `capacity_waits` and `duration_ms`
- Those three endings previously shared exit 0 and were indistinguishable
- G-TUN-R06: `tunnel_listening` gained the `bind` field, so an agent can audit from the contract alone whether a service was published beyond loopback (additive, existing consumers unaffected)

### Changed
- BREAKING, G-ERR-R02: Partial multi-host failure now exits 1 with `error_code: "partial_failure"` and `error_class: "partial"`, carrying `failed`/`total`
- It previously exited 65, the same code as malformed TOML, so an agent could not tell "1 of 10 hosts failed" from "the config is corrupt"
- Exit 65 is again reserved for genuine data errors
- BREAKING, G-TUN-R13: A non-loopback `--bind` now requires `--i-accept-network-exposure`
- Without it the tunnel exits 64 before any network I/O
- `--bind 0.0.0.0` previously published the forwarded remote service to the local network in silence
- G-TUN-R08: `--bind` is validated as an IP address by clap, so a typo fails at parse time (exit 2) instead of after a full SSH handshake
- G-TUN-R09: Bind failures map by `ErrorKind`: `AddrInUse`/`PermissionDenied` → 74 (retryable), `AddrNotAvailable`/`InvalidInput` → 64
- All of them previously collapsed into 65 classified permanent

### Fixed
- C2: `--no-input` now refuses stdin on `vps add` and `vps edit`
- The refusal was implemented in the exec/scp/tunnel override path only, so both registry commands: the ones most likely to run unattended, accepted the flag and read the password anyway
- The guard moved into `read_secret_stdin`, so every caller inherits it
- Covered by `tests/gaps_v059_agent_native.rs`, which did not exist when the flag shipped
- Docs gate: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` is green again: 56 broken intra-doc links across 14 sites are now fully qualified, and links to `pub(crate)` items (`take_utf8_capped`, `EXEC_CAPTURE_HARD_MAX_BYTES`, `lock_config`) became inline code, since private items have no public page to point at
- E3: `tunnel` honours `--use-agent` and `--agent-socket`
- clap accepted both and the dispatch discarded them, so a host registered for agent auth could not open a tunnel at all
- G-TUN-R10/R11: Forward copy uses `tokio::io::copy_bidirectional`, which reports per-direction byte counts and the error
- The previous two-`copy` `join!` discarded both `Result`s, making a mid-transfer failure indistinguishable from a clean finish at any verbosity
- G-TUN-R12: Concurrency saturation is announced once per run and reported as `capacity_waits`
- It was previously invisible and surfaced only as unexplained latency
- E4: The human banner prints the effective bind address instead of a hard-coded `localhost:`
- E7: `open_tunnel_channel` preserves the underlying error via `channel_src` instead of flattening it into a string
- E6: `tunnel-listening.schema.json` matches the clap range for `remote_port`
- With the modes added later in this release the shared minimum is 0, since `socks5` and `streamlocal` have no single remote port and `reverse` uses 0 to mean "server allocates"
- `local_port` keeps `minimum: 1`, which is the field an agent connects to
- B2: SFTP verifies byte counts in both directions
- SCP had checked `sent != size` and `received != size` since 0.5.2 while SFTP checked neither, so a truncated SFTP transfer reported `ok` with a plausible `bytes` field: the loop counted what it wrote, never what survived
- Uploads now compare against the announced local size and re-`stat` the remote (the destination-effect proof: a quota or a full filesystem is invisible otherwise)
- Downloads verify before the atomic rename, so a short read never reaches the final path
- Servers that omit `size` are accepted rather than failed, since the attribute is optional in the protocol

### Internal
- G-QA-R01: New `tests/test_quality.rs` fails the build on assertions between two literals or `assert!(true)`
- `src/tunnel.rs` shipped `assert_eq!(0_u64, 0)` named `timeout_zero_conceptually_rejected`, which exercised no product code while implying the one-shot guard was covered
- G-QA-R03: Documentation assertions moved to `tests/docs_conformance.rs`, so editing prose no longer turns the behavioural gate red without a functional regression
- G-DOC-R01/G-DOC-R02: Closed alongside the split above: `CONTRIBUTING.md` now lists every test suite by name instead of naming `gaps_v040` as the whole surface, and the pt-BR skill documents the `scp-transfer` fields (`ok`, `direction`, `bytes`, `duration_ms`) its English twin already did
- Both were fixed in this release without being cited in it
- G-TUN-R04/R05, E1, E2: Tunnel tests now exercise real behaviour: the `timeout_ms == 0` guard is called, `run_tunnel_with_client` is actually invoked, and the ephemeral-port assertion checks the bound port instead of `include_str!`-matching its own source
- C3: The eight-line "N/M failed" block duplicated across five batch paths is replaced by `errors::finish_batch`
- G-TUN split: `src/tunnel.rs` is now a module root over `local`, `reverse`, `socks` and `streamlocal`
- The three local-listener modes share one accept loop parameterised by destination: copying it would have meant maintaining the signal handling, admission gate, drain and saturation accounting in triplicate, which are exactly the parts that go subtly wrong in one copy and not the others
- `run_tunnel` takes a `TunnelRequest` instead of fifteen positional arguments, where a transposition compiled cleanly and only surfaced as a tunnel pointing somewhere unintended
- G-QA-R01: `gap_tun_003_source_local_addr` no longer greps `src/tunnel.rs` for the strings `local_addr()` and `effective_port`
- That assertion passed when the text sat in a comment, kept passing after the behaviour was deleted, and failed for the one reason that is not a regression: the module split moved the code
- It now drives the real accept loop with an injected client and reads the port the product published
- Unsolicited `forwarded-tcpip` and `forwarded-streamlocal` channels are rejected with `AdministrativelyProhibited` instead of russh's default of accepting and dropping them
- The inbound queue is bounded, so a server cannot grow client memory by opening channels faster than they are drained
- A full queue applies backpressure over the wire


## [0.5.3] - 2026-07-30

### Fixed
- G1: SFTP upload no longer truncates destination to zero bytes (`FileAttributes::empty()` instead of `Default`)
- G2: `-v` filter is crate-scoped (`warn,ssh_cli=…`)
- Never bare global `debug` (no password leak via `russh::client::encrypted`)
- G3: SFTP SETSTAT sends `atime`+`mtime` together (no epoch atime)
- G4: SFTP `set_metadata` Result is propagated (mutating SETSTAT is not best-effort)
- G5/G17: Multi-file SCP/SFTP cancel fills cancelled remainder
- `results.len() == input.len()`
- G6/G11: Signal-flag tests stay serial
- `reset_flags_for_tests` for isolation
- Cancel cardinality tests
- G7: E2E real SSH covers SFTP upload/download checksum matrix + recursive tree (E17/E18)
- G8: `exec --json` single-step emits exactly one NDJSON object (multi-step guard on JSON path)
- G9: SCP download propagates `sync_data` failure before atomic rename
- G10: Formatting debt addressed with release gate (`cargo fmt --check`)
- G12: Permission bits masked with `SFTP_PERM_MASK` (`0o7777`)
- G13: Removed circular test that asserted FIXED text in `gaps.md`
- G14: Graduated verbosity: `-v`/`-vv`/`-vvv` via `ArgAction::Count` (info/debug/trace, always allowlisted)
- G15: Inventory criteria require destination effect proof (checksum), not structural self-cert
- G16: English identifiers and channel errors in `client_real_scp.rs`
- G18: SFTP download local `set_permissions` failures are surfaced
- G19: Named `SFTP_PERM_MASK` constant (no bare magic mask)

### Changed
- Version 0.5.3


## [0.5.2] - 2026-07-19

### Added
- Global `--json`: (G-AUD-01): agent-friendly alias that forces JSON output (clap `from_global` on subcommands)
- `exec` / `sudo-exec` / `su-exec` active VPS: (G-AUD-04): one positional = COMMAND against `connect` active host
- `vps path` JSON envelope: (G-AUD-02): `event: vps-path` when format is JSON
- `fs_perm` module: (G-AUD-24): single source for secret file/dir Unix modes

### Fixed
- Password argv warning false positive: (G-AUD-08): inspect real `Option` fields, not `Debug` strings
- TLS missing PEM files: (G-AUD-05): `FileNotFound` / permanent class (not exit 74 retryable)
- `vps export` honors global JSON format: (G-AUD-03)
- ACME account create requires `--contact mailto:…`: (G-AUD-06/28)
- Primary auth mutual exclusion: on write (G-AUD-07): exactly one of password / key / agent
- Secrets error messages: no longer advertise env key stores (G-AUD-21)
- Log filter CLI-only: (G-AUD-22): ambient `RUST_LOG` ignored
- Use `-v`
- Concurrency hard cap single source: (G-AUD-19/23): `constants::MAX_CONCURRENCY`
- Skill description ≤1024: chars (G-AUD-15)
- ACME validation permanent: (G-E2E-01): `invalidContact` / 4xx problem types → exit 64 non-retryable (`tls/acme_error_map.rs`)
- Single JSON on `vps add` auto-key: (G-E2E-04): fold `secrets_key_auto_created` into `vps-added` (one document)
- Root `schema` + `doctor`: (G-E2E-02/03): agent discovery parity
- `doctor` is an alias of `vps doctor`
- Version `-dirty` with `.commit_hash`: (G-E2E-06): honest provenance on dirty trees
- clap `env` feature removed: (G-E2E-08)
- Help no longer teaches env stores (G-E2E-07)
- Export redacted mask: (G-E2E-10): `FIXED_MASK` (three asterisks), not empty string (superseded: redacted `vps export` now emits an empty `password` and null for the other secrets, and the mask appears only in `vps list` and `vps show`)
- `vps add --use-agent`: (G-E2E-19): registry auth triplo complete
- E2E harness offline SKIP: + release default bin (G-E2E-05)
- EN test idents (G-E2E-13)

### Removed
- `.github/workflows`: (G-AUD-11): local gates only
- No product hosted-runner config in tree
- `src/erros.rs` PT shim: (G-AUD-14)
- Product env config stores: `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` reads (G-AUD-12)

### Changed
- Version 0.5.1 → 0.5.2
- Integration tests aligned to CLI/XDG-only config (no env secrets/format store)
- Residual gate: `tests/gaps_v058_e2e_residual.rs` (G-E2E-01…15,17,19 FIXED, 16/18 MITIGATED)

### G-SFTP residual harden R01–R15

### Security
- Entry-name validation: + `ensure_local_under` on recursive/multi-file download (malicious SFTP server cannot escape local dest)
- Partial cleanup: on every SFTP download error (parity with SCP)
- Upload tree root: uses `symlink_metadata` (no-follow)

### Changed
- Outer wall-clock timeout: (`under_timeout`) on multi-file and FS ops (`ls|mkdir|rm|stat|rename`)
- `cli/scp_args.rs`: extracted (SRP, `cli/mod` monolith shrink)
- Docs/skills/llms: SCP = regular files
- Trees/FS = `sftp`

### G-SFTP: SFTP subsystem

### Added
- `russh-sftp` 2.3: (feature `ssh-real`), SFTP v3 client subsystem
- `ssh-cli sftp`: `upload|download` (optional `--recursive`), `ls`, `mkdir`, `rmdir`, `rm`, `stat`, `rename`
- Modules: `src/ssh/sftp_session.rs`, `sftp_path.rs`, `sftp_types.rs`, `src/sftp/`, `src/cli/sftp_args.rs`
- JSON schemas: `sftp-transfer`, `sftp-list`, `sftp-fs-op`, `sftp-batch`
- Gates: `tests/gaps_v057_sftp.rs`
- Multi-host `--all`/`--hosts` for SFTP upload/download (`map_bounded`)

### Changed
- ScpOptions / SftpOptions: `use_agent` + `agent_socket` (G-SFTP-17/18, CLI/XDG only)
- Stream transfers in 32 KiB chunks
- Partial download + atomic rename
- Symlink no-follow on trees

### Security
- Remote path validation (control chars / empty segments)
- Recursive depth cap
- Listing entry cap
- No full-file heap via `SftpSession::read/write`

### G-SSH: SSH client rules / russh

### Added
- `src/ssh/client_handler.rs`: TOFU `check_server_key` + `HostKeyOutcome` typed recovery (G-SSH-01/09/14)
- `src/ssh/client_connect.rs`: dial + auth chain file key → agent → password (G-SSH-04/16)
- `src/ssh/key_material.rs`: Unix 0600 private-key perms + RSA ≥2048 / DSA reject (G-SSH-03/07)
- CLI/XDG agent: `--use-agent`, `--agent-socket`
- `VpsRecord.use_agent` / `agent_socket` (no env store)
- Gates: `tests/gaps_v056_ssh.rs`

### Changed
- SSH client id: `SSH-2.0-ssh-cli` (no russh version banner fingerprinting)
- Config policy: explicit rekey 1h/1GiB, window 2MiB, packet 32KiB, TCP SO_KEEPALIVE via `socket2`
- deny.toml: ban `ssh2`, `thrussh`, `libssh-rs`
- SRP split of connect/handler/key load from session/SCP path

### Security
- Host-key change returns typed `HostKeyChanged` (exit EX_NOPERM) to agents
- Fail-closed when `known_hosts_path` is missing outside tests
- Password fallback remains opt-in via non-empty inventory secret (documented)

### G-UNSAFE: unsafe code and FFI

### Added
- `src/test_util/env.rs`: encapsulated `set_var`/`remove_var` with `// SAFETY:` (ed2024-ready)
- `src/vps/config_io.rs`: config path/load/save/permissions split (SRP / G-UNSAFE-10)
- Gates: `tests/gaps_v055_unsafe_ffi.rs` (allowlist, forbid, main order, no plaintext env)

### Changed
- `main`: `signals::register_handler()` before Tokio `new_multi_thread` (G-UNSAFE-13 / signal-hook first-hook)
- SIGTERM SAFETY: multi-bullet (async-signal-safe atomics only, double-hit justified vs `flag::register`)
- Windows console FFI: module safety docs
- One op per `unsafe` block unchanged
- VPS tests use `set_runtime_flags` for plaintext opt-out (no `SSH_CLI_ALLOW_PLAINTEXT_SECRETS` env)
- Secrets docs: fail-closed env key material
- Plaintext only via CLI flag
- Concurrency docs: CLI `--max-concurrency` + auto formula only (no env store)
- `forbid(unsafe_code)` on `ssh/mod`, `vps/mod`, `vps/model`, `vps/config_io`

### Security
- Product `unsafe` allowlist: `platform/windows.rs` + `signals.rs` only
- Test env mutation centralized

### G-ERR: error handling

### Added
- `SshCliError::Domain`: , `Crypto`, `Config` variants
- TLS/channel structured errors with optional `source`
- Helpers: `tls_msg` / `tls_src` / `channel_msg` / `channel_src` / `error_code()` (G-ERR-08/16)
- JSON envelope `error_code`: field for agents (stable snake_case)
- Gates: `tests/gaps_v054_error_handling.rs`
- SSH client split (G-ERR-12): `client.rs` facade + `client_real.rs` / `client_stub.rs` / `client_tests.rs`

### Changed
- Display messages lowercase, no trailing period (G-ERR-01 / thiserror convention)
- `paths` validation returns `SshCliResult` (no `anyhow`/`bail!`)
- `VpsRecord::validate*` → `Result<(), DomainError>`
- Secrets: env key material rejected (XDG + CLI only)
- Concurrency `resolve_limit` ignores env store
- `xdg_config_dir` uses `SshCliError::XdgDirectory`

### Security
- No secret material in error Display for crypto ops
- Fail-closed on `SSH_CLI_SECRETS_KEY*` env store

### G-DOM: domain types chrono/uuid/rust_decimal/url

### Added
- Four domain crates (coordinated): `chrono` 0.4.45 (`serde`+`clock`), `uuid` 1.24 (`v4`+`v7`+`serde`), `rust_decimal` 1.42 (`serde-with-str`+`macros`), `url` 2.5 (`serde`)
- `src/domain/` split (SRP): `error`, `names`, `ports`, `limits`, `command`, `time`, `ids`, `http_url`, `money`
- `Rfc3339Utc` / `AddedAt` / `CreatedAt`: `DateTime<Utc>` newtype
- VPS `added_at` + ACME timestamps
- `HttpsUrl` / `AcmeOrderUrl`: HTTPS-only parse for ACME order resume (XDG)
- `CorrelationId` (v4) / `BatchRunId` (v7): multi-host batch JSON field `batch_run_id`
- `Money<C: Currency>`: library-ready decimal money (not wired into VPS)
- Gates: `tests/gaps_v053_domain_types.rs` + proptest roundtrips (G-DOM-07/09)

### Changed
- Batch schemas (`health-check-batch`, `exec-batch`, `scp-batch`) require `batch_run_id` (UUID v7)
- Import `added_at` validates RFC 3339 (no length-only check)

### Security
- No `Local::now` in product sources
- No `serde-float` on decimals
- ACME URLs reject `http`/`data`/`javascript`

### G-TLS product: rustls / SSH-over-TLS / mTLS / ACME

### Added
- Feature `tls` (default): `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`
- `CryptoProvider::install_default`: in binary `main` (aws_lc_rs only, libraries never reinstall)
- `ClientConfig` builder: (`src/tls/client_config.rs`) with webpki-roots + optional mTLS client cert
- SSH-over-TLS: `ConnectionConfig.tls` / VPS fields `tls`, `tls_sni`, `tls_client_cert`, `tls_client_key`
- Dial TCP → rustls → `russh::connect_stream`
- CLI `tls`: `provider`, `paths`, `mtls import|list|show|remove`, `acme account create|show`, `acme issue --print-challenge`, `acme complete`, `acme status|list`
- XDG layout: `tls/mtls/<name>/`, `tls/acme/account.json`, `tls/acme/<domain>/` (0o600 secrets)
- Residual suite updates in `tests/gaps_v052_tls_policy.rs`

### Changed
- `deny.toml`: allow product `rustls`
- Keep bans on `openssl*`, `native-tls`, `libssh2-sys`, `ring`
- Allow license `CDLA-Permissive-2.0` (webpki-roots)
- PEM load via `rustls-pki-types::PemObject` (no unmaintained `rustls-pemfile`)

### G-TLS / rustls policy — prior session

### Added
- `src/ssh/connect.rs`: single `build_ssh_client_config` + Happy Eyeballs dial helpers (G-TLS-07/09)
- Residual suite: `tests/gaps_v052_tls_policy.rs`, lockfile/deny bans, no flate2, SECURITY/README policy (G-TLS-03)
- SECURITY Transport & crypto policy (G-TLS):, SSH ≠ TLS
- Aws-lc-rs
- Future rustls-only if HTTP
- Local gates only

### Changed
- SSH compression `none` only:, preferred algorithms never offer zlib (G-TLS-04)
- russh features: drop `flate2` (G-TLS-05)
- Keep `aws-lc-rs` only
- `deny.toml`: ban `openssl`, `ring`, `rustls` in addition to `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02)
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: crypto policy surfaces (G-TLS-01/06/08/11/12)

### Security
- No product TLS stack
- No OpenSSL/`native-tls`/`ring`/`rustls` in the dependency graph
- No product OTEL
- Secrets remain `SecretString` + XDG AEAD

### Type system

### Added
- Domain newtypes (G-TYPE-01…20): `src/domain/` with `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort` (ephemeral ≠ SSH)
- `ssh/session_io.rs`: extracted `truncate_utf8` / capped UTF-8 helpers (G-TYPE-14)
- Zero-cost layout tests for `SshPort` / `Option<SshPort>` niche

### Changed
- `VpsRecord` / `ConnectionConfig`: fields are domain-proven
- `try_new` replaces infallible `new`
- `HostSelection`: `VpsName` / `HostTag` (parse at CLI boundary)
- `ExecOptions` / `ScpOptions`: `Option<TimeoutMs>`
- Steps are `Vec<RemoteCommand>`
- CLI ports: `value_parser!(u16).range(1..=65535)` on VPS add/edit and tunnel remote port
- Local bind still allows `0`
- `paths::validate_and_normalize` → `Result<VpsName>`: (proof retained)
- Import JSON: empty host/user rejected
- Builds via `VpsRecord::try_new`

### Security
- Auth material check centralized (`secret_nonempty`)
- No secrets in domain error messages
- No product OTEL

### Session notes (validation / serde / componentization)

### Added
- Validation pipeline (G-SERDE-01…14): `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`
- Module `src/validation.rs` (parse → serde → validate → domain)
- Host tags on agent JSON wire (G-SERDE-06): `tags` on list/export/import DTOs with round-trip tests
- Structural config validation on load (G-SERDE-04): empty host/user, port 0, limit/char limits, tags charset
- Fuzz target: `import_envelope` for import deserialize (G-SERDE-12)
- `ssh/connection.rs`: ConnectionConfig extracted (G-COMP-R)
- `cli/tests.rs`: CLI unit tests extracted from the monolith (G-COMP-R)

### Changed
- deny_unknown_fields: on TOML `ConfigFile` / `VpsRecord` (G-SERDE-05)
- Import JSON remains Must-Ignore with warn (G-SERDE-14)
- SCP multi-host fan-out: uses `Arc<ScpOptions>` (G-MEM-SCP)
- `apply_scp_options` takes `&ScpOptions`
- CI Actions pinned by commit SHA: (G-PROC-PIN)
- Serde deps caret: `serde = "1"`, `serde_json = "1"`

### Security
- No product telemetry (OTEL still forbidden)
- Secrets stay in `SecretString`
- Validation never logs secrets

### Changed
- O1–O6 + process (mandatory): global `--fail-fast` (`map_bounded_with`)
- Host `tags` + `--tags` / `vps --tag`
- Multi-cmd `--step` one session
- SCP `--scp-file-concurrency` parallel channel windows
- Arc `ExecOptions` fan-out
- Proptest packing + fuzz targets
- CI miri/geiger/sbom/proptest
- `scripts/release_attest.sh`
- Zero product telemetry
- Deep componentization (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01): extract `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`)
- `ssh/scp_wire.rs`
- `scp/{mod,batch}.rs`
- `output/{mod,batch}.rs`
- `cli/{mod,dispatch}.rs`
- Populate `commands/*` thin reexports
- Residual EN renames (`metadata`, `config_path`)
- OPEN product security remains 0
- Inventoriable monoliths split
- Componentization (G-COMP-02…04): split `vps/doctor.rs`, `vps/import_export.rs`, and `vps/health.rs` out of the `vps` monolith (`mod.rs` ~2428 → ~1698 LOC)
- Re-export `HostHealthResult` / `run_health_check` for existing callers

### Security
- Closeout meta-audit (G-CLOSE): doctor/concurrency/SCP integer casts use `TryFrom` (no truncating `as`)
- Remaining pure modules `forbid(unsafe_code)`
- Extract `vps/selection.rs` (SRP)
- Re-ran context7-cli + docsrs-cli + duckduckgo for skill compliance
- Security development audit (G-SECDEV): secrets cross the CLI boundary as `SecretString` (`read_secret_stdin` + `ExecOptions`/`ScpOptions`/tunnel/health overrides)
- Pure modules `#![forbid(unsafe_code)]`
- Deny `clippy::mem_forget` + undocumented/multi-op unsafe
- STRIDE map + CVSS v4 preference in `SECURITY.md` (+ pt-BR)
- Defensive security audit (G-SEC): deny `unsafe_op_in_unsafe_fn`
- Release `overflow-checks`
- TOFU fingerprint constant-time compare
- Product paths free of `.unwrap`/`.expect`/`unreachable!` in CLI parsers, concurrency admit, and single-host selection arms
- Import port via `u16::try_from`
- `SshCliError` is `#[non_exhaustive]`
- Threat model in `SECURITY.md` (+ pt-BR)
- CI job `cargo deny check` (`deny.toml`)

### Added
- Retry audit (G-RETRY): typed error classification (`ErrorClass` / `ErrorLayer` / `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) on `SshCliError`
- Named `retry::RetryConfig` with full-jitter backoff and agent defaults (max 2 retries on exit 74)
- JSON error envelope fields `error_class`, `retryable`, `suggestion` + schema update
- In-process auto-retry of non-idempotent remote ops remains off (agent re-invokes the process)

### Fixed
- Network audit (G-NET): SSH dial uses async DNS + Happy Eyeballs multi-address race (`net::dial_tcp` + `russh::client::connect_stream`)
- Enable `TCP_NODELAY` and SSH keepalives (`15s` / max `3`)
- Private-key load and known_hosts TOFU run in `spawn_blocking`
- Tunnel accept continues on transient errors and sets nodelay on local forwards

### Changed
- Hardcode audit (G-HC): central `constants` module for XDG file names, env keys, app identity, network defaults (`DEFAULT_SSH_PORT`, tunnel bind/origin), process timing, AEAD/keyring sizes
- Single `paths::xdg_config_dir()` helper
- No product secrets/URLs were present
- Hosts remain registry/CLI data

### Fixed
- External process audit (G-PROC): `build.rs` git probes set explicit `Stdio` (null/piped)
- Remote commands reject NUL before SSH exec packing
- Test `ssh-keygen` fixtures use direct argv + explicit stdio and skip if missing
- Docs: CROSS_PLATFORM / AGENTS process-boundary policy (no local OpenSSH spawn, MSRV ≥ 1.77.2 BatBadBut, remote `sh -c` packing only on target host)

### Added
- Multi-host bounded concurrency (modus operandi): `health-check|exec|sudo-exec|su-exec|scp --all` fans out with `Semaphore` + `JoinSet` (cap from `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / auto CPUs×RAM formula, clamp 1..=64)
- Batch JSON: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`)
- Tunnel accept forwards use the same admission gate
- Selective multi-host `--hosts a,b,c`: same bounded fan-out and batch JSON as `--all` (even for one name)
- Unified via `HostSelection` + `resolve_host_jobs`
- Multi-file SCP (single-host, G-PAR-47): `scp upload VPS f1 f2 … REMOTE_DIR` / download symmetric, one SSH session, serial transfers (auth once)
- Multi-host × multi-file SCP (G-PAR-48): `scp upload --all f1 f2 … REMOTE_DIR` / `--hosts a,b`, outer bounded fan-out per host session
- Files serial on each session
- Download writes under `LOCAL_DIR/<host>/`
- TOFU flock (G-PAR-49): `known_hosts` mutations exclusive-lock + reload-merge under multi-host first-connect
- `vps doctor --probe-ssh [--hosts a,b]`: single JSON root `event: vps-doctor` with `local` + optional `ssh_probe` (no dual roots)
- `map_bounded` cancel: stops admission on SIGINT/SIGTERM
- `force_exit` aborts in-flight JoinSet (timer poll)
- Debug logs `available_permits`
- Span `fan_out_unit` per task (G-PAR-52)
- Agent docs/skills: multi-host fleet + multi-file / cartesian SCP + doctor envelope contract

### Changed
- SCP upload/download path validation and post-transfer metadata use `tokio::fs` / `spawn_blocking` (non-blocking Tokio workers under multi-host fan-out)
- `scripts/dist_multiarch.sh` supports `PARALLEL_JOBS` (default 2) via `xargs -P`


## [0.5.1] - 2026-07-17

### Fixed
- Import/export agent roundtrip: `vps export` default body is TOML even on non-TTY
- JSON only with `--json`
- Import accepts TOML (EN+PT keys) and JSON `vps-export` envelopes (GAP-AUD-001/022)
- Wire dual-read: deserialize EN + legacy PT aliases
- Serialize English keys
- Schema v3
- Default `added_at` when missing (GAP-AUD-002/021)
- Supersedes 0.5.0 wire note (PT keys via `serde(rename)` only)
- `secrets init` / `reencrypt` JSON: envelopes (`event: secrets-init|secrets-reencrypt`) via `--json` or `--output-format json` (GAP-AUD-003)
- Empty command error is English technical (`empty command`) under any locale (GAP-AUD-004)
- CRUD/connect/import success paths emit structured JSON when format is JSON (GAP-AUD-008)
- SCP remote-missing message normalized to `file not found: <path>` (GAP-AUD-025)
- EC 66 retained
- Import TOML parse errors map to sysexits 65 (`TomlDe`) (GAP-AUD-012)
- `SshAuthentication` exit code aligned to 77 (GAP-AUD-020)
- Timeout values `< 1000` ms emit a stderr warning (GAP-AUD-009)
- `--include-secrets` to pipe/non-TTY requires `--output` or `--i-understand-secrets-on-stdout` (GAP-AUD-011)
- Doctor `secrets_plaintext_opt_out` is JSON bool (GAP-AUD-013)

### Added
- CLI flags: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (env layers deprecated, still work) (GAP-AUD-006)
- Event `secrets-key-auto-created` when primary key is provisioned on first write (GAP-AUD-007)
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018)
- Password-on-argv stderr warning (GAP-AUD-010)

### Changed
- Version 0.5.0 → 0.5.1
- Tracing / residual identifiers standardized to English (GAP-AUD-005)
- Portuguese type aliases in `erros` module marked deprecated (GAP-AUD-017)

### Notes
- No crates.io/GitHub publish without explicit maintainer OK
- SCP real transfer contracts from 0.5.0 §1.1 must not regress


## [0.5.0] - 2026-07-15

### Fixed
- CRITICAL: `secrets init --force` now re-encrypts existing host secrets under the new primary key and writes `secrets.key.bak` (GAP-AUD-SEC-001)
- Doctor `permissions` field uses English `"missing"` instead of Portuguese `"ausente"`
- Technical error messages, clap help residual PT, and product identifiers standardized to English
- VPS names with internal whitespace are rejected (GAP-AUD-VAL-001)

### Changed
- Semver 0.5.0: English renames of public/lib identifiers and residual API surface (`generate_completions`, `plaintext_allowed`, `ENC_PREFIX`, `verify_tofu`, etc.)
- Wire TOML keys were still Portuguese via `serde(rename)` in this release (superseded in 0.5.1 by English serialize + dual-read EN/PT aliases, schema v3)
- `secrets init` / `secrets reencrypt` success lines go through `Message` i18n

### Notes
- No crates.io/GitHub publish in this change set without explicit maintainer OK


## [0.4.2] - 2026-07-15

### Fixed
- Tunnel ephemeral port: (`local_port=0`): after bind, JSON/banner report the OS-assigned port via `local_addr()` (never `0` post-bind) (GAP-SSH-TUN-003)
- Schema `local_port.minimum` is 1
- SCP remote missing: now exits 66 `ArquivoNaoEncontrado` (parity with local missing) instead of 74 `CanalFalhou` when OpenSSH reports `No such file` / `not found` (GAP-SSH-IO-010)
- Protocol/permission errors remain 74

### Added
- `vps export --json` agent-first envelope: `event: "vps-export"`, redacted hosts by default, no `sshcli-enc:` for empty secrets (GAP-SSH-UX-001 / EXP-001 parity)
- Schema `docs/schemas/vps-export.schema.json`
- Commit hash embed for crates.io: `build.rs` precedence `SSH_CLI_COMMIT_HASH` → `.commit_hash` pack file → git → `unknown` (GAP-SSH-REL-007)
- Official e2e E15 (tunnel port 0) + E16 (symlink) + E13 asserts exit 66
- ENV-001 fail2ban policy in script header (GAP-SSH-ENV-001, SCP-024)
- Suite `tests/gaps_v042_integration.rs`

### Changed
- Version 0.4.1 → 0.4.2
- Product-line docs + skills: tunnel remains positional args
- Port `0` = ephemeral
- Trust JSON `local_port` after bind
- Never invent `--local-port` (GAP-SSH-DOC-042)

### Security
- VPS TCP ban after audit e2e was fail2ban from intentional wrong-password tests (ENV-001), not TUN-003
- Whitelist/ignoreip is ops
- CLI one-shot does not open sshd permanently
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)


## [0.4.1] - 2026-07-15

### Fixed
- Export redacted empty secret: no longer emits `sshcli-enc:v1:…` ciphertext for password `""` (GAP-SSH-EXP-001)
- Empty secrets serialize as empty strings so cross-machine import of skeletons stays honest
- Tunnel one-shot deadline: after local bind no longer returns exit 74 `TimeoutSsh` when the agent already received `tunnel_listening` (GAP-SSH-TUN-002)
- Pre-bind timeout remains exit 74

### Added
- `tunnel` auth flag parity with exec/scp: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- `health-check` auth flag parity: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- SCP success JSON field `event: \"scp-transfer\"` + schema required (GAP-SSH-IO-009)
- Suite `tests/gaps_v041_integration.rs` (AUD-POST regression)
- `health-check` honors global `--replace-host-key` and enables JSON error envelope with `--json`

### Changed
- Version 0.4.0 → 0.4.1
- Product-line docs + skills document tunnel/health auth parity and scp-transfer event

### Security
- If you installed 0.4.0 from crates.io: redacted `vps export` could show fake empty-password ciphertext
- Tunnel agents could see `ok:true` then exit 74
- Upgrade to 0.4.1
- No telemetry

### Notes
- One-shot CLI: birth → execute → die
- Additive agent contracts only (PATCH)


## [0.4.0] - 2026-07-15

### Fixed
- SCP wire protocol: was broken on crates.io 0.3.9 (header used literal `\\n` instead of real newline `0x0a`, ACK/EOF sent empty data instead of byte `0x00`, status not validated, download header/terminator incorrect), SCP-010..013
- Remote SCP path shell-escape for spaces and meta-characters (SCP-014)
- Unit tests no longer crystalize the broken header form (SCP-015)
- Download no longer leaves a partial final file on failure: write `{path}.ssh-cli.partial` then atomic rename (SCP-022)
- Mode/times applied on the partial before rename (SCP-022b)
- Upload no longer loads the entire file into RAM (`fs::read`)
- Streams in 32 KiB chunks (SCP-018)
- `scp --json` enables the JSON error envelope on stderr (parity with tunnel, IO-007b)
- SCP file-only validation messages are i18n EN/PT (SCP-020b)

### Added
- Official e2e E10–E14 SCP coverage in `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, missing remote, mode/mtime preserve) (SCP-016, SCP-023)
- SCP flag parity with exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- Structured SCP success JSON + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-directional: remote `scp -tp`/`-fp`, `T` line + `C` mode parse, set_permissions + set_times (SCP-023/023b, e2e E14)
- `tunnel --json` emits structured `tunnel_listening` event after local bind (IO-008)
- i18n EN/PT success messages for SCP (SCP-020)
- Suite `tests/gaps_v040_integration.rs` (TEST-004)

### Changed
- Version 0.3.9 → 0.4.0
- Product-line docs document regular files only (no `-r` / no SFTP subsystem) and the 0.3.9 SCP wire regression (DOC-004, SCP-019, REL-004)
- Root docs honesty (SECURITY 0.4.x current, INTEGRATIONS real 0.4.0 surface, CONTRIBUTING gaps_v040) (DOC-004b)
- `docs/*` honesty: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + schemas index cover SCP file-only, partial, 32 KiB stream, preserve, `scp --json`, `tunnel --json` / `tunnel_listening`, and 0.3.9 wire warning (DOC-004c)
- `skills/*` honesty: bilingual agent skills + evals teach SCP file-only, scp-transfer JSON, `.ssh-cli.partial`, 32 KiB stream, mtime/mode preserve, tunnel `--json` / `tunnel_listening`, timeout flag matrix (DOC-004d)
- Added `docs/schemas/tunnel-listening.schema.json` for IO-008 agent contract
- `scp` honors global `--replace-host-key` and global `--output-format json`

### Security
- If you installed 0.3.9 from crates.io and used `scp`: that release advertised SCP but the wire implementation was inoperant (upload often produced 0-byte remote files or timed out)
- Upgrade to 0.4.0
- No telemetry

### Notes
- One-shot CLI: connect → transfer → disconnect → exit
- Large files: raise `--timeout` (covers connect + full transfer)


## [0.3.9] - 2026-07-15

### Fixed
- Post-0.3.8 audit residuals: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Default tracing level is error (agent-first)
- `-v` enables debug (LOG-001)
- Tunnel/JSON stderr no longer emits INFO progress banners by default (LOG-001)
- Key-only VPS JSON: empty password serializes as `null` instead of the three-asterisk mask (JSON-001)
- `health-check --timeout <ms>` override aligned with exec (CLI-004)
- Product-line docs bumped to 0.3.9 and residual behaviors documented across README, `llms*.txt`, INTEGRATIONS, `docs/*` (HOW_TO_USE, COOKBOOK, MIGRATION, TESTING, CROSS_PLATFORM, AGENTS, schemas), and skills (DOC-003 deep audit)
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
- Inventory `gaps.md` versioned (DOC-001)
- Release checklist `docs/RELEASE_CHECKLIST.md` (PROC-001)

### Security
- Upgrade russh 0.62.2 (security floor ≥0.60.3)
- Remove crypto COMPAT RC pins (DEP-002)
- `cargo deny`: `yanked=deny`, empty ignore list
- Drop dead `Unicode-DFS-2016` allow (DENY-001)
- Install resolve gate requires patched russh
- Allows stable primefield
- crossbeam-epoch ≥0.9.20 (RUSTSEC-2026-0204 / criterion dev)

### Changed
- Version 0.3.7 → 0.3.8
- `scripts/verify_install_resolve.sh` policy inverted (stable crypto allowed, patched russh required)

### Notes
- No telemetry (doctor reports `telemetry: false` only)
- Product fixes from uncommitted 0.3.7 ship in this release commit

### Added
- Full bilingual documentation framework (README, CONTRIBUTING, SECURITY, INTEGRATIONS, docs guides, schemas, skills)
- Dual license files `LICENSE-MIT` and `LICENSE-APACHE` with MIT OR Apache-2.0


## [0.3.7] - 2026-07-15

### Fixed
- All gaps listed in `gaps.md` at that time (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Domain write-path: `validar_e_normalizar`, port 1..=65535, key file exists (VAL-001..003)
- I/O: global `--output-format` on VPS CRUD, `health-check --json`, JSON error envelope, `--quiet` silences human success, `println!` only in `output` (IO-001..005)
- Tunnel `--timeout-ms` covers SSH connect + loop (TUN-001)
- SCP validates local file before connect (SCP-001)
- `vps remove` clears orphan `active`
- Lock file `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`
- Clap conflicts for password/*_stdin
- Completions broken-pipe safe (CLI-001..003)
- Signals tests `#[serial]`
- Help snapshot
- Non-tautological abort pattern test (TEST-001..003)
- Remote command failure uses process exit `EX_GENERAL` (not remote code) (EXIT-001)
- sudo/su password on channel stdin, not remote argv
- Mask is always three asterisks (SEC-001, SEC-002)
- Import redacted UX + `--allow-incomplete` (IMP-001)
- `cargo deny` green with dated russh/crypto pin policy (DEP-001)

### Changed
- Version 0.3.6 → 0.3.7
- Breaking (agent contracts): long secrets no longer show 12+4 chars (always three asterisks)
- Remote non-zero exit maps to process exit `1` with `remote_exit_code` in JSON error envelope
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
- Migrated crate ownership and repository to `danilo-aguiar-br` after previous GitHub account ban (crates.io owner was `ghost_*`)
- `repository` / `homepage` now point to `https://github.com/danilo-aguiar-br/ssh-cli`
- Author metadata updated to `Danilo Aguiar <daniloaguiarbr@proton.me>`
- Removed hosted-runner workflows and pipeline badges: new repository ships without them

### Note
- crates.io already had versions through `0.3.2` from the previous owner account
- This release is the first under the new owner and repository URL


## [0.2.1] - 2026-04-16

### Fixed
- Pin `elliptic-curve = "=0.14.0-rc.30"` to fix `cargo install ssh-cli` failure caused by incompatible `elliptic-curve 0.14.0-rc.31+` being resolved for `p256/p384/p521 0.14.0-rc.8`


## [0.2.0] - 2026-04-15

### Added
- Fix sudo-exec stdin password piping with `printf '%s\n'`
- Runtime overrides: --password, --sudo-password, --timeout flags on exec/sudo-exec/scp/tunnel
- LLM-friendly camelCase aliases (--sudoPassword, --suPassword)


## [0.1.0] - 2026-04-14
- Initial release

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.6...HEAD
[0.5.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.5...v0.5.6
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
