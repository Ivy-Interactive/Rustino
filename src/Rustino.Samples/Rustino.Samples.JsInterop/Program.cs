using Rustino.NET;

var window = new RustinoWindow();

window.WebMessageReceived += (sender, message) =>
{
    Console.WriteLine($"[C#] Received from JS: {message}");

    if (message == "ping")
        window.SendWebMessage("pong from C#!");
    else if (message == "get-time")
        window.ExecuteScript($"document.getElementById('response').textContent = {System.Text.Json.JsonSerializer.Serialize($"Server time: {DateTime.Now:HH:mm:ss}")}");
    else
        window.ExecuteScript($"document.getElementById('response').textContent = {System.Text.Json.JsonSerializer.Serialize("Echo: " + message)}");
};

window
    .SetTitle("JS Interop Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(800, 500)
    .SetDevToolsEnabled(true)
    .Center()
    .AddInitScript("""
        window.addEventListener('message', (e) => {
            const el = document.getElementById('response');
            if (el) el.textContent = 'From C#: ' + e.data;
        });
    """)
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>JS Interop</title></head>
        <body style="margin:0;font-family:system-ui,sans-serif;display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;background:#1a1a2e;color:#e0e0e0">
          <h1 style="color:#00d4ff;margin-bottom:1rem">JS ↔ C# Interop</h1>
          <div style="display:flex;gap:10px;margin-bottom:1rem">
            <button onclick="window.ipc.postMessage('ping')" style="padding:10px 20px;border:none;border-radius:6px;background:#00d4ff;color:#1a1a2e;font-size:1rem;cursor:pointer">Send Ping</button>
            <button onclick="window.ipc.postMessage('get-time')" style="padding:10px 20px;border:none;border-radius:6px;background:#ff6b6b;color:white;font-size:1rem;cursor:pointer">Get Server Time</button>
            <button onclick="window.ipc.postMessage(document.getElementById('custom').value)" style="padding:10px 20px;border:none;border-radius:6px;background:#4ecdc4;color:#1a1a2e;font-size:1rem;cursor:pointer">Send Custom</button>
          </div>
          <input id="custom" type="text" value="Hello from JS!" placeholder="Type a message..."
            style="padding:10px;width:300px;border:1px solid #333;border-radius:6px;background:#16213e;color:#e0e0e0;font-size:1rem;margin-bottom:1rem" />
          <div id="response" style="padding:20px;background:#16213e;border-radius:8px;min-width:400px;text-align:center;font-size:1.1rem;border:1px solid #333">
            Waiting for messages...
          </div>
        </body>
        </html>
    """));

window.WaitForClose();
