using System.Text;
using System.Text.Json.Serialization;

namespace Rustino.NET;

public class SizeEventArgs(int width, int height) : EventArgs
{
    public int Width { get; } = width;
    public int Height { get; } = height;
}

public class PointEventArgs(int x, int y) : EventArgs
{
    public int X { get; } = x;
    public int Y { get; } = y;
}

public class NavigationEventArgs(string url) : System.ComponentModel.CancelEventArgs
{
    public string Url { get; } = url;
}

public class PageLoadEventArgs(bool isStarted, string url) : EventArgs
{
    public bool IsStarted { get; } = isStarted;
    public bool IsFinished => !IsStarted;
    public string Url { get; } = url;
}

public class MonitorInfo
{
    [JsonPropertyName("name")]
    public string? Name { get; set; }

    [JsonPropertyName("x")]
    public int X { get; set; }

    [JsonPropertyName("y")]
    public int Y { get; set; }

    [JsonPropertyName("width")]
    public int Width { get; set; }

    [JsonPropertyName("height")]
    public int Height { get; set; }

    [JsonPropertyName("scaleFactor")]
    public double ScaleFactor { get; set; }

    [JsonPropertyName("isPrimary")]
    public bool IsPrimary { get; set; }

    public override string ToString() =>
        $"{Name ?? "Unknown"} ({Width}x{Height} at {X},{Y}, {ScaleFactor:F2}x{(IsPrimary ? ", primary" : "")})";
}

[JsonSerializable(typeof(MonitorInfo))]
[JsonSerializable(typeof(MonitorInfo[]))]
internal partial class MonitorJsonContext : JsonSerializerContext;

public class FileFilter(string name, params string[] extensions)
{
    public string Name { get; } = name;
    public string[] Extensions { get; } = extensions;

    internal static string? Encode(FileFilter[]? filters)
    {
        if (filters is not { Length: > 0 }) return null;
        var sb = new StringBuilder();
        for (var i = 0; i < filters.Length; i++)
        {
            if (i > 0) sb.Append(';');
            sb.Append(filters[i].Name);
            sb.Append('|');
            sb.Append(string.Join(',', filters[i].Extensions));
        }
        return sb.ToString();
    }
}
