using Rustino.NET;

var window = new RustinoWindow();

window.WebMessageReceived += (_, msg) =>
{
    switch (msg)
    {
        case "basic":
            RustinoWindow.ShowNotification(
                "Hello from Rustino!",
                "This is a basic notification.");
            break;

        case "detailed":
            RustinoWindow.ShowNotification(
                "Download Complete",
                "Your file report-2026.pdf has been saved to the Downloads folder.");
            break;

        case "alert":
            RustinoWindow.ShowNotification(
                "Build Failed",
                "3 errors found in Program.cs. Check the output window for details.");
            break;

        case "custom":
            var title = "Custom Notification";
            var body = $"Sent at {DateTime.Now:HH:mm:ss} from Rustino.";
            var success = RustinoWindow.ShowNotification(title, body);
            var escaped = System.Text.Json.JsonSerializer.Serialize(
                success ? $"Sent: \"{title}\"" : "Failed to send notification.");
            window.ExecuteScript($"document.getElementById('status').textContent = {escaped}");
            break;
    }
};

window
    .SetTitle("Notifications Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(700, 500)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Notifications</title>
        <style>
          * { box-sizing: border-box; margin: 0; }
          body { font-family: system-ui, sans-serif; display: flex; flex-direction: column; align-items: center; padding: 40px; background: #1a1a2e; color: #e0e0e0; }
          h1 { color: #00d4ff; margin-bottom: 0.5rem; }
          p { color: #888; margin-bottom: 2rem; }
          .buttons { display: flex; flex-direction: column; gap: 12px; width: 100%; max-width: 400px; }
          button { padding: 16px 24px; border: none; border-radius: 8px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: transform 0.1s; text-align: left; }
          button:active { transform: scale(0.97); }
          button span { display: block; font-weight: 400; font-size: 0.85rem; opacity: 0.8; margin-top: 4px; }
          .b1 { background: #00d4ff; color: #1a1a2e; }
          .b2 { background: #4ecdc4; color: #1a1a2e; }
          .b3 { background: #ff6b6b; color: white; }
          .b4 { background: #a78bfa; color: white; }
          #status { margin-top: 2rem; color: #888; font-size: 0.9rem; min-height: 1.2em; }
        </style>
        </head>
        <body>
          <h1>Native Notifications</h1>
          <p>Click a button to send an OS-native toast notification.</p>
          <div class="buttons">
            <button class="b1" onclick="window.ipc.postMessage('basic')">
              Basic Notification
              <span>Simple title and body text</span>
            </button>
            <button class="b2" onclick="window.ipc.postMessage('detailed')">
              Download Complete
              <span>Simulates a file download notification</span>
            </button>
            <button class="b3" onclick="window.ipc.postMessage('alert')">
              Build Failed
              <span>Simulates an error alert notification</span>
            </button>
            <button class="b4" onclick="window.ipc.postMessage('custom')">
              Custom with Timestamp
              <span>Shows current time and reports success/failure</span>
            </button>
          </div>
          <div id="status"></div>
        </body>
        </html>
    """));

window.WaitForClose();
