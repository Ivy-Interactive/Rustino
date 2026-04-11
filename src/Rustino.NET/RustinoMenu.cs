using System.Text.Json;
using System.Text.Json.Serialization;

namespace Rustino.NET;

public partial class RustinoMenu
{
    private readonly List<MenuItemDef> _items = new();

    public RustinoMenu AddItem(string id, string label, string? accelerator = null, bool enabled = true)
    {
        _items.Add(new MenuItemDef { Type = "normal", Id = id, Label = label, Accelerator = accelerator, Enabled = enabled });
        return this;
    }

    public RustinoMenu AddCheckItem(string id, string label, bool isChecked = false, bool enabled = true)
    {
        _items.Add(new MenuItemDef { Type = "check", Id = id, Label = label, Checked = isChecked, Enabled = enabled });
        return this;
    }

    public RustinoMenu AddSeparator()
    {
        _items.Add(new MenuItemDef { Type = "separator" });
        return this;
    }

    public RustinoMenu AddSubmenu(string label, Action<RustinoMenu> build, bool enabled = true)
    {
        var sub = new RustinoMenu();
        build(sub);
        _items.Add(new MenuItemDef { Type = "submenu", Label = label, Enabled = enabled, Items = sub._items });
        return this;
    }

    internal string ToJson()
    {
        return JsonSerializer.Serialize(_items, MenuJsonContext.Default.ListMenuItemDef);
    }

    private class MenuItemDef
    {
        [JsonPropertyName("type")]
        public string Type { get; set; } = "";

        [JsonPropertyName("id")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? Id { get; set; }

        [JsonPropertyName("label")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? Label { get; set; }

        [JsonPropertyName("accelerator")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? Accelerator { get; set; }

        [JsonPropertyName("enabled")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public bool? Enabled { get; set; }

        [JsonPropertyName("checked")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public bool? Checked { get; set; }

        [JsonPropertyName("items")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public List<MenuItemDef>? Items { get; set; }
    }

    [JsonSerializable(typeof(List<MenuItemDef>))]
    private partial class MenuJsonContext : JsonSerializerContext;
}
