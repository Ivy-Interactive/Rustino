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
