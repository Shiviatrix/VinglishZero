"""Typed, user-facing errors for study execution."""


class StudyExecutionError(RuntimeError):
    """Raised when immutable study execution cannot proceed safely."""
