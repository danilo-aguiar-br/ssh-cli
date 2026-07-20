#!/usr/bin/env bash
# G-PROC-SLSA: release attestation helper (one-shot; no telemetry).
# When COSIGN_KEY / SLSA tooling are unavailable, exits 0 with a checklist print.
set -euo pipefail
echo "ssh-cli release attestation checklist"
echo "1. cargo test --lib && cargo clippy --all-targets -- -D warnings"
echo "2. bash scripts/generate_sbom.sh"
echo "3. cargo package --allow-dirty (review)"
echo "4. git tag -a vX.Y.Z && git push origin vX.Y.Z  (only with human approval)"
echo "5. Optional: cosign sign-blob --key \"\$COSIGN_KEY\" target/package/*.crate"
echo "6. Optional: SLSA generator for GitHub Actions provenance on release.yml"
if [[ "${REQUIRE_COSIGN:-}" == "1" ]]; then
  command -v cosign >/dev/null || { echo "cosign missing"; exit 1; }
fi
echo "OK (documentation path; no secrets uploaded)"
