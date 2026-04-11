# Rustino

Cross-platform native desktop windows with embedded web views, powered by **Rust**. 

Inspired by [Photino.NET](https://tryphotino.io).

Rustino replaces Photino's C++ native layer with Rust, using [wry](https://github.com/nicbarker/gaia) for the webview and [tao](https://github.com/nicbarker/gaia) for window management — the same libraries that power [Tauri](https://tauri.app).

## Architecture

```
Your .NET App
    └── RustinoWindow (C#)          ← Rustino.NET
            └── P/Invoke
                └── rustino_native   ← Rustino.Native (Rust cdylib)
                    ├── wry           → WebView2 (Windows)
                    ├── wry           → WKWebView (macOS)
                    └── wry           → WebKitGTK (Linux)
```

## Quick Start

```csharp
using Rustino.NET;

var window = new RustinoWindow();
window
    .SetTitle("My App")
    .SetUseOsDefaultSize(false)
    .SetSize(1280, 800)
    .SetResizable(true)
    .Center()
    .Load(new Uri("https://example.com"));

window.WaitForClose();
```

## Migrating from Photino

The API is identical. Change two things:

```diff
- using Photino.NET;
+ using Rustino.NET;

- var window = new PhotinoWindow();
+ var window = new RustinoWindow();
```

All `.Set*()`, `.Center()`, `.Load()`, and `.WaitForClose()` calls remain the same.

## API Reference

### Configuration (pre-run)

| Method | Description |
|---|---|
| `SetTitle(string)` | Set the window title |
| `SetSize(int, int)` | Set window dimensions in pixels |
| `SetMinSize(int, int)` | Set minimum window size |
| `SetMaxSize(int, int)` | Set maximum window size |
| `SetPosition(int, int)` | Set window position |
| `SetUseOsDefaultSize(bool)` | Use OS default window size |
| `SetResizable(bool)` | Allow/prevent window resizing |
| `SetTopMost(bool)` | Keep window above all others |
| `SetChromeless(bool)` | Remove window decorations (title bar, borders) |
| `SetTransparent(bool)` | Enable transparent background |
| `SetMaximized(bool)` | Start maximized |
| `SetBackgroundColor(r, g, b, a)` | Set webview background color |
| `SetIconFile(string)` | Set window icon from .ico/.png file path |
| `SetIcon(Stream)` | Set window icon from a .NET stream (e.g. embedded resource) |
| `Center()` | Center window on the primary monitor |
| `SetDevToolsEnabled(bool)` | Enable browser developer tools |
| `SetJavascriptClipboardAccessEnabled(bool)` | Allow JS clipboard access |
| `SetIgnoreCertificateErrorsEnabled(bool)` | Ignore SSL certificate errors |
| `SetWebSecurityEnabled(bool)` | Enable/disable web security (CORS, etc.) |
| `SetMediaAutoplayEnabled(bool)` | Allow media to autoplay |
| `SetZoomHotkeysEnabled(bool)` | Enable Ctrl+/- zoom hotkeys |
| `SetUserAgent(string)` | Set custom user agent string |
| `SetUserDataFolder(string)` | Set webview data folder path |
| `AddInitScript(string)` | Add JavaScript to run before page loads |
| `Load(Uri)` / `Load(string)` | Navigate to a URL or local file |
| `LogVerbosity` | Set log verbosity (0 = silent) |

### Runtime (post-run)

| Method | Description |
|---|---|
| `Minimize()` | Minimize the window |
| `Maximize()` | Maximize the window |
| `Restore()` | Restore from minimized/maximized |
| `SetFullscreen(bool)` | Enter/exit fullscreen |
| `SetVisible(bool)` | Show/hide the window |
| `Focus()` | Bring focus to the window |
| `Close()` | Close the window |
| `ExecuteScript(string)` | Evaluate JavaScript in the webview |
| `SendWebMessage(string)` | Post a message to the webview |
| `SetZoom(double)` | Set webview zoom factor |
| `WaitForClose()` | Block until the window is closed |
| `Dispose()` | Release native resources (`RustinoWindow` implements `IDisposable`) |

### Dialogs

Native cross-platform file dialogs (powered by [rfd](https://github.com/PolyMeilex/rfd)):

```csharp
// Open file (single)
string[]? files = window.ShowOpenFileDialog(
    title: "Select an image",
    filters: [new FileFilter("Images", "jpg", "png", "gif")]);

// Open files (multi-select)
string[]? files = window.ShowOpenFileDialog(
    title: "Select files",
    multiSelect: true);

// Save file
string? path = window.ShowSaveFileDialog(
    title: "Save as",
    defaultPath: "document.pdf",
    filters: [new FileFilter("PDF", "pdf")]);

// Select folder
string[]? folders = window.ShowSelectFolderDialog(
    title: "Choose output directory");
```

All dialogs return `null` when canceled. File filters use the format `new FileFilter("Name", "ext1", "ext2", ...)`.

### Notifications

Native cross-platform toast notifications (powered by [notify-rust](https://github.com/hoodie/notify-rust)):

```csharp
// Static — no window instance required
RustinoWindow.ShowNotification("Download Complete", "Your file has been saved.");

// With icon (file path)
RustinoWindow.ShowNotification("Alert", "Something happened", iconPath: "/path/to/icon.png");

// With icon (embedded resource stream)
using var stream = Assembly.GetExecutingAssembly()
    .GetManifestResourceStream("MyApp.notify.png")!;
RustinoWindow.ShowNotification("Alert", "Something happened", stream);
```

Uses WinRT Toast (Windows), NSUserNotification (macOS), and D-Bus (Linux).

### Menus

Native cross-platform application menus and context menus (powered by [muda](https://github.com/nicbarker/gaia)):

```csharp
// Application menu bar
var menu = new RustinoMenu()
    .AddSubmenu("File", file => file
        .AddItem("new", "New", accelerator: "CmdOrCtrl+N")
        .AddItem("open", "Open...", accelerator: "CmdOrCtrl+O")
        .AddSeparator()
        .AddItem("exit", "Exit"))
    .AddSubmenu("Edit", edit => edit
        .AddItem("undo", "Undo", accelerator: "CmdOrCtrl+Z")
        .AddItem("redo", "Redo", accelerator: "CmdOrCtrl+Y")
        .AddSeparator()
        .AddCheckItem("wordwrap", "Word Wrap", isChecked: true))
    .AddSubmenu("Help", help => help
        .AddItem("about", "About"));

window.SetMenu(menu);

// Context menu (right-click)
var ctx = new RustinoMenu()
    .AddItem("cut", "Cut")
    .AddItem("copy", "Copy")
    .AddItem("paste", "Paste");

window.ShowContextMenu(ctx);

// Handle clicks
window.MenuItemClicked += (_, id) => Console.WriteLine($"Clicked: {id}");

// Remove menu bar
window.RemoveMenu();
```

### System Tray

Native cross-platform system tray icon with optional context menu (powered by [tray-icon](https://github.com/nicbarker/gaia)):

```csharp
// Tray icon with tooltip and context menu
var trayMenu = new RustinoMenu()
    .AddItem("show", "Show Window")
    .AddItem("hide", "Hide Window")
    .AddSeparator()
    .AddItem("quit", "Quit");

window.SetTrayIcon("icon.png", tooltip: "My App", menu: trayMenu);

// From embedded resource (Stream)
using var stream = Assembly.GetExecutingAssembly()
    .GetManifestResourceStream("MyApp.tray.png")!;
window.SetTrayIcon(stream, tooltip: "My App", menu: trayMenu);

// Handle tray icon clicks
window.TrayIconClicked += (_, _) => window.SetVisible(true);

// Remove tray icon
window.RemoveTrayIcon();
```

### Monitors

Enumerate connected displays with position, resolution, and DPI scale factor:

```csharp
// Get all monitors
MonitorInfo[] monitors = window.GetMonitors();
foreach (var m in monitors)
    Console.WriteLine($"{m.Name}: {m.Width}x{m.Height} at ({m.X},{m.Y}), scale={m.ScaleFactor}, primary={m.IsPrimary}");

// Get the monitor containing this window
MonitorInfo? current = window.GetCurrentMonitor();

// DPI-aware positioning: center window on a specific monitor
var target = monitors.First(m => !m.IsPrimary);
var (w, h) = window.GetSize();
window.SetPosition(
    target.X + (target.Width - w) / 2,
    target.Y + (target.Height - h) / 2);
```

`MonitorInfo` properties: `Name`, `X`, `Y`, `Width`, `Height`, `ScaleFactor`, `IsPrimary`.

### State Queries

| Property | Description |
|---|---|
| `IsMinimized` | Whether the window is minimized |
| `IsMaximized` | Whether the window is maximized |
| `IsFullscreen` | Whether the window is in fullscreen |
| `GetPosition()` | Returns `(X, Y)` position |
| `GetSize()` | Returns `(Width, Height)` size |
| `GetMonitors()` | Returns all connected `MonitorInfo[]` |
| `GetCurrentMonitor()` | Returns `MonitorInfo?` for the monitor containing the window |

### Events

| Event | Args | Description |
|---|---|---|
| `WindowClosing` | `CancelEventArgs` | Fired before close (set `Cancel = true` to prevent) |
| `WindowClosed` | `EventArgs` | Fired after the window is destroyed |
| `SizeChanged` | `SizeEventArgs` | Fired on resize (`.Width`, `.Height`) |
| `LocationChanged` | `PointEventArgs` | Fired on move (`.X`, `.Y`) |
| `FocusChanged` | `bool` | Fired on focus/blur |
| `WebMessageReceived` | `string` | Fired when JS calls `window.ipc.postMessage(msg)` |
| `PageLoaded` | `PageLoadEventArgs` | Fired on page load start/finish (`.IsStarted`, `.Url`) |
| `Navigating` | `NavigationEventArgs` | Fired before navigation (`.Url`, set `Cancel = true` to block) |
| `MenuItemClicked` | `string` | Fired when a menu item is clicked (the item's ID) |
| `TrayIconClicked` | `EventArgs` | Fired when the system tray icon is clicked |

### Observable Streams (IObservable&lt;T&gt;)

All events are also available as `IObservable<T>` properties for reactive programming (no System.Reactive dependency required):

| Property | Type | Description |
|---|---|---|
| `WhenSizeChanged` | `IObservable<(int Width, int Height)>` | Size change stream |
| `WhenLocationChanged` | `IObservable<(int X, int Y)>` | Position change stream |
| `WhenFocusChanged` | `IObservable<bool>` | Focus/blur stream |
| `WhenWebMessageReceived` | `IObservable<string>` | JS message stream |
| `WhenPageLoaded` | `IObservable<PageLoadEventArgs>` | Page load stream |
| `WhenNavigating` | `IObservable<NavigationEventArgs>` | Navigation stream |
| `WhenWindowClosed` | `IObservable<EventArgs>` | Window closed stream |
| `WhenMenuItemClicked` | `IObservable<string>` | Menu item click stream |
| `WhenTrayIconClicked` | `IObservable<EventArgs>` | Tray icon click stream |

All streams complete automatically when the window closes or is disposed.

### Rustino.NET.Reactive (companion package)

For System.Reactive operators, add the `Rustino.NET.Reactive` package:

```csharp
using System.Reactive.Linq;
using Rustino.NET.Reactive;

// Throttled resize
window.WhenSizeChangedThrottled(TimeSpan.FromMilliseconds(200))
    .Subscribe(size => Console.WriteLine($"{size.Width}x{size.Height}"));

// Message routing by prefix
window.WhenWebMessageWithPrefix("cmd:")
    .Subscribe(cmd => HandleCommand(cmd));

// Page load completion only
window.WhenPageLoadCompleted()
    .Subscribe(e => Console.WriteLine($"Loaded: {e.Url}"));

// Throttled move events
window.WhenLocationChangedThrottled(TimeSpan.FromMilliseconds(100))
    .Subscribe(pos => Console.WriteLine($"Moved to {pos.X},{pos.Y}"));

// Distinct focus changes
window.WhenFocusChangedDistinct()
    .Subscribe(focused => Console.WriteLine($"Focus: {focused}"));
```

## Building from Source

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.80+)
- [.NET SDK](https://dotnet.microsoft.com/download) (10.0+)
- **Windows**: WebView2 runtime (pre-installed on Windows 10/11)
- **macOS**: Xcode Command Line Tools
- **Linux**: `libgtk-3-dev libwebkit2gtk-4.1-dev`

### Build

```bash
# Build the Rust native library
cd src/Rustino.Native
cargo build --release

# Build the .NET wrapper
cd ../Rustino.NET
dotnet build

# Run a sample
cd ../Rustino.Samples/Rustino.Samples.HelloWorld
dotnet run
```

## Cross-Platform Support

| Platform | WebView Engine | Native Library |
|---|---|---|
| Windows x64/ARM64 | WebView2 (Chromium) | `rustino_native.dll` |
| macOS x64/ARM64 | WKWebView (WebKit) | `librustino_native.dylib` |
| Linux x64/ARM64 | WebKitGTK | `librustino_native.so` |

## License

MIT — see [LICENSE](LICENSE).

Inspired by and API-compatible with [Photino](https://tryphotino.io), originally created by TryPhotino (Apache-2.0).
