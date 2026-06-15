"""Low-level generated contract for the pamoja core (the escape hatch).

This module re-exports the native :mod:`pamoja._core` extension verbatim. It is
the Python analog of ``@pamoja/core/raw`` in the Node binding: anything the
ergonomic facade does not surface is still reachable here without leaving the SDK.
"""

from . import _core
from ._core import *  # noqa: F401,F403

__all__ = [name for name in dir(_core) if not name.startswith("_")]
