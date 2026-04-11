using Rustino.NET;

var window = new RustinoWindow();

window.WebMessageReceived += (_, msg) =>
{
    string? result = null;

    switch (msg)
    {
        case "open-single":
            var file = window.ShowOpenFileDialog(
                title: "Select a File",
                filters: [
                    new FileFilter("Images", "jpg", "jpeg", "png", "gif", "bmp"),
                    new FileFilter("Documents", "pdf", "doc", "docx", "txt"),
                    new FileFilter("All Files", "*")
                ]);
            result = file != null ? string.Join("\n", file) : "(cancelled)";
            break;

        case "open-multi":
            var files = window.ShowOpenFileDialog(
                title: "Select Multiple Files",
                multiSelect: true,
                filters: [new FileFilter("All Files", "*")]);
            result = files != null ? string.Join("\n", files) : "(cancelled)";
            break;

        case "save":
            var savePath = window.ShowSaveFileDialog(
                title: "Save File As",
                defaultPath: "document.txt",
                filters: [
                    new FileFilter("Text Files", "txt"),
                    new FileFilter("CSV Files", "csv"),
                    new FileFilter("All Files", "*")
                ]);
            result = savePath ?? "(cancelled)";
            break;

        case "folder-single":
            var folder = window.ShowSelectFolderDialog(
                title: "Select a Folder");
            result = folder != null ? string.Join("\n", folder) : "(cancelled)";
            break;

        case "folder-multi":
            var folders = window.ShowSelectFolderDialog(
                title: "Select Multiple Folders",
                multiSelect: true);
            result = folders != null ? string.Join("\n", folders) : "(cancelled)";
            break;
    }

    if (result != null)
    {
        var escaped = System.Text.Json.JsonSerializer.Serialize(result);
        window.ExecuteScript($"document.getElementById('result').textContent = {escaped}");
    }
};

window
    .SetTitle("File Dialogs Sample")
    .SetUseOsDefaultSize(false)
    .SetSize(800, 600)
    .SetDevToolsEnabled(true)
    .Center()
    .Load("data:text/html," + Uri.EscapeDataString("""
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"><title>Dialogs</title>
        <style>
          * { box-sizing: border-box; margin: 0; }
          body { font-family: system-ui, sans-serif; display: flex; flex-direction: column; align-items: center; padding: 40px; background: #1a1a2e; color: #e0e0e0; }
          h1 { color: #00d4ff; margin-bottom: 0.5rem; }
          p { color: #888; margin-bottom: 2rem; }
          .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; width: 100%; max-width: 500px; margin-bottom: 2rem; }
          button { padding: 14px 20px; border: none; border-radius: 8px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: transform 0.1s; }
          button:active { transform: scale(0.97); }
          .open { background: #00d4ff; color: #1a1a2e; }
          .save { background: #ff6b6b; color: white; }
          .folder { background: #4ecdc4; color: #1a1a2e; }
          .full { grid-column: 1 / -1; }
          pre { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 16px; width: 100%; max-width: 500px; min-height: 120px; white-space: pre-wrap; word-break: break-all; font-size: 0.9rem; color: #ccc; }
          h3 { color: #4ecdc4; margin-bottom: 0.5rem; width: 100%; max-width: 500px; }
        </style>
        </head>
        <body>
          <h1>Native File Dialogs</h1>
          <p>Click a button to open an OS-native dialog.</p>
          <div class="grid">
            <button class="open" onclick="window.ipc.postMessage('open-single')">Open File</button>
            <button class="open" onclick="window.ipc.postMessage('open-multi')">Open Files (Multi)</button>
            <button class="save" onclick="window.ipc.postMessage('save')">Save File</button>
            <button class="folder" onclick="window.ipc.postMessage('folder-single')">Select Folder</button>
            <button class="folder full" onclick="window.ipc.postMessage('folder-multi')">Select Folders (Multi)</button>
          </div>
          <h3>Result</h3>
          <pre id="result">No dialog opened yet.</pre>
        </body>
        </html>
    """));

window.WaitForClose();
