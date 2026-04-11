# Rustino

Cross-platform native desktop windows with embedded web views, powered by **Rust**. Drop-in replacement for [Photino.NET](https://tryphotino.io).

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

// With icon
RustinoWindow.ShowNotification("Alert", "Something happened", iconPath: "/path/to/icon.png");
```

Uses WinRT Toast (Windows), NSUserNotification (macOS), and D-Bus (Linux).

### State Queries

| Property | Description |
|---|---|
| `IsMinimized` | Whether the window is minimized |
| `IsMaximized` | Whether the window is maximized |
| `IsFullscreen` | Whether the window is in fullscreen |
| `GetPosition()` | Returns `(X, Y)` position |
| `GetSize()` | Returns `(Width, Height)` size |

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

## Samples

- **HelloWorld** — Load a URL in a window
- **HtmlContent** — Load local HTML from a temp file
- **JsInterop** — Bidirectional JavaScript ↔ C# messaging
- **FeatureShowcase** — Interactive demo of events, IPC, window state, and zoom
- **AllOptions** — Every configuration option and runtime operation
- **Reactive** — IObservable streams with System.Reactive operators

## License

MIT — see [LICENSE](LICENSE).

Inspired by and API-compatible with [Photino](https://tryphotino.io), originally created by TryPhotino (Apache-2.0).
