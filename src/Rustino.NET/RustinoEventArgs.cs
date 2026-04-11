using System.Text;

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
