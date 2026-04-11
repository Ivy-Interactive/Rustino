using Rustino.NET;

var window = new RustinoWindow();

window.WebMessageReceived += (_, msg) =>
{
    if (msg == "clear")
    {
        window.ClearBadge();
    }
    else if (int.TryParse(msg, out var count))
    {
        window.SetBadgeCount(count);
    }
};

window
    .SetTitle("Badge Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(600, 500)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Badge</title>
        <style>
          * { box-sizing: border-box; margin: 0; }
          body { font-family: system-ui, sans-serif; display: flex; flex-direction: column; align-items: center; padding: 40px; background: #1a1a2e; color: #e0e0e0; }
          h1 { color: #ff6b6b; margin-bottom: 0.5rem; }
          p { color: #888; margin-bottom: 2rem; text-align: center; }
          .grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; max-width: 400px; width: 100%; }
          button { padding: 14px; border: none; border-radius: 8px; font-size: 1.1rem; font-weight: 700; cursor: pointer; transition: transform 0.1s; }
          button:active { transform: scale(0.93); }
          .num { background: #2d2d4e; color: #e0e0e0; }
          .num:hover { background: #3d3d6e; }
          .clear { background: #ff6b6b; color: white; grid-column: span 2; }
          .clear:hover { background: #ff8585; }
          .big { background: #a78bfa; color: white; grid-column: span 2; }
          .big:hover { background: #b99bff; }
          .info { margin-top: 2rem; color: #666; font-size: 0.85rem; max-width: 400px; text-align: center; line-height: 1.5; }
        </style>
        </head>
        <body>
          <h1>Taskbar Badge</h1>
          <p>Set a badge count on the taskbar icon.<br>On Windows this renders an overlay; on macOS it sets the dock badge.</p>
          <div class="grid">
            <button class="num" onclick="send('1')">1</button>
            <button class="num" onclick="send('2')">2</button>
            <button class="num" onclick="send('3')">3</button>
            <button class="num" onclick="send('5')">5</button>
            <button class="num" onclick="send('10')">10</button>
            <button class="num" onclick="send('42')">42</button>
            <button class="num" onclick="send('99')">99</button>
            <button class="num" onclick="send('100')">100+</button>
            <button class="clear" onclick="send('clear')">Clear Badge</button>
            <button class="big" onclick="simulate()">Simulate Counting</button>
          </div>
          <div class="info">
            On Windows, the badge appears as a red circle with the number overlaid on the taskbar icon.<br>
            On macOS, it uses the native dock badge label. Linux is not yet supported.
          </div>
          <script>
            function send(msg) { window.ipc.postMessage(msg); }
            function simulate() {
              let i = 0;
              const iv = setInterval(() => {
                i++;
                send(String(i));
                if (i >= 12) { clearInterval(iv); }
              }, 500);
            }
          </script>
        </body>
        </html>
    """));

window.WaitForClose();
