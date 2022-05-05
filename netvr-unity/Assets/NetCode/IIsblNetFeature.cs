using System.Text.Json;

interface IIsblNetFeature
{
    void OnMessage(IsblNet net, JsonElement node);
    void Reset();
    void FixedUpdate(IsblNet net);
    void Update(IsblNet net);
}
