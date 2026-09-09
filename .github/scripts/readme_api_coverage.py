#!/usr/bin/env python3
"""Fail if a public item of the crate is not mentioned in README.md.

The README is compiled as a doctest, so its *code* cannot rot. Its *coverage*
still can: an item added to lib.rs with no README mention is invisible to
readers. This closes that gap.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LIB = ROOT / "src" / "lib.rs"
README = ROOT / "README.md"

ITEM = re.compile(
    r"^pub\s+(?:mod|type|struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)", re.M
)
MACRO = re.compile(r"^macro_rules!\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)


def main() -> int:
    lib = LIB.read_text(encoding="utf-8")
    readme = README.read_text(encoding="utf-8")

    items = sorted(set(ITEM.findall(lib)))
    macros = sorted(set(MACRO.findall(lib)))

    missing = [n for n in items if not re.search(rf"\b{re.escape(n)}\b", readme)]
    missing += [f"{n}!" for n in macros if f"{n}!" not in readme]

    total = len(items) + len(macros)
    print(f"{total} public items, {len(missing)} missing from README.md")
    for name in sorted(set(items) | {m + "!" for m in macros}):
        mark = "MISSING" if name in missing else "ok"
        print(f"  {mark:>7}  {name}")

    if missing:
        print(f"\nNot mentioned in README.md: {', '.join(missing)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
