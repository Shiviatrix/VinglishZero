"""Portable, deterministic implementation metrics collected from generated artifacts."""

from __future__ import annotations

import re
from pathlib import Path


SOURCE_SUFFIXES = {".py", ".c", ".h", ".cc", ".cpp", ".cxx", ".java", ".rs", ".js", ".ts", ".kt", ".swift"}
FUNCTION_RE = re.compile(r"\b(?:def|fn|func|function|[A-Za-z_][\w:<>]*\s+)\s+([A-Za-z_]\w*)\s*\(")
CLASS_RE = re.compile(r"\b(?:class|struct|enum|interface)\s+[A-Za-z_]\w*")
BRANCH_RE = re.compile(r"\b(?:if|for|while|case|catch|&&|\|\||\?)\b")


def source_files(artifact: Path) -> list[Path]:
    """List supported source files in stable order, excluding generated outputs."""

    return sorted(path for path in artifact.rglob("*") if path.is_file() and path.suffix.lower() in SOURCE_SUFFIXES)


def collect(artifact: Path) -> dict[str, int | float]:
    """Measure source size and structural counts without executing artifact code."""

    files = source_files(artifact)
    loc = blank = comments = classes = functions = branches = documented = 0
    for path in files:
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        loc += len(lines)
        blank += sum(not line.strip() for line in lines)
        comments += sum(line.lstrip().startswith(("#", "//", "/*", "*")) for line in lines)
        text = "\n".join(lines)
        classes += len(CLASS_RE.findall(text))
        function_matches = list(FUNCTION_RE.finditer(text))
        functions += len(function_matches)
        branches += len(BRANCH_RE.findall(text))
        documented += sum(
            1
            for match in function_matches
            if any(marker in text[max(0, match.start() - 300) : match.start()] for marker in ("///", "/**", "#", "\"\"\""))
        )
    complexity = functions + branches
    documentation = 0.0 if not functions else documented * 100.0 / functions
    return {
        "loc": loc,
        "blank_lines": blank,
        "comment_lines": comments,
        "module_count": len({path.parent for path in files}),
        "class_count": classes,
        "function_count": functions,
        "cyclomatic_complexity": complexity,
        "documentation_coverage_percent": documentation,
        "dependency_count": 0,
    }
