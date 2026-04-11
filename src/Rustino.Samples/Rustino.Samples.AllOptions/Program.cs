using System.ComponentModel;
using Rustino.NET;

var window = new RustinoWindow();

// --- Event handlers ---

window.WindowClosing += (sender, args) =>
{
    Console.WriteLine("[Event] WindowClosing");
};

window.WindowClosed += (sender, args) =>
{
    Console.WriteLine("[Event] WindowClosed");
};

window.SizeChanged += (sender, args) =>
{
    if (args is SizeEventArgs size)
        Console.WriteLine($"[Event] SizeChanged: {size.Width}x{size.Height}");
};

window.LocationChanged += (sender, args) =>
{
    if (args is PointEventArgs pos)
        Console.WriteLine($"[Event] LocationChanged: ({pos.X}, {pos.Y})");
};

window.FocusChanged += (sender, focused) =>
{
    if (focused is bool f)
        Console.WriteLine($"[Event] FocusChanged: {f}");
};

window.PageLoaded += (sender, args) =>
{
    if (args is PageLoadEventArgs pl)
        Console.WriteLine($"[Event] PageLoad: {(pl.IsStarted ? "Started" : "Finished")} - {pl.Url}");
};

window.WebMessageReceived += (sender, message) =>
{
    Console.WriteLine($"[IPC] Received: {message}");

    switch (message)
    {
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
        case "get-state":
            var (w, h) = window.GetSize();
            var (x, y) = window.GetPosition();
            var state = $"Size: {w}x{h}, Pos: ({x},{y}), Min: {window.IsMinimized}, Max: {window.IsMaximized}, FS: {window.IsFullscreen}";
            window.ExecuteScript($"document.getElementById('log').textContent += '\\n' + {System.Text.Json.JsonSerializer.Serialize(state)}");
            break;
        default:
            window.ExecuteScript($"document.getElementById('log').textContent += '\\nEcho: ' + {System.Text.Json.JsonSerializer.Serialize(message)}");
            break;
    }
};

window
    // Window title and icon
    .SetTitle("All Options Demo")
    .SetIconFile("")

    // Size & constraints
    .SetUseOsDefaultSize(false)
    .SetSize(1024, 768)
    .SetMinSize(400, 300)
    .SetMaxSize(1920, 1080)
    .SetResizable(true)

    // Position
    .Center()

    // Window chrome
    .SetTopMost(false)
    .SetChromeless(false)
    .SetTransparent(false)

    // Background color (dark theme)
    .SetBackgroundColor(26, 26, 46)

    // WebView features
    .SetDevToolsEnabled(true)
    .SetJavascriptClipboardAccessEnabled(false)
    .SetIgnoreCertificateErrorsEnabled(true)
    .SetWebSecurityEnabled(false)
    .SetMediaAutoplayEnabled(true)
    .SetZoomHotkeysEnabled(true)

    // Init script (runs before page content)
    .AddInitScript("""
        window.addEventListener('message', (e) => {
            const el = document.getElementById('log');
            if (el) el.textContent += '\nFrom C#: ' + e.data;
        });
    """)

    // Load inline HTML demonstrating all features
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>All Options Demo</title>
        <style>
          * { box-sizing: border-box; margin: 0; padding: 0; }
          body { font-family: system-ui, sans-serif; background: #1a1a2e; color: #e0e0e0; padding: 20px; }
          h1 { color: #00d4ff; margin-bottom: 12px; font-size: 1.4rem; }
          h2 { color: #4ecdc4; margin: 12px 0 6px; font-size: 1rem; }
          .grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; }
          .section { background: #16213e; border-radius: 8px; padding: 12px; border: 1px solid #333; }
          button { padding: 6px 12px; border: none; border-radius: 5px; cursor: pointer; font-size: 0.8rem; margin: 2px; transition: opacity 0.2s; }
          button:hover { opacity: 0.8; }
          .b { background: #00d4ff; color: #1a1a2e; }
          .g { background: #4ecdc4; color: #1a1a2e; }
          .r { background: #ff6b6b; color: white; }
          .y { background: #ffd93d; color: #1a1a2e; }
          .p { background: #a855f7; color: white; }
          #log { margin-top: 12px; padding: 10px; background: #0f3460; border-radius: 6px; font-family: monospace; font-size: 0.8rem; min-height: 60px; white-space: pre-wrap; max-height: 150px; overflow-y: auto; }
          .info { font-size: 0.75rem; color: #888; margin-top: 8px; }
        </style>
        </head>
        <body>
          <h1>Rustino — All Options Demo</h1>
          <div class="grid">
            <div class="section">
              <h2>Window State</h2>
              <button class="b" onclick="send('minimize')">Minimize</button>
              <button class="b" onclick="send('maximize')">Maximize</button>
              <button class="b" onclick="send('restore')">Restore</button>
              <button class="g" onclick="send('fullscreen')">Fullscreen</button>
              <button class="g" onclick="send('exit-fullscreen')">Exit FS</button>
            </div>
            <div class="section">
              <h2>Window Chrome</h2>
              <button class="y" onclick="send('chromeless')">Chromeless</button>
              <button class="y" onclick="send('decorated')">Decorated</button>
              <button class="p" onclick="send('topmost-on')">Always On Top</button>
              <button class="p" onclick="send('topmost-off')">Normal</button>
            </div>
            <div class="section">
              <h2>Zoom</h2>
              <button class="b" onclick="send('zoom-in')">150%</button>
              <button class="b" onclick="send('zoom-reset')">100%</button>
              <button class="b" onclick="send('zoom-out')">75%</button>
            </div>
            <div class="section">
              <h2>Size & Position</h2>
              <button class="g" onclick="send('size-small')">Small (640x480)</button>
              <button class="g" onclick="send('size-large')">Large (1200x800)</button>
              <button class="g" onclick="send('center')">Center</button>
              <button class="g" onclick="send('move-tl')">Top-Left</button>
            </div>
            <div class="section">
              <h2>JS Interop</h2>
              <button class="y" onclick="send('hello from JS!')">Send Message</button>
              <button class="y" onclick="send(prompt('Enter message:') || '')">Custom Msg</button>
              <button class="y" onclick="send('get-state')">Get State</button>
            </div>
            <div class="section">
              <h2>Lifecycle</h2>
              <button class="r" onclick="send('close')">Close Window</button>
              <p class="info">Events logged to console &amp; panel below</p>
            </div>
          </div>
          <div id="log">Ready. All options configured.</div>
          <script>
            function send(msg) { window.ipc.postMessage(msg); }
          </script>
        </body>
        </html>
    """));

window.WaitForClose();
