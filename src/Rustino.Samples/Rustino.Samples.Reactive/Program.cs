using System.Reactive.Linq;
using Rustino.NET;
using Rustino.NET.Reactive;

var window = new RustinoWindow();

// Core observable: throttled resize logging (no System.Reactive needed for subscribe)
window.WhenSizeChanged
    .Subscribe(size => Console.WriteLine($"[Raw] Resized: {size.Width}x{size.Height}"));

// Reactive extension: throttled resize (uses System.Reactive Throttle operator)
window.WhenSizeChangedThrottled(TimeSpan.FromMilliseconds(300))
    .Subscribe(size => Console.WriteLine($"[Throttled] Resized: {size.Width}x{size.Height}"));

// Reactive extension: page load completion only
window.WhenPageLoadCompleted()
    .Subscribe(e => Console.WriteLine($"[PageLoad] Finished: {e.Url}"));

// Core observable: focus changes
window.WhenFocusChanged
    .Subscribe(focused => Console.WriteLine($"[Focus] {(focused ? "Gained" : "Lost")}"));

// Reactive extension: message routing by prefix
window.WhenWebMessageWithPrefix("cmd:")
    .Subscribe(cmd => Console.WriteLine($"[Command] {cmd}"));

// Core observable: all web messages
window.WhenWebMessageReceived
    .Subscribe(msg =>
    {
        Console.WriteLine($"[Message] {msg}");
        if (msg == "ping")
            window.SendWebMessage("pong!");
    });

// Window closed notification
window.WhenWindowClosed
    .Subscribe(_ => Console.WriteLine("[Window] Closed"));

window
    .SetTitle("Reactive Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(800, 500)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Reactive Sample</title></head>
        <body style="margin:0;font-family:system-ui,sans-serif;display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;background:#1e1e2e;color:#cdd6f4">
          <h1 style="color:#89b4fa;margin-bottom:1rem">Reactive Observable Sample</h1>
          <p style="color:#a6adc8;margin-bottom:2rem">Resize the window, click buttons, and watch the console output.</p>
          <div style="display:flex;gap:10px;margin-bottom:1rem">
            <button onclick="window.ipc.postMessage('ping')" style="padding:10px 20px;border:none;border-radius:6px;background:#89b4fa;color:#1e1e2e;font-size:1rem;cursor:pointer">Send Ping</button>
            <button onclick="window.ipc.postMessage('cmd:reload')" style="padding:10px 20px;border:none;border-radius:6px;background:#a6e3a1;color:#1e1e2e;font-size:1rem;cursor:pointer">Send cmd:reload</button>
            <button onclick="window.ipc.postMessage('cmd:save')" style="padding:10px 20px;border:none;border-radius:6px;background:#f9e2af;color:#1e1e2e;font-size:1rem;cursor:pointer">Send cmd:save</button>
          </div>
          <div id="response" style="padding:15px;background:#313244;border-radius:8px;min-width:400px;text-align:center;border:1px solid #45475a">
            Waiting for messages...
          </div>
          <script>
            window.addEventListener('message', (e) => {
              document.getElementById('response').textContent = 'From C#: ' + e.data;
            });
          </script>
        </body>
        </html>
    """));

window.WaitForClose();
