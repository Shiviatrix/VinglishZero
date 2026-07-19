"""Crash-safe advisory file locks for independent study-runner processes."""

from __future__ import annotations

import os
import time
from pathlib import Path
from typing import IO

from .errors import StudyExecutionError

try:  # POSIX: advisory locks are automatically released when a process dies.
    import fcntl
except ImportError:  # pragma: no cover - Windows uses msvcrt below.
    fcntl = None  # type: ignore[assignment]

try:  # Windows: locking a byte in a regular file is process-scoped.
    import msvcrt
except ImportError:  # pragma: no cover - POSIX uses fcntl above.
    msvcrt = None  # type: ignore[assignment]


class FileLock:
    """An exclusive, bounded-wait lock backed by an OS-managed file descriptor.

    The lock file may remain after a crash, but the OS releases the advisory
    lock with the dead process. Reopening the file therefore recovers stale
    locks without deleting forensic evidence.
    """

    def __init__(self, path: Path, timeout_seconds: float = 30.0) -> None:
        self.path = path
        self.timeout_seconds = timeout_seconds
        self.handle: IO[str] | None = None

    def __enter__(self) -> "FileLock":
        self.path.parent.mkdir(parents=True, exist_ok=True)
        try:
            self.handle = self.path.open("a+", encoding="utf-8")
            self.handle.seek(0)
            if not self.handle.read(1):
                self.handle.seek(0)
                self.handle.write("0")
                self.handle.flush()
                os.fsync(self.handle.fileno())
            deadline = time.monotonic() + self.timeout_seconds
            while not self._try_acquire():
                if time.monotonic() >= deadline:
                    raise StudyExecutionError(f"timed out acquiring study lock: {self.path}")
                time.sleep(0.05)
            return self
        except (OSError, StudyExecutionError) as error:
            self._close()
            if isinstance(error, StudyExecutionError):
                raise
            raise StudyExecutionError(f"cannot acquire study lock {self.path}: {error}") from error

    def _try_acquire(self) -> bool:
        assert self.handle is not None
        try:
            if fcntl is not None:
                fcntl.flock(self.handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            elif msvcrt is not None:  # pragma: no cover - exercised on Windows.
                self.handle.seek(0)
                msvcrt.locking(self.handle.fileno(), msvcrt.LK_NBLCK, 1)
            else:  # pragma: no cover - Python supports one of these platforms.
                raise StudyExecutionError("no supported cross-process file locking primitive is available")
        except (BlockingIOError, OSError):
            return False
        return True

    def __exit__(self, exc_type: object, exc: object, traceback: object) -> None:
        try:
            if self.handle is not None:
                if fcntl is not None:
                    fcntl.flock(self.handle.fileno(), fcntl.LOCK_UN)
                elif msvcrt is not None:  # pragma: no cover - exercised on Windows.
                    self.handle.seek(0)
                    msvcrt.locking(self.handle.fileno(), msvcrt.LK_UNLCK, 1)
        except OSError as error:
            raise StudyExecutionError(f"cannot release study lock {self.path}: {error}") from error
        finally:
            self._close()

    def _close(self) -> None:
        if self.handle is not None:
            self.handle.close()
            self.handle = None
