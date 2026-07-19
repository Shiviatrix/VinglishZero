"""Execution support for the controlled generation study.

The package deliberately contains no model client or benchmark-specific logic.
It executes only commands and immutable inputs declared in a ready manifest.
"""

from .errors import StudyExecutionError
from .manifest import ExecutionManifest, load_manifest
from .scheduler import Run, schedule

__all__ = ["ExecutionManifest", "Run", "StudyExecutionError", "load_manifest", "schedule"]
