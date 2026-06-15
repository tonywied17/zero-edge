"""Pamoja device SDK for Python.

This is the ergonomic facade over the native pamoja core: the default import most
users ever touch. It adds idiomatic ergonomics - exceptions for errors, an async
iterator over incoming messages, ``async with`` lifecycle, and keyword
construction - without adding behavior; all real work happens in the Rust core.

The generated low-level contract remains available at :mod:`pamoja.raw`.
"""

from ._core import PamojaError, version
from .mqtt import MqttClient, MqttMessage, Qos

__all__ = ["version", "MqttClient", "MqttMessage", "Qos", "PamojaError"]
