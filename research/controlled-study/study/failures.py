"""Deterministic failure taxonomy classification from captured evaluator output."""

from __future__ import annotations


RULES = (
    ("security", ("security", "injection", "unsafe", "vulnerability", "cve")),
    ("concurrency", ("race", "deadlock", "concurrent", "thread")),
    ("memory", ("memory", "leak", "use-after-free", "overflow", "allocation")),
    ("transaction", ("transaction", "rollback", "commit", "atomic")),
    ("parser", ("parse", "syntax", "token", "lexer")),
    ("semantic", ("semantic", "type error", "type mismatch", "invalid value")),
    ("algorithm", ("algorithm", "incorrect result", "wrong result", "assertionerror")),
    ("architecture", ("architecture", "module boundary", "coupling")),
    ("state_management", ("state", "cache", "mutation")),
    ("optimization", ("timeout", "performance", "slow")),
    ("api", ("api", "interface", "endpoint")),
    ("documentation", ("documentation", "docstring", "readme")),
    ("testing", ("test", "coverage", "fixture")),
)


def classify(text: str) -> str:
    """Return the first taxonomy category matched by an ordered public rule set."""

    lower = text.lower()
    for category, markers in RULES:
        if any(marker in lower for marker in markers):
            return category
    return "unknown"
