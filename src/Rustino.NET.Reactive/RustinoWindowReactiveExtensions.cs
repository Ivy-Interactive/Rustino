using System.Reactive.Linq;

namespace Rustino.NET.Reactive;

public static class RustinoWindowReactiveExtensions
{
    public static IObservable<T> WhenWebMessage<T>(this RustinoWindow window,
        Func<string, T?> deserializer) where T : class
    {
        return window.WhenWebMessageReceived
            .Select(deserializer)
            .Where(msg => msg != null)!;
    }

    public static IObservable<string> WhenWebMessageWithPrefix(this RustinoWindow window,
        string prefix)
    {
        return window.WhenWebMessageReceived
            .Where(msg => msg.StartsWith(prefix, StringComparison.Ordinal))
            .Select(msg => msg[prefix.Length..]);
    }

    public static IObservable<(int Width, int Height)> WhenSizeChangedThrottled(
        this RustinoWindow window, TimeSpan? throttle = null)
    {
        return window.WhenSizeChanged
            .Throttle(throttle ?? TimeSpan.FromMilliseconds(250));
    }

    public static IObservable<(int X, int Y)> WhenLocationChangedThrottled(
        this RustinoWindow window, TimeSpan? throttle = null)
    {
        return window.WhenLocationChanged
            .Throttle(throttle ?? TimeSpan.FromMilliseconds(250));
    }

    public static IObservable<PageLoadEventArgs> WhenPageLoadCompleted(
        this RustinoWindow window)
    {
        return window.WhenPageLoaded
            .Where(e => e.IsFinished);
    }

    public static IObservable<bool> WhenFocusChangedDistinct(this RustinoWindow window)
    {
        return window.WhenFocusChanged.DistinctUntilChanged();
    }
}
