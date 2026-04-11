using Rustino.NET;

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

// --- Context menu (shown from JS) ---
var contextMenu = new RustinoMenu()
    .AddItem("ctx-cut", "Cut")
    .AddItem("ctx-copy", "Copy")
    .AddItem("ctx-paste", "Paste")
    .AddSeparator()
    .AddItem("ctx-select-all", "Select All");

// --- System tray ---
var trayMenu = new RustinoMenu()
    .AddItem("tray-show", "Show Window")
    .AddItem("tray-hide", "Hide Window")
    .AddSeparator()
    .AddItem("tray-quit", "Quit");

// --- Handle menu clicks ---
window.MenuItemClicked += (_, id) =>
{
    Console.WriteLine($"[Menu] Clicked: {id}");

    switch (id)
    {
        case "exit" or "tray-quit":
            window.Close();
            break;
        case "tray-show":
            window.SetVisible(true).Focus();
            break;
        case "tray-hide":
            window.SetVisible(false);
            break;
        case "ctx-menu":
            window.ShowContextMenu(contextMenu);
            break;
        default:
            window.ExecuteScript($"document.getElementById('log').textContent += 'Menu: {id}\\n'");
            break;
    }
};

window.TrayIconClicked += (_, _) =>
{
    Console.WriteLine("[Tray] Icon clicked");
    window.SetVisible(true).Focus();
};

window.WebMessageReceived += (_, msg) =>
{
    if (msg == "show-context-menu")
        window.ShowContextMenu(contextMenu);
};

window
    .SetTitle("Menus & Tray Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(900, 600)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Menus</title></head>
        <body style="margin:0;font-family:system-ui,sans-serif;display:flex;flex-direction:column;align-items:center;padding:40px;background:#1a1a2e;color:#e0e0e0">
          <h1 style="color:#00d4ff;margin-bottom:0.5rem">Menus & System Tray</h1>
          <p style="color:#888;margin-bottom:2rem">Use the menu bar above, right-click for context menu, or check the system tray.</p>
          <div style="display:flex;gap:12px;margin-bottom:2rem">
            <button onclick="window.ipc.postMessage('show-context-menu')"
              style="padding:12px 24px;border:none;border-radius:8px;background:#00d4ff;color:#1a1a2e;font-weight:600;font-size:1rem;cursor:pointer">
              Show Context Menu
            </button>
          </div>
          <div style="width:100%;max-width:600px">
            <h3 style="color:#4ecdc4;margin-bottom:0.5rem">Event Log</h3>
            <pre id="log" style="background:#16213e;border:1px solid #333;border-radius:8px;padding:16px;min-height:200px;white-space:pre-wrap;font-size:0.9rem;color:#ccc;overflow-y:auto;max-height:300px">Waiting for menu events...
        </pre>
          </div>
        </body>
        </html>
    """));

window.SetMenu(appMenu);
window.SetTrayIcon(
    Path.Combine(AppContext.BaseDirectory, "tray.png"),
    tooltip: "Rustino Menus Sample",
    menu: trayMenu);

window.WaitForClose();
