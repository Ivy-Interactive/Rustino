namespace Rustino.NET;

internal sealed class EventObservable<T> : IObservable<T>
{
    private readonly object _lock = new();
    private readonly List<IObserver<T>> _observers = new();
    private bool _completed;

    public IDisposable Subscribe(IObserver<T> observer)
    {
        lock (_lock)
        {
            if (_completed)
            {
                observer.OnCompleted();
                return EmptyDisposable.Instance;
            }
            _observers.Add(observer);
        }
        return new Unsubscriber(this, observer);
    }

    internal void Emit(T value)
    {
        IObserver<T>[] snapshot;
        lock (_lock)
        {
            if (_completed) return;
            snapshot = _observers.ToArray();
        }
        foreach (var observer in snapshot)
            observer.OnNext(value);
    }

    internal void Complete()
    {
        IObserver<T>[] snapshot;
        lock (_lock)
        {
            if (_completed) return;
            _completed = true;
            snapshot = _observers.ToArray();
            _observers.Clear();
        }
        foreach (var observer in snapshot)
            observer.OnCompleted();
    }

    private sealed class Unsubscriber(EventObservable<T> parent, IObserver<T> observer) : IDisposable
    {
        public void Dispose()
        {
            lock (parent._lock) { parent._observers.Remove(observer); }
        }
    }

    private sealed class EmptyDisposable : IDisposable
    {
        internal static readonly EmptyDisposable Instance = new();
        public void Dispose() { }
    }
}
