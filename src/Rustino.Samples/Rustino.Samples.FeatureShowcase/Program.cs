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
        case "zoom-in": window.SetZoom(1.5); break;
        case "zoom-out": window.SetZoom(0.75); break;
        case "zoom-reset": window.SetZoom(1.0); break;
        case "close": window.Close(); break;
        case "get-state":
            var (w, h) = window.GetSize();
            var (x, y) = window.GetPosition();
            var state = $"Size: {w}x{h}, Pos: ({x},{y}), Min: {window.IsMinimized}, Max: {window.IsMaximized}, FS: {window.IsFullscreen}";
            window.ExecuteScript($"document.getElementById('state').textContent = {System.Text.Json.JsonSerializer.Serialize(state)}");
            break;
        default:
            window.ExecuteScript($"document.getElementById('state').textContent = {System.Text.Json.JsonSerializer.Serialize("Echo: " + message)}");
            break;
    }
};

window
    .SetTitle("Feature Showcase")
    .SetUseOsDefaultSize(false)
    .SetSize(900, 700)
    .SetResizable(true)
    .SetDevToolsEnabled(true)
    .SetMinSize(400, 300)
    .SetMaxSize(1920, 1080)
    .SetBackgroundColor(26, 26, 46)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Feature Showcase</title>
        <style>
          * { box-sizing: border-box; margin: 0; padding: 0; }
          body { font-family: system-ui, sans-serif; background: #1a1a2e; color: #e0e0e0; padding: 24px; }
          h1 { color: #00d4ff; margin-bottom: 16px; font-size: 1.5rem; }
          h2 { color: #4ecdc4; margin: 16px 0 8px; font-size: 1.1rem; }
          .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
          .section { background: #16213e; border-radius: 8px; padding: 16px; border: 1px solid #333; }
          button { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 0.85rem; margin: 3px; transition: opacity 0.2s; }
          button:hover { opacity: 0.8; }
          .btn-blue { background: #00d4ff; color: #1a1a2e; }
          .btn-green { background: #4ecdc4; color: #1a1a2e; }
          .btn-red { background: #ff6b6b; color: white; }
          .btn-yellow { background: #ffd93d; color: #1a1a2e; }
          #state { margin-top: 12px; padding: 12px; background: #0f3460; border-radius: 6px; font-family: monospace; font-size: 0.85rem; min-height: 40px; }
        </style>
        </head>
        <body>
          <h1>Rustino Feature Showcase</h1>
          <div class="grid">
            <div class="section">
              <h2>Window State</h2>
              <button class="btn-blue" onclick="send('minimize')">Minimize</button>
              <button class="btn-blue" onclick="send('maximize')">Maximize</button>
              <button class="btn-blue" onclick="send('restore')">Restore</button>
              <button class="btn-green" onclick="send('fullscreen')">Fullscreen</button>
              <button class="btn-green" onclick="send('exit-fullscreen')">Exit FS</button>
              <button class="btn-yellow" onclick="send('chromeless')">Chromeless</button>
              <button class="btn-yellow" onclick="send('decorated')">Decorated</button>
            </div>
            <div class="section">
              <h2>Zoom</h2>
              <button class="btn-blue" onclick="send('zoom-in')">Zoom In (150%)</button>
              <button class="btn-blue" onclick="send('zoom-reset')">Reset (100%)</button>
              <button class="btn-blue" onclick="send('zoom-out')">Zoom Out (75%)</button>
            </div>
            <div class="section">
              <h2>Query State</h2>
              <button class="btn-green" onclick="send('get-state')">Get Window State</button>
              <button class="btn-red" onclick="send('close')">Close Window</button>
            </div>
            <div class="section">
              <h2>JS Interop</h2>
              <button class="btn-yellow" onclick="send('hello from JS!')">Send Message</button>
              <button class="btn-yellow" onclick="send(prompt('Enter message:') || '')">Custom Message</button>
            </div>
          </div>
          <div id="state">Click "Get Window State" to see current state...</div>
          <script>
            function send(msg) { window.ipc.postMessage(msg); }
            window.addEventListener('message', function(e) {
              document.getElementById('state').textContent = 'From C#: ' + e.data;
            });
          </script>
        </body>
        </html>
    """));

window.WaitForClose();
