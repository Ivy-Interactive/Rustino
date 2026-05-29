using System.Collections.Concurrent;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text.Json;
using Microsoft.Extensions.Logging;

namespace Rustino.NET;

public class RustinoWindow : IDisposable
{
    private IntPtr _nativeHandle;
    private int _disposed;
    private ILogger? _logger;
    private LogCallback? _logCallbackDelegate;
    private GCHandle _logCallbackHandle;

    // Original configuration fields
    private string _title = "Rustino Window";
    private int _width = 800;
    private int _height = 600;
    private bool _resizable = true;
    private bool _topmost;
    private bool _useOsDefaultSize = true;
    private bool _devToolsEnabled;
    private bool _clipboardEnabled;
    private bool _ignoreCertErrors;
    private bool _webSecurityEnabled = true;
    private string? _iconFile;
    private bool _center;

    // New configuration fields
    private bool _transparent;
    private bool _decorations = true;
    private bool _visible = true;
    private bool _maximized;
    private bool _fullscreen;
    private (int X, int Y)? _position;
    private (int Width, int Height)? _minSize;
    private (int Width, int Height)? _maxSize;
    private (byte R, byte G, byte B, byte A)? _backgroundColor;
    private string? _userAgent;
    private string? _userDataFolder;
    private bool _mediaAutoplay = true;
    private bool _zoomHotkeys;
    private readonly List<string> _initScripts = new();

    // Observable streams for reactive consumers
    private readonly EventObservable<(int Width, int Height)> _sizeChangedObs = new();
    private readonly EventObservable<(int X, int Y)> _locationChangedObs = new();
    private readonly EventObservable<bool> _focusChangedObs = new();
    private readonly EventObservable<string> _webMessageObs = new();
    private readonly EventObservable<PageLoadEventArgs> _pageLoadedObs = new();
    private readonly EventObservable<NavigationEventArgs> _navigatingObs = new();
    private readonly EventObservable<EventArgs> _windowClosedObs = new();
    private readonly EventObservable<string> _menuItemClickedObs = new();
    private readonly EventObservable<EventArgs> _trayIconClickedObs = new();

    public int LogVerbosity { get; set; }

    // --- Instance routing for callbacks ---
    private static readonly ConcurrentDictionary<IntPtr, RustinoWindow> Instances = new();

    // Static delegate instances pinned by static fields (prevents GC)
    private static readonly ClosingCallback ClosingCb = OnClosingNative;
    private static readonly VoidContextCallback ClosedCb = OnClosedNative;
    private static readonly SizeCallback ResizedCb = OnResizedNative;
    private static readonly PointCallback MovedCb = OnMovedNative;
    private static readonly IntCallback FocusCb = OnFocusChangedNative;
    private static readonly StringCallback WebMsgCb = OnWebMessageNative;
    private static readonly PageLoadCallback PageLoadCb = OnPageLoadNative;
    private static readonly NavigationCallback NavCb = OnNavigationNative;
    private static readonly StringCallback MenuItemCb = OnMenuItemClickedNative;
    private static readonly VoidContextCallback TrayCb = OnTrayIconClickedNative;

    // --- Logging delegate ---
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void LogCallback(IntPtr context, int level, IntPtr message);

    // --- Events ---

    public event EventHandler<CancelEventArgs>? WindowClosing;
    public event EventHandler? WindowClosed;
    public event EventHandler<SizeEventArgs>? SizeChanged;
    public event EventHandler<PointEventArgs>? LocationChanged;
    public event EventHandler<bool>? FocusChanged;
    public event EventHandler<string>? WebMessageReceived;
    public event EventHandler<PageLoadEventArgs>? PageLoaded;
    public event EventHandler<NavigationEventArgs>? Navigating;
    public event EventHandler<string>? MenuItemClicked;
    public event EventHandler? TrayIconClicked;

    // --- Observable streams ---

    public IObservable<(int Width, int Height)> WhenSizeChanged => _sizeChangedObs;
    public IObservable<(int X, int Y)> WhenLocationChanged => _locationChangedObs;
    public IObservable<bool> WhenFocusChanged => _focusChangedObs;
    public IObservable<string> WhenWebMessageReceived => _webMessageObs;
    public IObservable<PageLoadEventArgs> WhenPageLoaded => _pageLoadedObs;
    public IObservable<NavigationEventArgs> WhenNavigating => _navigatingObs;
    public IObservable<EventArgs> WhenWindowClosed => _windowClosedObs;
    public IObservable<string> WhenMenuItemClicked => _menuItemClickedObs;
    public IObservable<EventArgs> WhenTrayIconClicked => _trayIconClickedObs;

    // --- Notifications (static — no window required) ---

    public static bool ShowNotification(string title, string body, string? iconPath = null, string? appId = null)
    {
        NativeLibraryResolver.EnsureRegistered();
        return RustinoDllImports.rustino_show_notification(title, body, iconPath, appId) != 0;
    }

    public static bool ShowNotification(string title, string body, Stream icon, string? appId = null)
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"rustino_notify_{Guid.NewGuid():N}.png");
        try
        {
            using (var fs = File.Create(tempPath))
                icon.CopyTo(fs);
            return ShowNotification(title, body, tempPath, appId);
        }
        finally
        {
            try { File.Delete(tempPath); } catch { }
        }
    }

    // --- Constructor ---

    public RustinoWindow()
    {
        NativeLibraryResolver.EnsureRegistered();
    }

    // --- Logger configuration ---

    public RustinoWindow SetLogger(ILogger logger)
    {
        _logger = logger;
        return this;
    }

    // --- Original builder methods ---

    public RustinoWindow SetUseOsDefaultSize(bool useDefault)
    {
        _useOsDefaultSize = useDefault;
        return this;
    }

    public RustinoWindow SetSize(int width, int height)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(width);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(height);
        _width = width;
        _height = height;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_size(_nativeHandle, width, height);
        return this;
    }

    public RustinoWindow SetTitle(string title)
    {
        _title = title;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_title(_nativeHandle, title);
        return this;
    }

    public RustinoWindow SetResizable(bool resizable)
    {
        _resizable = resizable;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_resizable(_nativeHandle, resizable ? 1 : 0);
        return this;
    }

    public RustinoWindow SetTopMost(bool topMost)
    {
        _topmost = topMost;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_topmost(_nativeHandle, topMost ? 1 : 0);
        return this;
    }

    public RustinoWindow SetJavascriptClipboardAccessEnabled(bool enabled)
    {
        _clipboardEnabled = enabled;
        return this;
    }

    public RustinoWindow SetDevToolsEnabled(bool enabled)
    {
        _devToolsEnabled = enabled;
        return this;
    }

    public RustinoWindow SetIgnoreCertificateErrorsEnabled(bool enabled)
    {
        _ignoreCertErrors = enabled;
        return this;
    }

    public RustinoWindow SetWebSecurityEnabled(bool enabled)
    {
        _webSecurityEnabled = enabled;
        return this;
    }

    public RustinoWindow SetIconFile(string path)
    {
        _iconFile = path;
        if (_nativeHandle != IntPtr.Zero)
        {
            RustinoDllImports.rustino_set_icon_file(_nativeHandle, path);
            if (OperatingSystem.IsMacOS())
            {
                SetMacDockIcon(path);
            }
        }
        return this;
    }

    public RustinoWindow SetIcon(Stream icon)
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"rustino_icon_{Guid.NewGuid():N}.png");
        using (var fs = File.Create(tempPath))
            icon.CopyTo(fs);
        return SetIconFile(tempPath);
    }

    public RustinoWindow Center()
    {
        _center = true;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_center(_nativeHandle);
        return this;
    }

    public RustinoWindow Load(Uri uri)
    {
        EnsureNative();
        RustinoDllImports.rustino_navigate_to_url(_nativeHandle, uri.AbsoluteUri);
        return this;
    }

    public RustinoWindow Load(string pathOrUrl)
    {
        EnsureNative();
        if (pathOrUrl.StartsWith("data:text/html,", StringComparison.OrdinalIgnoreCase))
        {
            var html = Uri.UnescapeDataString(pathOrUrl["data:text/html,".Length..]);
            RustinoDllImports.rustino_navigate_to_string(_nativeHandle, html);
        }
        else
        {
            RustinoDllImports.rustino_navigate_to_url(_nativeHandle, pathOrUrl);
        }
        return this;
    }

    // --- New builder methods (Phase 3-5) ---

    public RustinoWindow SetTransparent(bool transparent)
    {
        _transparent = transparent;
        return this;
    }

    public RustinoWindow SetChromeless(bool chromeless)
    {
        _decorations = !chromeless;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_decorations(_nativeHandle, _decorations ? 1 : 0);
        return this;
    }

    public RustinoWindow SetPosition(int x, int y)
    {
        _position = (x, y);
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_position(_nativeHandle, x, y);
        return this;
    }

    public RustinoWindow SetMinSize(int width, int height)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(width);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(height);
        _minSize = (width, height);
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_min_size(_nativeHandle, width, height);
        return this;
    }

    public RustinoWindow SetMaxSize(int width, int height)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(width);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(height);
        _maxSize = (width, height);
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_max_size(_nativeHandle, width, height);
        return this;
    }

    public RustinoWindow SetBackgroundColor(byte r, byte g, byte b, byte a = 255)
    {
        _backgroundColor = (r, g, b, a);
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_background_color(_nativeHandle, r, g, b, a);
        return this;
    }

    public RustinoWindow SetUserAgent(string userAgent)
    {
        _userAgent = userAgent;
        return this;
    }

    public RustinoWindow SetUserDataFolder(string path)
    {
        _userDataFolder = path;
        return this;
    }

    public RustinoWindow SetMediaAutoplayEnabled(bool enabled)
    {
        _mediaAutoplay = enabled;
        return this;
    }

    public RustinoWindow SetZoomHotkeysEnabled(bool enabled)
    {
        _zoomHotkeys = enabled;
        return this;
    }

    public RustinoWindow AddInitScript(string script)
    {
        _initScripts.Add(script);
        return this;
    }

    // --- Window state (post-run operations) ---

    public RustinoWindow SetMaximized(bool maximized)
    {
        _maximized = maximized;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_maximized(_nativeHandle, maximized ? 1 : 0);
        return this;
    }

    public RustinoWindow Minimize()
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_minimized(_nativeHandle, 1);
        return this;
    }

    public RustinoWindow Maximize()
    {
        _maximized = true;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_maximized(_nativeHandle, 1);
        return this;
    }

    public RustinoWindow Restore()
    {
        _maximized = false;
        if (_nativeHandle != IntPtr.Zero)
        {
            RustinoDllImports.rustino_set_minimized(_nativeHandle, 0);
            RustinoDllImports.rustino_set_maximized(_nativeHandle, 0);
        }
        return this;
    }

    public RustinoWindow SetFullscreen(bool fullscreen)
    {
        _fullscreen = fullscreen;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_fullscreen(_nativeHandle, fullscreen ? 1 : 0);
        return this;
    }

    public RustinoWindow SetVisible(bool visible)
    {
        _visible = visible;
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_visible(_nativeHandle, visible ? 1 : 0);
        return this;
    }

    public RustinoWindow Focus()
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_focus(_nativeHandle);
        return this;
    }

    public bool IsMinimized =>
        _nativeHandle != IntPtr.Zero && RustinoDllImports.rustino_is_minimized(_nativeHandle) != 0;

    public bool IsMaximized =>
        _nativeHandle != IntPtr.Zero && RustinoDllImports.rustino_is_maximized(_nativeHandle) != 0;

    public bool IsFullscreen =>
        _nativeHandle != IntPtr.Zero && RustinoDllImports.rustino_is_fullscreen(_nativeHandle) != 0;

    public (int X, int Y) GetPosition()
    {
        if (_nativeHandle == IntPtr.Zero) return (0, 0);
        RustinoDllImports.rustino_get_position(_nativeHandle, out var x, out var y);
        return (x, y);
    }

    public (int Width, int Height) GetSize()
    {
        if (_nativeHandle == IntPtr.Zero) return (_width, _height);
        RustinoDllImports.rustino_get_size(_nativeHandle, out var w, out var h);
        return (w, h);
    }

    // --- WebView operations (post-run) ---

    public RustinoWindow ExecuteScript(string script)
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_evaluate_script(_nativeHandle, script);
        return this;
    }

    public RustinoWindow SendWebMessage(string message)
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_send_web_message(_nativeHandle, message);
        return this;
    }

    public RustinoWindow SetZoom(double factor)
    {
        if (factor <= 0 || !double.IsFinite(factor))
            throw new ArgumentOutOfRangeException(nameof(factor), "Zoom factor must be a positive finite number.");
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_zoom(_nativeHandle, factor);
        return this;
    }

    // --- Monitors (post-run) ---

    public MonitorInfo[] GetMonitors()
    {
        if (_nativeHandle == IntPtr.Zero) return [];
        var ptr = RustinoDllImports.rustino_get_monitors(_nativeHandle);
        var json = ConsumeStringResult(ptr);
        if (json == null) return [];
        return JsonSerializer.Deserialize(json, MonitorJsonContext.Default.MonitorInfoArray) ?? [];
    }

    public MonitorInfo? GetCurrentMonitor()
    {
        if (_nativeHandle == IntPtr.Zero) return null;
        var ptr = RustinoDllImports.rustino_get_current_monitor(_nativeHandle);
        var json = ConsumeStringResult(ptr);
        if (json == null) return null;
        return JsonSerializer.Deserialize(json, MonitorJsonContext.Default.MonitorInfo);
    }

    // --- Menus (post-run) ---

    public RustinoWindow SetMenu(RustinoMenu menu)
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_menu(_nativeHandle, menu.ToJson());
        return this;
    }

    public RustinoWindow RemoveMenu()
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_remove_menu(_nativeHandle);
        return this;
    }

    public RustinoWindow ShowContextMenu(RustinoMenu menu, double? x = null, double? y = null)
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_show_context_menu(
                _nativeHandle, menu.ToJson(), x ?? -1, y ?? -1);
        return this;
    }

    // --- System Tray (post-run) ---

    public RustinoWindow SetTrayIcon(string iconPath, string? tooltip = null, RustinoMenu? menu = null)
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_set_tray_icon(
                _nativeHandle, iconPath, tooltip, menu?.ToJson());
        return this;
    }

    public RustinoWindow SetTrayIcon(Stream icon, string? tooltip = null, RustinoMenu? menu = null)
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"rustino_tray_{Guid.NewGuid():N}.png");
        using (var fs = File.Create(tempPath))
            icon.CopyTo(fs);
        return SetTrayIcon(tempPath, tooltip, menu);
    }

    // --- Badge ---

    public RustinoWindow SetBadgeCount(int? count, string? background = null, string? foreground = null)
    {
        if (_nativeHandle != IntPtr.Zero)
        {
            var (bgR, bgG, bgB) = ParseHexColor(background, 0xE0, 0x1E, 0x5A);
            var (fgR, fgG, fgB) = ParseHexColor(foreground, 0xFF, 0xFF, 0xFF);
            RustinoDllImports.rustino_set_badge_count(_nativeHandle, count ?? 0, bgR, bgG, bgB, fgR, fgG, fgB);
        }
        return this;
    }

    public RustinoWindow ClearBadge()
    {
        return SetBadgeCount(null);
    }

    private static (byte r, byte g, byte b) ParseHexColor(string? hex, byte defaultR, byte defaultG, byte defaultB)
    {
        if (string.IsNullOrEmpty(hex))
            return (defaultR, defaultG, defaultB);
        var s = hex.StartsWith('#') ? hex[1..] : hex;
        if (s.Length == 6 &&
            byte.TryParse(s[0..2], System.Globalization.NumberStyles.HexNumber, null, out var r) &&
            byte.TryParse(s[2..4], System.Globalization.NumberStyles.HexNumber, null, out var g) &&
            byte.TryParse(s[4..6], System.Globalization.NumberStyles.HexNumber, null, out var b))
        {
            return (r, g, b);
        }
        return (defaultR, defaultG, defaultB);
    }

    public RustinoWindow RemoveTrayIcon()
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_remove_tray_icon(_nativeHandle);
        return this;
    }

    // --- Dialogs (post-run) ---

    public string[]? ShowOpenFileDialog(
        string? title = null,
        string? defaultPath = null,
        FileFilter[]? filters = null,
        bool multiSelect = false)
    {
        if (_nativeHandle == IntPtr.Zero) return null;
        var filterStr = FileFilter.Encode(filters);
        var ptr = RustinoDllImports.rustino_show_open_file_dialog(
            _nativeHandle, title, defaultPath, filterStr, multiSelect ? 1 : 0);
        return ConsumePathResult(ptr);
    }

    public string? ShowSaveFileDialog(
        string? title = null,
        string? defaultPath = null,
        FileFilter[]? filters = null)
    {
        if (_nativeHandle == IntPtr.Zero) return null;
        var filterStr = FileFilter.Encode(filters);
        var ptr = RustinoDllImports.rustino_show_save_file_dialog(
            _nativeHandle, title, defaultPath, filterStr);
        return ConsumeStringResult(ptr);
    }

    public string[]? ShowSelectFolderDialog(
        string? title = null,
        string? defaultPath = null,
        bool multiSelect = false)
    {
        if (_nativeHandle == IntPtr.Zero) return null;
        var ptr = RustinoDllImports.rustino_show_select_folder_dialog(
            _nativeHandle, title, defaultPath, multiSelect ? 1 : 0);
        return ConsumePathResult(ptr);
    }

    private static string? ConsumeStringResult(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero) return null;
        var result = Marshal.PtrToStringUTF8(ptr);
        RustinoDllImports.rustino_free_string(ptr);
        return result;
    }

    private static string[]? ConsumePathResult(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero) return null;
        var joined = Marshal.PtrToStringUTF8(ptr);
        RustinoDllImports.rustino_free_string(ptr);
        return joined?.Split('\n', StringSplitOptions.RemoveEmptyEntries);
    }

    // --- Blocking run ---

    public void WaitForClose()
    {
        EnsureNative();
        RegisterCallbacks();

        if (OperatingSystem.IsWindows()
            && Thread.CurrentThread.GetApartmentState() != ApartmentState.STA)
        {
            var thread = new Thread(() =>
                RustinoDllImports.rustino_wait_for_exit(_nativeHandle));
            thread.SetApartmentState(ApartmentState.STA);
            thread.Start();
            thread.Join();
        }
        else if (OperatingSystem.IsMacOS()
            && Thread.CurrentThread.ManagedThreadId != 1)
        {
            throw new InvalidOperationException(
                "On macOS, WaitForClose() must be called from the main thread. " +
                "The AppKit event loop requires the main thread to function correctly.");
        }
        else
        {
            RustinoDllImports.rustino_wait_for_exit(_nativeHandle);
        }

        UnregisterCallbacks();
    }

    public void Close()
    {
        if (_nativeHandle != IntPtr.Zero)
            RustinoDllImports.rustino_close(_nativeHandle);
    }

    // --- Native lifecycle ---

    private void EnsureNative()
    {
        if (_nativeHandle != IntPtr.Zero) return;

        var titlePtr = Marshal.StringToCoTaskMemUTF8(_title);
        var iconPtr = _iconFile != null ? Marshal.StringToCoTaskMemUTF8(_iconFile) : IntPtr.Zero;

        // Setup logging callback if ILogger is provided
        IntPtr logCallbackPtr = IntPtr.Zero;
        IntPtr logContextPtr = IntPtr.Zero;

        if (_logger != null)
        {
            _logCallbackDelegate = OnLogMessageNative;
            _logCallbackHandle = GCHandle.Alloc(this);
            logCallbackPtr = Marshal.GetFunctionPointerForDelegate(_logCallbackDelegate);
            logContextPtr = GCHandle.ToIntPtr(_logCallbackHandle);
        }

        try
        {
            var parameters = new RustinoNativeParameters
            {
                Title = titlePtr,
                IconFile = iconPtr,
                Width = _width,
                Height = _height,
                CenterOnInitialize = _center ? 1 : 0,
                UseOsDefaultSize = _useOsDefaultSize ? 1 : 0,
                Resizable = _resizable ? 1 : 0,
                Topmost = _topmost ? 1 : 0,
                DevToolsEnabled = _devToolsEnabled ? 1 : 0,
                ClipboardEnabled = _clipboardEnabled ? 1 : 0,
                IgnoreCertificateErrors = _ignoreCertErrors ? 1 : 0,
                WebSecurityEnabled = _webSecurityEnabled ? 1 : 0,
                LogVerbosity = LogVerbosity,
                LogCallback = logCallbackPtr,
                LogContext = logContextPtr,
            };

            _nativeHandle = RustinoDllImports.rustino_ctor(ref parameters);

            if (_nativeHandle == IntPtr.Zero)
                throw new InvalidOperationException("Failed to create native Rustino window.");
        }
        finally
        {
            Marshal.FreeCoTaskMem(titlePtr);
            if (iconPtr != IntPtr.Zero) Marshal.FreeCoTaskMem(iconPtr);
        }

        try
        {
            // Apply extended configuration via setters
            if (_transparent)
                RustinoDllImports.rustino_set_transparent(_nativeHandle, 1);
            if (!_decorations)
                RustinoDllImports.rustino_set_decorations(_nativeHandle, 0);
            if (!_visible)
                RustinoDllImports.rustino_set_visible(_nativeHandle, 0);
            if (_maximized)
                RustinoDllImports.rustino_set_maximized(_nativeHandle, 1);
            if (_fullscreen)
                RustinoDllImports.rustino_set_fullscreen(_nativeHandle, 1);
            if (_position is { } pos)
                RustinoDllImports.rustino_set_position(_nativeHandle, pos.X, pos.Y);
            if (_minSize is { } min)
                RustinoDllImports.rustino_set_min_size(_nativeHandle, min.Width, min.Height);
            if (_maxSize is { } max)
                RustinoDllImports.rustino_set_max_size(_nativeHandle, max.Width, max.Height);
            if (_backgroundColor is { } bg)
                RustinoDllImports.rustino_set_background_color(_nativeHandle, bg.R, bg.G, bg.B, bg.A);
            if (_userAgent != null)
                RustinoDllImports.rustino_set_user_agent(_nativeHandle, _userAgent);
            if (_userDataFolder != null)
                RustinoDllImports.rustino_set_user_data_folder(_nativeHandle, _userDataFolder);
            if (!_mediaAutoplay)
                RustinoDllImports.rustino_set_media_autoplay(_nativeHandle, 0);
            if (_zoomHotkeys)
                RustinoDllImports.rustino_set_zoom_hotkeys(_nativeHandle, 1);
            foreach (var script in _initScripts)
                RustinoDllImports.rustino_add_init_script(_nativeHandle, script);

            if (_iconFile != null && OperatingSystem.IsMacOS())
            {
                SetMacDockIcon(_iconFile);
            }
        }
        catch
        {
            RustinoDllImports.rustino_dtor(_nativeHandle);
            _nativeHandle = IntPtr.Zero;
            throw;
        }
    }

    // --- Callback wiring ---

    private void RegisterCallbacks()
    {
        Instances[_nativeHandle] = this;
        RustinoDllImports.rustino_set_callback_context(_nativeHandle, _nativeHandle);
        RustinoDllImports.rustino_set_closing_handler(_nativeHandle, ClosingCb);
        RustinoDllImports.rustino_set_closed_handler(_nativeHandle, ClosedCb);
        RustinoDllImports.rustino_set_resized_handler(_nativeHandle, ResizedCb);
        RustinoDllImports.rustino_set_moved_handler(_nativeHandle, MovedCb);
        RustinoDllImports.rustino_set_focus_changed_handler(_nativeHandle, FocusCb);
        RustinoDllImports.rustino_set_web_message_received_handler(_nativeHandle, WebMsgCb);
        RustinoDllImports.rustino_set_page_load_handler(_nativeHandle, PageLoadCb);
        RustinoDllImports.rustino_set_navigation_handler(_nativeHandle, NavCb);
        RustinoDllImports.rustino_set_menu_event_handler(_nativeHandle, MenuItemCb);
        RustinoDllImports.rustino_set_tray_icon_event_handler(_nativeHandle, TrayCb);
    }

    private void UnregisterCallbacks()
    {
        Instances.TryRemove(_nativeHandle, out _);
    }

    // --- Static native callbacks ---

    private static int OnClosingNative(IntPtr ctx)
    {
        if (Instances.TryGetValue(ctx, out var w) && w.WindowClosing is { } handler)
        {
            var args = new CancelEventArgs();
            handler.Invoke(w, args);
            return args.Cancel ? 1 : 0;
        }
        return 0;
    }

    private static void OnClosedNative(IntPtr ctx)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        w.WindowClosed?.Invoke(w, EventArgs.Empty);
        w._windowClosedObs.Emit(EventArgs.Empty);
        CompleteAllObservables(w);
    }

    private static void OnResizedNative(IntPtr ctx, int width, int height)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        w.SizeChanged?.Invoke(w, new SizeEventArgs(width, height));
        w._sizeChangedObs.Emit((width, height));
    }

    private static void OnMovedNative(IntPtr ctx, int x, int y)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        w.LocationChanged?.Invoke(w, new PointEventArgs(x, y));
        w._locationChangedObs.Emit((x, y));
    }

    private static void OnFocusChangedNative(IntPtr ctx, int focused)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        var isFocused = focused != 0;
        w.FocusChanged?.Invoke(w, isFocused);
        w._focusChangedObs.Emit(isFocused);
    }

    private static void OnWebMessageNative(IntPtr ctx, IntPtr msgPtr)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        var msg = Marshal.PtrToStringUTF8(msgPtr);
        if (msg == null) return;
        w.WebMessageReceived?.Invoke(w, msg);
        w._webMessageObs.Emit(msg);
    }

    private static void OnPageLoadNative(IntPtr ctx, int eventType, IntPtr urlPtr)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        var url = Marshal.PtrToStringUTF8(urlPtr) ?? "";
        var args = new PageLoadEventArgs(eventType == 0, url);
        w.PageLoaded?.Invoke(w, args);
        w._pageLoadedObs.Emit(args);
    }

    private static int OnNavigationNative(IntPtr ctx, IntPtr urlPtr)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return 0;
        var url = Marshal.PtrToStringUTF8(urlPtr) ?? "";
        var args = new NavigationEventArgs(url);
        w.Navigating?.Invoke(w, args);
        w._navigatingObs.Emit(args);
        return args.Cancel ? 1 : 0;
    }

    private static void OnMenuItemClickedNative(IntPtr ctx, IntPtr idPtr)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        var id = Marshal.PtrToStringUTF8(idPtr);
        if (id == null) return;
        w.MenuItemClicked?.Invoke(w, id);
        w._menuItemClickedObs.Emit(id);
    }

    private static void OnTrayIconClickedNative(IntPtr ctx)
    {
        if (!Instances.TryGetValue(ctx, out var w)) return;
        w.TrayIconClicked?.Invoke(w, EventArgs.Empty);
        w._trayIconClickedObs.Emit(EventArgs.Empty);
    }

    private static void OnLogMessageNative(IntPtr ctx, int level, IntPtr messagePtr)
    {
        if (ctx == IntPtr.Zero) return;
        var handle = GCHandle.FromIntPtr(ctx);
        if (handle.Target is not RustinoWindow w || w._logger == null) return;

        var message = Marshal.PtrToStringUTF8(messagePtr);
        if (string.IsNullOrEmpty(message)) return;

        var logLevel = level switch
        {
            0 => LogLevel.Trace,
            1 => LogLevel.Debug,
            2 => LogLevel.Information,
            3 => LogLevel.Warning,
            4 => LogLevel.Error,
            5 => LogLevel.Critical,
            _ => LogLevel.Information
        };

        w._logger.Log(logLevel, message);
    }

    // --- Observable completion ---

    private static void CompleteAllObservables(RustinoWindow w)
    {
        w._sizeChangedObs.Complete();
        w._locationChangedObs.Complete();
        w._focusChangedObs.Complete();
        w._webMessageObs.Complete();
        w._pageLoadedObs.Complete();
        w._navigatingObs.Complete();
        w._windowClosedObs.Complete();
        w._menuItemClickedObs.Complete();
        w._trayIconClickedObs.Complete();
    }

    // --- Dispose ---

    public void Dispose()
    {
        if (Interlocked.CompareExchange(ref _disposed, 1, 0) != 0)
            return;

        CompleteAllObservables(this);

        if (_nativeHandle != IntPtr.Zero)
        {
            Instances.TryRemove(_nativeHandle, out _);
            RustinoDllImports.rustino_dtor(_nativeHandle);
            _nativeHandle = IntPtr.Zero;
        }

        if (_logCallbackHandle.IsAllocated)
        {
            _logCallbackHandle.Free();
        }

        GC.SuppressFinalize(this);
    }

    ~RustinoWindow() => Dispose();

    // --- dynamic macOS Dock Icon / Windows AppId Helpers ---

    [DllImport("/usr/lib/libobjc.A.dylib")]
    private static extern IntPtr objc_getClass(string name);

    [DllImport("/usr/lib/libobjc.A.dylib")]
    private static extern IntPtr sel_registerName(string name);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern IntPtr objc_msgSend(IntPtr receiver, IntPtr selector);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern IntPtr objc_msgSend(IntPtr receiver, IntPtr selector, IntPtr arg);

    [DllImport("shell32.dll", SetLastError = true)]
    private static extern void SetCurrentProcessExplicitAppUserModelID([MarshalAs(UnmanagedType.LPWStr)] string appId);

    private string? _applicationId;

    public RustinoWindow SetApplicationId(string applicationId)
    {
        _applicationId = applicationId;
        if (OperatingSystem.IsWindows() && !IsDotnetTool())
        {
            try
            {
                SetCurrentProcessExplicitAppUserModelID(applicationId);
            }
            catch { }
        }
        return this;
    }

    private static IntPtr CreateNSString(string str)
    {
        IntPtr nsStringClass = objc_getClass("NSString");
        IntPtr stringWithUTF8StringSel = sel_registerName("stringWithUTF8String:");
        IntPtr utf8Ptr = Marshal.StringToCoTaskMemUTF8(str);
        try
        {
            return objc_msgSend(nsStringClass, stringWithUTF8StringSel, utf8Ptr);
        }
        finally
        {
            Marshal.FreeCoTaskMem(utf8Ptr);
        }
    }

    private static void SetMacDockIcon(string iconPath)
    {
        try
        {
            IntPtr nsImageClass = objc_getClass("NSImage");
            if (nsImageClass == IntPtr.Zero) return;

            IntPtr nsStringPath = CreateNSString(iconPath);
            if (nsStringPath == IntPtr.Zero) return;

            IntPtr allocSel = sel_registerName("alloc");
            IntPtr nsImageAllocated = objc_msgSend(nsImageClass, allocSel);
            if (nsImageAllocated == IntPtr.Zero) return;

            IntPtr initSel = sel_registerName("initWithContentsOfFile:");
            IntPtr nsImage = objc_msgSend(nsImageAllocated, initSel, nsStringPath);
            if (nsImage == IntPtr.Zero) return;

            IntPtr nsAppClass = objc_getClass("NSApplication");
            if (nsAppClass == IntPtr.Zero) return;

            IntPtr sharedAppSel = sel_registerName("sharedApplication");
            IntPtr nsApp = objc_msgSend(nsAppClass, sharedAppSel);
            if (nsApp == IntPtr.Zero) return;

            IntPtr setIconSel = sel_registerName("setApplicationIconImage:");
            objc_msgSend(nsApp, setIconSel, nsImage);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Failed to set macOS Dock icon: {ex}");
        }
    }

    private static bool IsDotnetTool()
    {
        var processPath = Environment.ProcessPath ?? string.Empty;
        if (processPath.Contains(".dotnet") && (processPath.Contains("tools") || processPath.Contains("store")))
            return true;
        
        var argv0 = Environment.GetCommandLineArgs().FirstOrDefault() ?? string.Empty;
        if (argv0.Contains(".dotnet") && (argv0.Contains("tools") || argv0.Contains("store")))
            return true;

        return false;
    }
}
