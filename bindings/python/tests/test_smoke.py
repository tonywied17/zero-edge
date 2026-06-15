"""Smoke test: confirms the facade loads, the native core is reachable, and the
MQTT transport surfaces errors as exceptions (no broker required)."""

import asyncio

import pamoja
from pamoja import MqttClient, PamojaError, Qos, version


def test_version_returns_string():
    assert isinstance(version(), str)
    assert version() == pamoja.version()


def test_qos_exposes_protocol_levels():
    assert Qos.AT_LEAST_ONCE.value == "AtLeastOnce"
    assert Qos.AT_MOST_ONCE.value == "AtMostOnce"
    assert Qos.EXACTLY_ONCE.value == "ExactlyOnce"


def test_raw_escape_hatch_exposes_the_native_contract():
    from pamoja import raw

    assert hasattr(raw, "MqttClient")
    assert raw.version() == version()


def test_connect_failure_raises_and_leaves_client_disconnected():
    async def run():
        client = MqttClient(
            client_id="smoke",
            host="127.0.0.1",
            port=47811,
            keep_alive_secs=1,
        )

        assert await client.is_connected() is False

        try:
            await client.connect()
        except PamojaError as err:
            assert "transport error" in str(err)
        else:
            raise AssertionError("connecting to a closed port should raise")

        assert await client.is_connected() is False

    asyncio.run(run())
