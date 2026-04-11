using System.Text.Json;
using Rustino.NET;

var window = new RustinoWindow();

window.WebMessageReceived += (_, msg) =>
{
    switch (msg)
    {
        case "get-monitors":
            var monitors = window.GetMonitors();
            var current = window.GetCurrentMonitor();
            var data = JsonSerializer.Serialize(new
            {
                monitors,
                currentName = current?.Name
            });
            window.ExecuteScript($"updateMonitors({data})");
            break;

        case "move-primary":
            MoveToMonitor(isPrimary: true);
            break;

        default:
            if (msg.StartsWith("move-to:"))
            {
                var idx = int.Parse(msg["move-to:".Length..]);
                MoveToIndex(idx);
            }
            break;
    }
};

void MoveToMonitor(bool isPrimary)
{
    var target = window.GetMonitors().FirstOrDefault(m => m.IsPrimary == isPrimary);
    if (target == null) return;
    var (w, h) = window.GetSize();
    var x = target.X + (target.Width - w) / 2;
    var y = target.Y + (target.Height - h) / 2;
    window.SetPosition(x, y);
}

void MoveToIndex(int idx)
{
    var monitors = window.GetMonitors();
    if (idx < 0 || idx >= monitors.Length) return;
    var target = monitors[idx];
    var (w, h) = window.GetSize();
    var x = target.X + (target.Width - w) / 2;
    var y = target.Y + (target.Height - h) / 2;
    window.SetPosition(x, y);
}

window
    .SetTitle("Multi-Monitor Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(900, 600)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Monitors</title>
        <style>
          * { box-sizing: border-box; margin: 0; }
          body { font-family: system-ui, sans-serif; display: flex; flex-direction: column; align-items: center; padding: 40px; background: #1a1a2e; color: #e0e0e0; }
          h1 { color: #00d4ff; margin-bottom: 0.5rem; }
          p { color: #888; margin-bottom: 1.5rem; }
          button { padding: 10px 20px; border: none; border-radius: 6px; background: #00d4ff; color: #1a1a2e; font-size: 1rem; font-weight: 600; cursor: pointer; }
          button:active { transform: scale(0.97); }
          .refresh { margin-bottom: 2rem; }
          .canvas { position: relative; width: 100%; max-width: 700px; height: 250px; background: #16213e; border: 1px solid #333; border-radius: 8px; margin-bottom: 1.5rem; }
          .monitor { position: absolute; border: 2px solid #4ecdc4; border-radius: 4px; display: flex; flex-direction: column; align-items: center; justify-content: center; font-size: 0.7rem; cursor: pointer; transition: background 0.15s; }
          .monitor:hover { background: rgba(78,205,196,0.2); }
          .monitor.primary { border-color: #00d4ff; }
          .monitor.current { background: rgba(0,212,255,0.15); }
          .details { width: 100%; max-width: 700px; }
          table { width: 100%; border-collapse: collapse; background: #16213e; border-radius: 8px; overflow: hidden; }
          th { background: #0f3460; text-align: left; padding: 10px 14px; color: #00d4ff; font-size: 0.85rem; }
          td { padding: 8px 14px; border-top: 1px solid #333; font-size: 0.85rem; }
          .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; font-weight: 600; margin-left: 6px; }
          .badge-primary { background: #00d4ff; color: #1a1a2e; }
          .badge-current { background: #4ecdc4; color: #1a1a2e; }
          .move-btn { padding: 4px 10px; border: 1px solid #4ecdc4; border-radius: 4px; background: transparent; color: #4ecdc4; font-size: 0.8rem; cursor: pointer; }
          .move-btn:hover { background: #4ecdc4; color: #1a1a2e; }
        </style>
        </head>
        <body>
          <h1>Multi-Monitor Enumeration</h1>
          <p>Detect all connected monitors with position, resolution, and DPI scale factor.</p>
          <button class="refresh" onclick="window.ipc.postMessage('get-monitors')">Refresh Monitors</button>
          <div class="canvas" id="canvas"></div>
          <div class="details">
            <table>
              <thead><tr><th>#</th><th>Name</th><th>Position</th><th>Resolution</th><th>Scale</th><th></th><th></th></tr></thead>
              <tbody id="tbody"><tr><td colspan="7" style="text-align:center;color:#666">Click "Refresh Monitors" to detect displays</td></tr></tbody>
            </table>
          </div>
          <script>
          function updateMonitors(data) {
            const { monitors, currentName } = data;
            // Find bounds for canvas layout
            let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
            monitors.forEach(m => {
              minX = Math.min(minX, m.x);
              minY = Math.min(minY, m.y);
              maxX = Math.max(maxX, m.x + m.width);
              maxY = Math.max(maxY, m.y + m.height);
            });
            const totalW = maxX - minX;
            const totalH = maxY - minY;
            const canvas = document.getElementById('canvas');
            const cw = canvas.clientWidth - 20;
            const ch = canvas.clientHeight - 20;
            const scale = Math.min(cw / totalW, ch / totalH);
            canvas.innerHTML = '';
            monitors.forEach((m, i) => {
              const isCurrent = m.name === currentName;
              const div = document.createElement('div');
              div.className = 'monitor' + (m.isPrimary ? ' primary' : '') + (isCurrent ? ' current' : '');
              div.style.left = (10 + (m.x - minX) * scale) + 'px';
              div.style.top = (10 + (m.y - minY) * scale) + 'px';
              div.style.width = (m.width * scale) + 'px';
              div.style.height = (m.height * scale) + 'px';
              div.innerHTML = '<b>' + (i+1) + '</b><span>' + m.width + 'x' + m.height + '</span><span>' + m.scaleFactor.toFixed(1) + 'x</span>';
              div.onclick = () => window.ipc.postMessage('move-to:' + i);
              canvas.appendChild(div);
            });
            // Table
            const tbody = document.getElementById('tbody');
            tbody.innerHTML = monitors.map((m, i) => {
              const isCurrent = m.name === currentName;
              const badges = (m.isPrimary ? '<span class="badge badge-primary">primary</span>' : '')
                           + (isCurrent ? '<span class="badge badge-current">current</span>' : '');
              return '<tr><td>' + (i+1) + '</td><td>' + (m.name||'—') + badges + '</td>'
                + '<td>' + m.x + ', ' + m.y + '</td>'
                + '<td>' + m.width + ' × ' + m.height + '</td>'
                + '<td>' + m.scaleFactor.toFixed(2) + 'x</td>'
                + '<td><button class="move-btn" onclick="window.ipc.postMessage(\'move-to:' + i + '\')">Move here</button></td></tr>';
            }).join('');
          }
          window.ipc.postMessage('get-monitors');
          </script>
        </body>
        </html>
    """));

window.WaitForClose();
