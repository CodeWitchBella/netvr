using System.Text.Json;

interface IIsblNetFeature
{
    void OnMessage(IsblNet net, JsonElement node);
    void Reset();
    void Tick(IsblNet net);
}
