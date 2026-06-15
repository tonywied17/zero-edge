// Smoke test: confirms the facade loads, the native core is reachable, and the
// MQTT transport surfaces errors as exceptions (no broker required).
using Pamoja.Core;

string version = PamojaCore.Version;
Console.WriteLine($"pamoja version: {version}");
Assert(!string.IsNullOrEmpty(version), "version should be a non-empty string");

Assert((int)Qos.AtLeastOnce == 1, "Qos should expose protocol levels");

await using var client = new MqttClient(new MqttClientOptions
{
    ClientId = "smoke",
    Host = "127.0.0.1",
    Port = 47811,
    KeepAliveSecs = 1,
});

Assert(!await client.IsConnectedAsync(), "a fresh client should not be connected");

try
{
    await client.ConnectAsync();
    Fail("connecting to a closed port should throw");
}
catch (PamojaException error)
{
    Assert(
        error.Message.Contains("transport error", StringComparison.Ordinal),
        $"expected a transport error, got: {error.Message}");
}

Assert(!await client.IsConnectedAsync(), "a failed connect should leave the client disconnected");

Console.WriteLine("ok");

static void Assert(bool condition, string message)
{
    if (!condition)
    {
        Fail(message);
    }
}

static void Fail(string message)
{
    Console.Error.WriteLine($"assertion failed: {message}");
    Environment.Exit(1);
}
