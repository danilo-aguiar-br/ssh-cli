#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fail if a published Markdown document breaks the project formatting contract.

Why this exists
---------------
Gaps 25 and 37 declared "the formatting linter" as the acceptance criterion for their
own closure, and no such linter existed anywhere in the tree. `rg -i 'linter'` outside
`CHANGELOG*` and `gaps.md` returned zero, and `scripts/` held no Markdown checker. The
format happened to be clean on the day it was measured by hand, which is exactly the
state a gate exists to stop depending on: clean by tidiness, not clean by construction.

A criterion that cannot fail is not a criterion. This script is the missing artifact, so
the rule set has an executable definition instead of a claim.

Scope
-----
The published surface only: root `*.md`, `docs/**/*.md` and `skills/**/*.md`.

`docs_rules/` and `docs-prd/` are raw research corpora copied into the tree, not prose
this project authors. Holding them to the house style would require rewriting source
material, which is why they are out of scope — declared here rather than silently
skipped, because an omission nobody wrote down is indistinguishable from an oversight.

The rules
---------
  1. bold          `**text**`
  2. italic        `*text*`
  3. horizontal    `---` or `***` alone on a line (YAML frontmatter delimiters exempt)
  4. star-bullet   `* item`, since `-` is the only permitted marker
  5. bullet-stop   bullet ending with `.`
  6. bullet-comma  bullet ending with `,`
  7. heading-depth H4 and deeper
  8. h1-count      exactly one H1 per document
  9. h1-spacing    exactly two blank lines after the H1
 10. h2-spacing    exactly two blank lines before an H2
 11. header-gap    zero blank lines between an H2 or H3 and its first bullet
 12. dash-usage    em dash only inside an H3 title
 13. blockquote    decorative `>` — only the opening tagline is allowed
 14. emoji         pictographs in prose

Rules 2, 9, 10, 12 and 13 carry a measured exception, because the published tree
already satisfies them and the exceptions are what make the rule true rather than
aspirational:

  * rule 2 ignores asterisks inside an inline code span, which is how a shell glob or a
    `password/*_stdin` pattern appears in prose
  * rules 9 and 10 are spacing floors the H1 and H2 headings already meet
  * rule 12 permits the dash in `### 0.5.6 — keyring and contract`, which is the
    release-heading idiom used across `docs/`
  * rule 13 permits one tagline blockquote directly after the H1

Contract (agent-native)
-----------------------
  stdout : one final OK line on success. Nothing else, ever.
  stderr : every violation as `path:line:rule:text`, then a GAP verdict per rule.
  exit   : 0 clean, 1 violations found, 2 usage error.

`--selftest` builds fixtures in memory and asserts that every rule fires. A checker
nobody has seen fail is a checker nobody can trust, and the self-test is the proof that
each rule is wired to a real detector rather than to a regex that never matches.

Portability
-----------
Pure Python 3 standard library: no bash, rg, awk or jq, so it runs unchanged on Linux,
macOS and Windows.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

# Directories whose Markdown is imported source material, not authored prose.
OUT_OF_SCOPE = ("docs_rules", "docs-prd")

INLINE_CODE = re.compile(r"`[^`]*`")
BOLD = re.compile(r"\*\*")
ITALIC = re.compile(r"(?<![\w*\\])\*(?=[^\s*])([^*\n]{1,200}?)(?<=[^\s*])\*(?![\w*\\])")
HORIZONTAL = re.compile(r"^\s*(\*\*\*|---)\s*$")
STAR_BULLET = re.compile(r"^\s*\* ")
BULLET = re.compile(r"^\s*-\s+\S")
HEADING = re.compile(r"^(#{1,6}) ")
H1 = re.compile(r"^# ")
H2 = re.compile(r"^## ")
H2_OR_H3 = re.compile(r"^#{2,3} ")
H4_PLUS = re.compile(r"^#{4,} ")
BLOCKQUOTE = re.compile(r"^> ")
FENCE = re.compile(r"^\s*```")

# Pictographic emoji blocks only, plus the variation selector that dresses them up.
#
# Arrows (`→`, `←`, `↔`), comparison operators (`≥`, `≤`, `≠`) and the other math
# symbols are typography, not emoji: they carry meaning in prose like `exit 64 → permanent`
# and the project uses them in bulk. A detector that flags them is a detector nobody
# leaves enabled, so the block list below is deliberately narrow and names its exclusions.
EMOJI = re.compile(
    "["
    "\U0001F000-\U0001F0FF"  # mahjong, dominoes, playing cards
    "\U0001F300-\U0001F5FF"  # misc symbols and pictographs
    "\U0001F600-\U0001F64F"  # emoticons
    "\U0001F680-\U0001F6FF"  # transport and map
    "\U0001F700-\U0001F77F"  # alchemical
    "\U0001F780-\U0001F7FF"  # geometric shapes extended
    "\U0001F800-\U0001F8FF"  # supplemental arrows-C
    "\U0001F900-\U0001F9FF"  # supplemental symbols and pictographs
    "\U0001FA00-\U0001FAFF"  # symbols and pictographs extended-A
    "\U0001FB00-\U0001FBFF"  # legacy computing
    "️"                 # variation selector-16: turns a glyph into an emoji
    "⃣"                 # combining enclosing keycap
    "]"
)

RuleHits = Dict[str, List[str]]


def published_documents(root: Path) -> List[Path]:
    """Every Markdown file in the published surface, sorted for a stable report."""
    found: List[Path] = []
    for pattern in ("*.md", "docs/**/*.md", "skills/**/*.md"):
        for path in sorted(root.glob(pattern)):
            if not path.is_file():
                continue
            parts = path.relative_to(root).parts
            if any(part.startswith(".") for part in parts):
                continue
            if parts and parts[0] in OUT_OF_SCOPE:
                continue
            if path not in found:
                found.append(path)
    found.sort()
    return found


def frontmatter_end(lines: Sequence[str]) -> int:
    """Index of the closing `---` of a YAML frontmatter block, or -1 when absent.

    `skills/*/SKILL.md` opens with frontmatter, whose `---` delimiters are metadata
    fences, not horizontal rules.
    """
    if not lines or lines[0].strip() != "---":
        return -1
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            return index
    return -1


def next_nonblank(lines: Sequence[str], start: int) -> int:
    """Index of the first non-blank line at or after `start`, or `len(lines)`."""
    index = start
    while index < len(lines) and lines[index].strip() == "":
        index += 1
    return index


def blank_run(lines: Sequence[str], start: int) -> int:
    """How many consecutive blank lines begin at `start`."""
    count = 0
    index = start
    while index < len(lines) and lines[index].strip() == "":
        count += 1
        index += 1
    return count


def scan_document(root: Path, path: Path) -> RuleHits:
    """Return every violation of `path`, keyed by rule id as `path:line:text`."""
    rel = path.relative_to(root).as_posix()
    lines = gk.read_text(path).splitlines()
    hits: RuleHits = {}

    def hit(rule: str, number: int, text: str) -> None:
        hits.setdefault(rule, []).append(f"{rel}:{number}:{rule}:{text.strip()[:120]}")

    fm_end = frontmatter_end(lines)
    in_fence = False
    h1_seen = 0
    h1_line = -1
    taglines_seen = 0
    body_started = False

    for index, line in enumerate(lines):
        number = index + 1

        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence or index <= fm_end:
            continue

        # The body starts at the first bullet after the H1. Everything between them is the
        # opening block: badges in `README.md`, the tagline everywhere else.
        if h1_line >= 0 and BULLET.match(line):
            body_started = True

        prose = INLINE_CODE.sub("", line)

        if BOLD.search(prose):
            hit("bold", number, line)
        if ITALIC.search(prose):
            hit("italic", number, line)
        if HORIZONTAL.match(line):
            hit("horizontal", number, line)
        if STAR_BULLET.match(line):
            hit("star-bullet", number, line)
        if H4_PLUS.match(line):
            hit("heading-depth", number, line)
        if EMOJI.search(prose):
            hit("emoji", number, line)

        if BULLET.match(line):
            stripped = line.rstrip()
            if stripped.endswith("."):
                hit("bullet-stop", number, line)
            elif stripped.endswith(","):
                hit("bullet-comma", number, line)

        if H1.match(line):
            h1_seen += 1
            h1_line = index
        if "—" in prose and not line.startswith("### "):
            hit("dash-usage", number, line)

        if BLOCKQUOTE.match(line):
            # Permitted exactly once, as the opening tagline between the H1 and the first
            # bullet. Badge rows sit in that span in `README.md`, so the tagline is not
            # necessarily adjacent to the H1: it is any blockquote before the body starts.
            opens_tagline = h1_line >= 0 and not body_started
            if opens_tagline:
                taglines_seen += 1
                opens_tagline = taglines_seen == 1
            if not opens_tagline:
                hit("blockquote", number, line)

        if H2_OR_H3.match(line):
            gap = blank_run(lines, index + 1)
            following = next_nonblank(lines, index + 1)
            if following < len(lines) and BULLET.match(lines[following]) and gap > 0:
                hit("header-gap", number, line)
        if H2.match(line) and index > 0:
            run = 0
            back = index - 1
            while back >= 0 and lines[back].strip() == "":
                run += 1
                back -= 1
            if back >= 0 and run != 2:
                hit("h2-spacing", number, line)

    if h1_seen != 1:
        hit("h1-count", 1, f"document has {h1_seen} H1 headings")
    if h1_line >= 0:
        following = next_nonblank(lines, h1_line + 1)
        if following < len(lines):
            run = blank_run(lines, h1_line + 1)
            if run != 2:
                hit("h1-spacing", h1_line + 1, f"H1 followed by {run} blank lines")

    return hits


def scan_tree(root: Path) -> Tuple[RuleHits, int]:
    """Merge the violations of every published document, plus the file count."""
    merged: RuleHits = {}
    documents = published_documents(root)
    for path in documents:
        for rule, records in scan_document(root, path).items():
            merged.setdefault(rule, []).extend(records)
    return merged, len(documents)


def scan_text(text: str, rel: str = "fixture.md") -> RuleHits:
    """Scan in-memory Markdown, for the self-test."""
    lines = text.splitlines()
    hits: RuleHits = {}
    fm_end = frontmatter_end(lines)
    in_fence = False
    h1_seen = 0
    h1_line = -1
    taglines_seen = 0
    body_started = False

    def hit(rule: str, number: int) -> None:
        hits.setdefault(rule, []).append(f"{rel}:{number}:{rule}")

    for index, line in enumerate(lines):
        number = index + 1
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence or index <= fm_end:
            continue
        if h1_line >= 0 and BULLET.match(line):
            body_started = True
        prose = INLINE_CODE.sub("", line)
        if BOLD.search(prose):
            hit("bold", number)
        if ITALIC.search(prose):
            hit("italic", number)
        if HORIZONTAL.match(line):
            hit("horizontal", number)
        if STAR_BULLET.match(line):
            hit("star-bullet", number)
        if H4_PLUS.match(line):
            hit("heading-depth", number)
        if EMOJI.search(prose):
            hit("emoji", number)
        if BULLET.match(line):
            stripped = line.rstrip()
            if stripped.endswith("."):
                hit("bullet-stop", number)
            elif stripped.endswith(","):
                hit("bullet-comma", number)
        if H1.match(line):
            h1_seen += 1
            h1_line = index
        if "—" in prose and not line.startswith("### "):
            hit("dash-usage", number)
        if BLOCKQUOTE.match(line):
            opens_tagline = h1_line >= 0 and not body_started
            if opens_tagline:
                taglines_seen += 1
                opens_tagline = taglines_seen == 1
            if not opens_tagline:
                hit("blockquote", number)
        if H2_OR_H3.match(line):
            gap = blank_run(lines, index + 1)
            following = next_nonblank(lines, index + 1)
            if following < len(lines) and BULLET.match(lines[following]) and gap > 0:
                hit("header-gap", number)
        if H2.match(line) and index > 0:
            run = 0
            back = index - 1
            while back >= 0 and lines[back].strip() == "":
                run += 1
                back -= 1
            if back >= 0 and run != 2:
                hit("h2-spacing", number)
    if h1_seen != 1:
        hit("h1-count", 1)
    if h1_line >= 0:
        following = next_nonblank(lines, h1_line + 1)
        if following < len(lines):
            run = blank_run(lines, h1_line + 1)
            if run != 2:
                hit("h1-spacing", h1_line + 1)
    return hits


# One fixture per rule. Each must fire, and the clean control must stay silent.
SELF_TEST_CASES: List[Tuple[str, str, bool]] = [
    ("bold", "# T\n\n\n## S\n- a **b** c\n", True),
    ("italic", "# T\n\n\n## S\n- a *b* c\n", True),
    ("horizontal", "# T\n\n\n## S\n- a\n\n---\n", True),
    ("star-bullet", "# T\n\n\n## S\n* a\n", True),
    ("bullet-stop", "# T\n\n\n## S\n- a.\n", True),
    ("bullet-comma", "# T\n\n\n## S\n- a,\n", True),
    ("heading-depth", "# T\n\n\n## S\n- a\n\n#### D\n- b\n", True),
    ("h1-count", "## S\n- a\n", True),
    ("h1-spacing", "# T\n\n## S\n- a\n", True),
    ("h2-spacing", "# T\n\n\n## A\n- a\n## B\n- b\n", True),
    ("header-gap", "# T\n\n\n## S\n\n- a\n", True),
    ("dash-usage", "# T\n\n\n## S\n- a — b\n", True),
    ("blockquote", "# T\n\n\n## S\n- a\n\n> decorative\n", True),
    ("emoji", "# T\n\n\n## S\n- a \U0001F600 b\n", True),
    # Clean control: every exception in one document.
    (
        "none",
        "# T\n\n\n> tagline\n\n\n## S\n- a `password/*_stdin` b\n\n\n### 0.5.6 — release\n- c\n",
        False,
    ),
    # Frontmatter delimiters are metadata fences, not horizontal rules.
    ("none", "---\nname: x\n---\n\n\n# T\n\n\n## S\n- a\n", False),
    # Asterisks inside a fence are code, not emphasis.
    ("none", "# T\n\n\n## S\n- a\n\n```\nx = **y**\n```\n", False),
]


def run_selftest() -> int:
    """Assert every rule fires on its fixture and the clean controls stay silent."""
    failures = 0
    for want, text, should_fire in SELF_TEST_CASES:
        hits = scan_text(text)
        if should_fire:
            if want not in hits:
                gk.eprint(f"SELFTEST FAIL: rule {want} did not fire on its fixture")
                failures += 1
        elif hits:
            gk.eprint(f"SELFTEST FAIL: clean control fired {sorted(hits)}")
            failures += 1
    if failures:
        return 1
    sys.stdout.write(f"Markdown self-test: OK ({len(SELF_TEST_CASES)} cases)\n")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="check_markdown_format.py",
        description="Fail if a published Markdown document breaks the formatting contract.",
    )
    parser.add_argument(
        "--selftest",
        action="store_true",
        help="prove every rule fires on a fixture instead of scanning the tree",
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    if args.selftest:
        return run_selftest()

    hits, documents = scan_tree(gk.repo_root())
    total = sum(len(records) for records in hits.values())
    for rule in sorted(hits):
        for record in hits[rule]:
            gk.eprint(record)
        gk.eprint(f"GAP: {rule} ({len(hits[rule])} violations)")
    if total:
        gk.eprint(f"markdown format gate: {total} violations in {documents} documents")
        return 1

    sys.stdout.write(f"Markdown format gate: OK ({documents} documents)\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
