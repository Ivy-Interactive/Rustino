using Rustino.NET;

var html = Path.Combine(Path.GetTempPath(), $"rustino_sample_{Guid.NewGuid():N}.html");
File.WriteAllText(html, """
    <!DOCTYPE html>
    <html>
    <head><meta charset="utf-8"><title>Rustino HTML Sample</title></head>
    <body style="margin:0;font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;background:#f8f8f8">
      <div style="text-align:center">
        <h1 style="font-size:2.5rem;margin-bottom:0.5rem">Hello from Rustino!</h1>
        <p style="color:#666;font-size:1.1rem">This HTML was loaded from a local temp file.</p>
      </div>
    </body>
    </html>
    """);

try
{
    var window = new RustinoWindow();
    window
        .SetTitle("HTML Content Sample")
        .SetUseOsDefaultSize(false)
        .SetSize(800, 600)
        .Center()
        .Load(new Uri($"file://{html}"));

    window.WaitForClose();
}
finally
{
    try { File.Delete(html); } catch { }
}
