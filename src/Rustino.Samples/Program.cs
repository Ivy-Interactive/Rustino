using System.ComponentModel;
using System.Reactive.Linq;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Rustino.NET;
using Rustino.NET.Reactive;

// --- Logging ---
using var loggerFactory = LoggerFactory.Create(b => b.AddConsole().SetMinimumLevel(LogLevel.Warning));
var logger = loggerFactory.CreateLogger("Rustino");
var iconPath = Path.Combine(AppContext.BaseDirectory, "icon.png");
var menuInitialized = false;

// --- Splashscreen ---
var splashPath = Path.Combine(AppContext.BaseDirectory, "splash.png");
using (var splash = new RustinoSplashscreen(splashPath, 400, 400))
{
    Thread.Sleep(3000);
}

var window = new RustinoWindow();

// --- Application menu bar ---
var appMenu = new RustinoMenu()
    .AddSubmenu("File", file => file
        .AddItem("new", "New", accelerator: "CmdOrCtrl+N")
        .AddItem("open", "Open...", accelerator: "CmdOrCtrl+O")
        .AddItem("save", "Save", accelerator: "CmdOrCtrl+S")
        .AddSeparator()
        .AddItem("exit", "Exit", accelerator: "Alt+F4"))
    .AddSubmenu("Edit", edit => edit
        .AddItem("undo", "Undo", accelerator: "CmdOrCtrl+Z")
        .AddItem("redo", "Redo", accelerator: "CmdOrCtrl+Y")
        .AddSeparator()
        .AddItem("cut", "Cut", accelerator: "CmdOrCtrl+X")
        .AddItem("copy", "Copy", accelerator: "CmdOrCtrl+C")
        .AddItem("paste", "Paste", accelerator: "CmdOrCtrl+V"))
    .AddSubmenu("View", view => view
        .AddCheckItem("sidebar", "Show Sidebar", isChecked: true)
        .AddCheckItem("statusbar", "Show Status Bar", isChecked: true)
        .AddSeparator()
        .AddSubmenu("Theme", theme => theme
            .AddItem("theme-light", "Light")
            .AddItem("theme-dark", "Dark")
            .AddItem("theme-system", "System Default")))
    .AddSubmenu("Help", help => help
        .AddItem("docs", "Documentation")
        .AddSeparator()
        .AddItem("about", "About Rustino"));

var contextMenu = new RustinoMenu()
    .AddItem("ctx-cut", "Cut")
    .AddItem("ctx-copy", "Copy")
    .AddItem("ctx-paste", "Paste")
    .AddSeparator()
    .AddItem("ctx-select-all", "Select All");

var trayMenu = new RustinoMenu()
    .AddItem("tray-show", "Show Window")
    .AddItem("tray-hide", "Hide Window")
    .AddSeparator()
    .AddItem("tray-quit", "Quit");

// --- Events (logged to console) ---
window.WindowClosing += (_, _) => Console.WriteLine("[Event] WindowClosing");
window.WindowClosed += (_, _) => Console.WriteLine("[Event] WindowClosed");
window.SizeChanged += (_, a) => { if (a is SizeEventArgs s) Console.WriteLine($"[Event] SizeChanged: {s.Width}x{s.Height}"); };
window.LocationChanged += (_, a) => { if (a is PointEventArgs p) Console.WriteLine($"[Event] LocationChanged: ({p.X}, {p.Y})"); };
window.FocusChanged += (_, f) => { if (f is bool b) Console.WriteLine($"[Event] FocusChanged: {b}"); };
window.PageLoaded += (_, a) =>
{
    if (a is PageLoadEventArgs pl)
    {
        Console.WriteLine($"[Event] PageLoad: {(pl.IsStarted ? "Started" : "Finished")} - {pl.Url}");
        if (pl.IsFinished && !menuInitialized)
        {
            menuInitialized = true;
            window.SetMenu(appMenu);
            window.SetTrayIcon(iconPath, tooltip: "Rustino Feature Showcase", menu: trayMenu);
        }
    }
};

// --- Reactive observables (logged to console) ---
window.WhenSizeChangedThrottled(TimeSpan.FromMilliseconds(300))
    .Subscribe(s => Console.WriteLine($"[Reactive] Resized: {s.Width}x{s.Height}"));

window.WhenPageLoadCompleted()
    .Subscribe(e => Console.WriteLine($"[Reactive] PageLoad completed: {e.Url}"));

window.WhenFocusChangedDistinct()
    .Subscribe(f => Console.WriteLine($"[Reactive] Focus: {(f ? "Gained" : "Lost")}"));

window.WhenWebMessageWithPrefix("cmd:")
    .Subscribe(cmd => Console.WriteLine($"[Reactive] Command: {cmd}"));

window.WhenWindowClosed
    .Subscribe(_ => Console.WriteLine("[Reactive] Window closed"));

// --- Menu clicks ---
window.MenuItemClicked += (_, id) =>
{
    Console.WriteLine($"[Menu] {id}");
    switch (id)
    {
        case "exit" or "tray-quit": window.Close(); break;
        case "tray-show": window.SetVisible(true).Focus(); break;
        case "tray-hide": window.SetVisible(false); break;
        default: Log($"Menu: {id}"); break;
    }
};

window.TrayIconClicked += (_, _) =>
{
    Console.WriteLine("[Tray] Icon clicked");
    window.SetVisible(true).Focus();
};

// --- IPC message handler ---
window.WebMessageReceived += (_, msg) =>
{
    Console.WriteLine($"[IPC] {msg}");
    HandleMessage(msg);
};

void HandleMessage(string msg)
{
    switch (msg)
    {
        // Window state
        case "minimize": window.Minimize(); break;
        case "maximize": window.Maximize(); break;
        case "restore": window.Restore(); break;
        case "fullscreen": window.SetFullscreen(true); break;
        case "exit-fullscreen": window.SetFullscreen(false); break;
        case "chromeless": window.SetChromeless(true); break;
        case "decorated": window.SetChromeless(false); break;
        case "topmost-on": window.SetTopMost(true); break;
        case "topmost-off": window.SetTopMost(false); break;
        case "zoom-in": window.SetZoom(1.5); break;
        case "zoom-out": window.SetZoom(0.75); break;
        case "zoom-reset": window.SetZoom(1.0); break;
        case "size-small": window.SetSize(640, 480); break;
        case "size-large": window.SetSize(1200, 800); break;
        case "center": window.Center(); break;
        case "move-tl": window.SetPosition(50, 50); break;
        case "close": window.Close(); break;

        // Query state
        case "get-state":
            var (w, h) = window.GetSize();
            var (x, y) = window.GetPosition();
            Log($"Size: {w}x{h}, Pos: ({x},{y}), Min: {window.IsMinimized}, Max: {window.IsMaximized}, FS: {window.IsFullscreen}");
            break;

        // JS interop
        case "ping":
            window.SendWebMessage("pong from C#!");
            break;
        case "get-time":
            Log($"Server time: {DateTime.Now:HH:mm:ss}");
            break;

        // Badges
        case "badge-clear":
            window.ClearBadge();
            Log("Badge cleared");
            break;

        // Dialogs
        case "open-single":
            var file = window.ShowOpenFileDialog(
                title: "Select a File",
                filters: [
                    new FileFilter("Images", "jpg", "jpeg", "png", "gif", "bmp"),
                    new FileFilter("Documents", "pdf", "doc", "docx", "txt"),
                    new FileFilter("All Files", "*")
                ]);
            Log(file != null ? $"Opened: {string.Join(", ", file)}" : "Dialog cancelled");
            break;
        case "open-multi":
            var files = window.ShowOpenFileDialog(title: "Select Multiple Files", multiSelect: true,
                filters: [new FileFilter("All Files", "*")]);
            Log(files != null ? $"Opened {files.Length} file(s):\n{string.Join("\n", files)}" : "Dialog cancelled");
            break;
        case "save":
            var savePath = window.ShowSaveFileDialog(title: "Save File As", defaultPath: "document.txt",
                filters: [new FileFilter("Text Files", "txt"), new FileFilter("CSV Files", "csv"), new FileFilter("All Files", "*")]);
            Log(savePath != null ? $"Save to: {savePath}" : "Dialog cancelled");
            break;
        case "folder-single":
            var folder = window.ShowSelectFolderDialog(title: "Select a Folder");
            Log(folder != null ? $"Folder: {string.Join(", ", folder)}" : "Dialog cancelled");
            break;
        case "folder-multi":
            var folders = window.ShowSelectFolderDialog(title: "Select Multiple Folders", multiSelect: true);
            Log(folders != null ? $"Folders:\n{string.Join("\n", folders)}" : "Dialog cancelled");
            break;

        // Notifications
        case "notify-basic":
            RustinoWindow.ShowNotification("Hello from Rustino!", "This is a basic notification.", appId: "Rustino");
            Log("Notification sent");
            break;
        case "notify-detailed":
            RustinoWindow.ShowNotification("Download Complete", "report-2026.pdf saved to Downloads.", appId: "Rustino");
            Log("Notification sent");
            break;
        case "notify-alert":
            RustinoWindow.ShowNotification("Build Failed", "3 errors found in Program.cs.", appId: "Rustino");
            Log("Notification sent");
            break;

        // Context menu
        case "show-context-menu":
            window.ShowContextMenu(contextMenu);
            break;

        // Monitors
        case "get-monitors":
            var monitors = window.GetMonitors();
            var current = window.GetCurrentMonitor();
            var jsonOpts = new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase };
            var data = JsonSerializer.Serialize(new { monitors, currentName = current?.Name }, jsonOpts);
            window.ExecuteScript($"updateMonitors({data})");
            break;

        // Reactive demo
        case "cmd:reload":
        case "cmd:save":
            Log($"Reactive command received: {msg}");
            break;

        default:
            if (msg.StartsWith("badge:"))
            {
                var parts = msg[6..].Split(':');
                if (int.TryParse(parts[0], out var count))
                {
                    string? bg = parts.Length > 1 ? parts[1] : null;
                    string? fg = parts.Length > 2 ? parts[2] : null;
                    window.SetBadgeCount(count, background: bg, foreground: fg);
                    Log($"Badge set to {count} (bg={bg ?? "default"}, fg={fg ?? "default"})");
                }
            }
            else if (msg.StartsWith("move-to:") && int.TryParse(msg[8..], out var idx))
            {
                var mons = window.GetMonitors();
                if (idx >= 0 && idx < mons.Length)
                {
                    var target = mons[idx];
                    var (sw, sh) = window.GetSize();
                    window.SetPosition(target.X + (target.Width - sw) / 2, target.Y + (target.Height - sh) / 2);
                }
            }
            else
            {
                Log($"Echo: {msg}");
            }
            break;
    }
}

void Log(string text)
{
    var escaped = JsonSerializer.Serialize(text);
    window.ExecuteScript($"appendLog({escaped})");
}

// --- Build & run ---

window
    .SetLogger(logger)
    .SetTitle("Rustino — Feature Showcase")
    .SetIconFile(iconPath)
    .SetUseOsDefaultSize(false)
    .SetSize(1100, 800)
    .SetMinSize(600, 400)
    .SetMaxSize(1920, 1080)
    .SetResizable(true)
    .Center()
    .SetTopMost(false)
    .SetChromeless(false)
    .SetTransparent(false)
    .SetBackgroundColor(26, 26, 46)
    .SetDevToolsEnabled(true)
    .SetJavascriptClipboardAccessEnabled(false)
    .SetIgnoreCertificateErrorsEnabled(true)
    .SetWebSecurityEnabled(false)
    .SetMediaAutoplayEnabled(true)
    .SetZoomHotkeysEnabled(true)
    .AddInitScript("""
        window.addEventListener('message', (e) => {
            appendLog('From C#: ' + e.data);
        });
    """)
    .Load("data:text/html," + Uri.EscapeDataString(Html()));

window.WaitForClose();

static string Html() => """
    <!DOCTYPE html>
    <html>
    <head>
    <meta charset="utf-8">
    <title>Rustino Feature Showcase</title>
    <style>
      * { box-sizing: border-box; margin: 0; padding: 0; }
      body { font-family: system-ui, -apple-system, sans-serif; background: #1a1a2e; color: #e0e0e0; height: 100vh; display: flex; flex-direction: column; overflow: hidden; }

      /* Tab bar */
      .tabs { display: flex; background: #16213e; border-bottom: 1px solid #333; padding: 0 8px; flex-shrink: 0; }
      .tab { padding: 10px 18px; cursor: pointer; font-size: 0.85rem; font-weight: 500; color: #888; border-bottom: 2px solid transparent; transition: all 0.15s; user-select: none; }
      .tab:hover { color: #ccc; }
      .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }

      /* Panels */
      .panels { flex: 1; overflow-y: auto; padding: 20px; }
      .panel { display: none; }
      .panel.active { display: block; }

      /* Shared */
      h2 { color: #00d4ff; font-size: 1.2rem; margin-bottom: 12px; }
      h3 { color: #4ecdc4; font-size: 0.95rem; margin: 14px 0 8px; }
      .card { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 14px; margin-bottom: 12px; }
      .row { display: flex; flex-wrap: wrap; gap: 6px; }
      button { padding: 7px 14px; border: none; border-radius: 5px; cursor: pointer; font-size: 0.82rem; font-weight: 500; transition: opacity 0.15s; }
      button:hover { opacity: 0.8; }
      button:active { transform: scale(0.97); }
      .b { background: #00d4ff; color: #1a1a2e; }
      .g { background: #4ecdc4; color: #1a1a2e; }
      .r { background: #ff6b6b; color: white; }
      .y { background: #ffd93d; color: #1a1a2e; }
      .p { background: #a855f7; color: white; }
      .o { background: #2d2d4e; color: #e0e0e0; }
      .o:hover { background: #3d3d6e; }
      input[type="text"] { padding: 7px 10px; border: 1px solid #333; border-radius: 5px; background: #0f3460; color: #e0e0e0; font-size: 0.85rem; width: 250px; }

      /* Log */
      .log-bar { flex-shrink: 0; border-top: 1px solid #333; background: #0f3460; }
      .log-header { display: flex; align-items: center; justify-content: space-between; padding: 6px 12px; cursor: pointer; user-select: none; }
      .log-header span { font-size: 0.8rem; color: #00d4ff; font-weight: 600; }
      .log-header button { padding: 3px 8px; font-size: 0.7rem; }
      #log { padding: 8px 12px; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 0.78rem; max-height: 150px; overflow-y: auto; white-space: pre-wrap; color: #aaa; display: none; }
      .log-bar.open #log { display: block; }

      /* Monitor canvas */
      .monitor-canvas { position: relative; width: 100%; height: 200px; background: #0f3460; border-radius: 6px; margin-bottom: 12px; }
      .mon { position: absolute; border: 2px solid #4ecdc4; border-radius: 4px; display: flex; flex-direction: column; align-items: center; justify-content: center; font-size: 0.7rem; cursor: pointer; transition: background 0.15s; }
      .mon:hover { background: rgba(78,205,196,0.2); }
      .mon.primary { border-color: #00d4ff; }
      .mon.current { background: rgba(0,212,255,0.15); }
      table { width: 100%; border-collapse: collapse; }
      th { background: #0f3460; text-align: left; padding: 8px 10px; color: #00d4ff; font-size: 0.8rem; }
      td { padding: 6px 10px; border-top: 1px solid #333; font-size: 0.8rem; }
      .badge-tag { display: inline-block; padding: 1px 6px; border-radius: 3px; font-size: 0.7rem; font-weight: 600; margin-left: 4px; }
      .badge-primary { background: #00d4ff; color: #1a1a2e; }
      .badge-current { background: #4ecdc4; color: #1a1a2e; }
      .move-btn { padding: 3px 8px; border: 1px solid #4ecdc4; border-radius: 4px; background: transparent; color: #4ecdc4; font-size: 0.75rem; cursor: pointer; }
      .move-btn:hover { background: #4ecdc4; color: #1a1a2e; }
    </style>
    </head>
    <body>

    <!-- Tab bar -->
    <div class="tabs">
      <div class="tab active" data-tab="window">Window</div>
      <div class="tab" data-tab="interop">JS Interop</div>
      <div class="tab" data-tab="dialogs">Dialogs</div>
      <div class="tab" data-tab="menus">Menus</div>
      <div class="tab" data-tab="notifications">Notifications</div>
      <div class="tab" data-tab="badges">Badges</div>
      <div class="tab" data-tab="monitors">Monitors</div>
      <div class="tab" data-tab="reactive">Reactive</div>
    </div>

    <div class="panels">

      <!-- WINDOW -->
      <div class="panel active" id="tab-window">
        <h2>Window Management</h2>
        <div class="card">
          <h3>State</h3>
          <div class="row">
            <button class="b" onclick="send('minimize')">Minimize</button>
            <button class="b" onclick="send('maximize')">Maximize</button>
            <button class="b" onclick="send('restore')">Restore</button>
            <button class="g" onclick="send('fullscreen')">Fullscreen</button>
            <button class="g" onclick="send('exit-fullscreen')">Exit Fullscreen</button>
          </div>
        </div>
        <div class="card">
          <h3>Chrome</h3>
          <div class="row">
            <button class="y" onclick="send('chromeless')">Chromeless</button>
            <button class="y" onclick="send('decorated')">Decorated</button>
            <button class="p" onclick="send('topmost-on')">Always On Top</button>
            <button class="p" onclick="send('topmost-off')">Normal</button>
          </div>
        </div>
        <div class="card">
          <h3>Zoom</h3>
          <div class="row">
            <button class="b" onclick="send('zoom-in')">150%</button>
            <button class="b" onclick="send('zoom-reset')">100%</button>
            <button class="b" onclick="send('zoom-out')">75%</button>
          </div>
        </div>
        <div class="card">
          <h3>Size & Position</h3>
          <div class="row">
            <button class="g" onclick="send('size-small')">Small (640×480)</button>
            <button class="g" onclick="send('size-large')">Large (1200×800)</button>
            <button class="g" onclick="send('center')">Center</button>
            <button class="g" onclick="send('move-tl')">Top-Left (50,50)</button>
          </div>
        </div>
        <div class="card">
          <h3>Query & Lifecycle</h3>
          <div class="row">
            <button class="b" onclick="send('get-state')">Get Window State</button>
            <button class="r" onclick="send('close')">Close Window</button>
          </div>
        </div>
      </div>

      <!-- JS INTEROP -->
      <div class="panel" id="tab-interop">
        <h2>JavaScript ↔ C# Interop</h2>
        <div class="card">
          <h3>SendWebMessage / WebMessageReceived</h3>
          <div class="row">
            <button class="b" onclick="send('ping')">Send Ping</button>
            <button class="g" onclick="send('get-time')">Get Server Time</button>
          </div>
        </div>
        <div class="card">
          <h3>Custom Message</h3>
          <div class="row">
            <input id="custom-msg" type="text" value="Hello from JS!" />
            <button class="y" onclick="send(document.getElementById('custom-msg').value)">Send</button>
          </div>
        </div>
        <div class="card">
          <h3>ExecuteScript</h3>
          <p style="color:#888;font-size:0.82rem;margin-bottom:8px">C# calls ExecuteScript() to update the log panel below. Init scripts are injected before page load to listen for messages.</p>
        </div>
      </div>

      <!-- DIALOGS -->
      <div class="panel" id="tab-dialogs">
        <h2>Native File Dialogs</h2>
        <div class="card">
          <h3>Open File</h3>
          <div class="row">
            <button class="b" onclick="send('open-single')">Open File</button>
            <button class="b" onclick="send('open-multi')">Open Files (Multi)</button>
          </div>
        </div>
        <div class="card">
          <h3>Save File</h3>
          <div class="row">
            <button class="r" onclick="send('save')">Save File As...</button>
          </div>
        </div>
        <div class="card">
          <h3>Select Folder</h3>
          <div class="row">
            <button class="g" onclick="send('folder-single')">Select Folder</button>
            <button class="g" onclick="send('folder-multi')">Select Folders (Multi)</button>
          </div>
        </div>
      </div>

      <!-- MENUS -->
      <div class="panel" id="tab-menus">
        <h2>Menus & System Tray</h2>
        <div class="card">
          <h3>Application Menu</h3>
          <p style="color:#888;font-size:0.82rem">The menu bar above is set via SetMenu(). It includes submenus, check items, separators, and keyboard accelerators.</p>
        </div>
        <div class="card">
          <h3>Context Menu</h3>
          <div class="row">
            <button class="b" onclick="send('show-context-menu')">Show Context Menu</button>
          </div>
          <p style="color:#888;font-size:0.82rem;margin-top:8px">Shown at cursor position via ShowContextMenu().</p>
        </div>
        <div class="card">
          <h3>System Tray</h3>
          <p style="color:#888;font-size:0.82rem">A tray icon is registered via SetTrayIcon() with a tooltip and menu. Click the tray icon to show the window.</p>
        </div>
      </div>

      <!-- NOTIFICATIONS -->
      <div class="panel" id="tab-notifications">
        <h2>Native Notifications</h2>
        <div class="card">
          <div class="row" style="flex-direction:column;gap:10px">
            <button class="b" onclick="send('notify-basic')" style="text-align:left;padding:12px 16px">
              <b>Basic Notification</b><br><span style="font-weight:400;font-size:0.78rem;opacity:0.8">Simple title and body text</span>
            </button>
            <button class="g" onclick="send('notify-detailed')" style="text-align:left;padding:12px 16px">
              <b>Download Complete</b><br><span style="font-weight:400;font-size:0.78rem;opacity:0.8">Simulates a file download notification</span>
            </button>
            <button class="r" onclick="send('notify-alert')" style="text-align:left;padding:12px 16px">
              <b>Build Failed</b><br><span style="font-weight:400;font-size:0.78rem;opacity:0.8">Simulates an error alert notification</span>
            </button>
          </div>
        </div>
      </div>

      <!-- BADGES -->
      <div class="panel" id="tab-badges">
        <h2>Taskbar Badge</h2>
        <p style="color:#888;font-size:0.85rem;margin-bottom:12px">Set a badge count on the taskbar icon. On Windows this renders an overlay; on macOS it sets the dock badge.</p>
        <div class="card">
          <div class="row" style="align-items:center;margin-bottom:8px">
            <label style="font-size:0.8rem;color:#aaa;margin-right:6px">BG:</label>
            <input id="badge-bg" type="color" value="#E01E5A" style="width:32px;height:24px;border:none;background:none;cursor:pointer">
            <label style="font-size:0.8rem;color:#aaa;margin:0 6px">FG:</label>
            <input id="badge-fg" type="color" value="#FFFFFF" style="width:32px;height:24px;border:none;background:none;cursor:pointer">
          </div>
          <div class="row">
            <button class="o" onclick="sendBadge(1)">1</button>
            <button class="o" onclick="sendBadge(2)">2</button>
            <button class="o" onclick="sendBadge(3)">3</button>
            <button class="o" onclick="sendBadge(5)">5</button>
            <button class="o" onclick="sendBadge(10)">10</button>
            <button class="o" onclick="sendBadge(42)">42</button>
            <button class="o" onclick="sendBadge(99)">99</button>
            <button class="r" onclick="send('badge-clear')">Clear</button>
          </div>
        </div>
        <div class="card">
          <button class="p" onclick="simulateBadge()">Simulate Counting</button>
        </div>
      </div>

      <!-- MONITORS -->
      <div class="panel" id="tab-monitors">
        <h2>Multi-Monitor Enumeration</h2>
        <div class="card">
          <button class="b" onclick="send('get-monitors')">Refresh Monitors</button>
        </div>
        <div class="monitor-canvas" id="mon-canvas"></div>
        <div class="card" style="padding:0;overflow:hidden">
          <table>
            <thead><tr><th>#</th><th>Name</th><th>Position</th><th>Resolution</th><th>Scale</th><th></th></tr></thead>
            <tbody id="mon-tbody"><tr><td colspan="6" style="text-align:center;color:#666">Click "Refresh Monitors"</td></tr></tbody>
          </table>
        </div>
      </div>

      <!-- REACTIVE -->
      <div class="panel" id="tab-reactive">
        <h2>Reactive Observables</h2>
        <p style="color:#888;font-size:0.85rem;margin-bottom:12px">
          Rustino exposes IObservable&lt;T&gt; streams for all events. The Reactive extension library adds throttling, filtering, and completion helpers. Watch the console output as you interact.
        </p>
        <div class="card">
          <h3>Message Routing by Prefix</h3>
          <div class="row">
            <button class="b" onclick="send('cmd:reload')">Send cmd:reload</button>
            <button class="g" onclick="send('cmd:save')">Send cmd:save</button>
          </div>
          <p style="color:#888;font-size:0.78rem;margin-top:8px">WhenWebMessageWithPrefix("cmd:") filters messages by prefix.</p>
        </div>
        <div class="card">
          <h3>Observables Active</h3>
          <ul style="color:#888;font-size:0.82rem;list-style:none;line-height:1.8">
            <li>• WhenSizeChangedThrottled (300ms) — resize the window</li>
            <li>• WhenPageLoadCompleted — fires on page finish</li>
            <li>• WhenFocusChangedDistinct — focus/blur the window</li>
            <li>• WhenWebMessageWithPrefix("cmd:") — filtered messages</li>
            <li>• WhenWindowClosed — fires on close</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Log panel -->
    <div class="log-bar" id="log-bar">
      <div class="log-header" onclick="toggleLog()">
        <span>Log</span>
        <button class="r" onclick="event.stopPropagation();clearLog()">Clear</button>
      </div>
      <div id="log">Ready.</div>
    </div>

    <script>
      function send(msg) { window.ipc.postMessage(msg); }

      // Tabs
      document.querySelectorAll('.tab').forEach(t => {
        t.addEventListener('click', () => {
          document.querySelectorAll('.tab').forEach(x => x.classList.remove('active'));
          document.querySelectorAll('.panel').forEach(x => x.classList.remove('active'));
          t.classList.add('active');
          document.getElementById('tab-' + t.dataset.tab).classList.add('active');
        });
      });

      // Log
      function appendLog(text) {
        const el = document.getElementById('log');
        el.textContent += '\n' + text;
        el.scrollTop = el.scrollHeight;
        const bar = document.getElementById('log-bar');
        if (!bar.classList.contains('open')) bar.classList.add('open');
      }
      function clearLog() { document.getElementById('log').textContent = 'Cleared.'; }
      function toggleLog() { document.getElementById('log-bar').classList.toggle('open'); }

      // Badge
      function sendBadge(n) {
        const bg = document.getElementById('badge-bg').value;
        const fg = document.getElementById('badge-fg').value;
        send('badge:' + n + ':' + bg + ':' + fg);
      }
      function simulateBadge() {
        let i = 0;
        const iv = setInterval(() => { sendBadge(i); i++; if (i >= 12) clearInterval(iv); }, 500);
      }

      // Monitor visualization
      function updateMonitors(data) {
        const { monitors, currentName } = data;
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        monitors.forEach(m => { minX = Math.min(minX, m.x); minY = Math.min(minY, m.y); maxX = Math.max(maxX, m.x + m.width); maxY = Math.max(maxY, m.y + m.height); });
        const canvas = document.getElementById('mon-canvas');
        const cw = canvas.clientWidth - 20, ch = canvas.clientHeight - 20;
        const scale = Math.min(cw / (maxX - minX), ch / (maxY - minY));
        canvas.innerHTML = '';
        monitors.forEach((m, i) => {
          const div = document.createElement('div');
          div.className = 'mon' + (m.isPrimary ? ' primary' : '') + (m.name === currentName ? ' current' : '');
          div.style.cssText = 'left:' + (10 + (m.x - minX) * scale) + 'px;top:' + (10 + (m.y - minY) * scale) + 'px;width:' + (m.width * scale) + 'px;height:' + (m.height * scale) + 'px';
          div.innerHTML = '<b>' + (i+1) + '</b><span>' + m.width + '×' + m.height + '</span><span>' + m.scaleFactor.toFixed(1) + '×</span>';
          div.onclick = () => send('move-to:' + i);
          canvas.appendChild(div);
        });
        document.getElementById('mon-tbody').innerHTML = monitors.map((m, i) => {
          const badges = (m.isPrimary ? '<span class="badge-tag badge-primary">primary</span>' : '') + (m.name === currentName ? '<span class="badge-tag badge-current">current</span>' : '');
          return '<tr><td>' + (i+1) + '</td><td>' + (m.name||'—') + badges + '</td><td>' + m.x + ', ' + m.y + '</td><td>' + m.width + ' × ' + m.height + '</td><td>' + m.scaleFactor.toFixed(2) + '×</td><td><button class="move-btn" onclick="send(\'move-to:' + i + '\')">Move here</button></td></tr>';
        }).join('');
      }

      // Auto-refresh monitors on tab switch
      document.querySelector('[data-tab="monitors"]').addEventListener('click', () => send('get-monitors'));
    </script>
    </body>
    </html>
    """;
