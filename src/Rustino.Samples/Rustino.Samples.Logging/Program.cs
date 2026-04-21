using Microsoft.Extensions.Logging;
using Rustino.NET;

// Example 1: With ILogger - warnings route to console logger
Console.WriteLine("=== Example 1: With ILogger ===");
using var loggerFactory = LoggerFactory.Create(builder =>
{
    builder.AddConsole().SetMinimumLevel(LogLevel.Warning);
});
var logger = loggerFactory.CreateLogger<Program>();

using (var window = new RustinoWindow()
    .SetLogger(logger)
    .SetTitle("Logging Sample - With Logger")
    .SetSize(800, 600)
    .SetWebSecurityEnabled(false)          // On non-Windows, this triggers a warning
    .SetIgnoreCertificateErrorsEnabled(true)) // On non-Windows, this also triggers a warning
{
    window.Load("data:text/html,<h1>Check console for warnings routed via ILogger</h1>");
    Console.WriteLine("Window created with ILogger. On non-Windows platforms, warnings will be logged via ILogger.");
    Console.WriteLine("Press Enter to close and continue...");
    Console.ReadLine();
}

// Example 2: Without ILogger - warnings are suppressed
Console.WriteLine("\n=== Example 2: Without ILogger ===");
using (var window2 = new RustinoWindow()
    .SetTitle("Logging Sample - Without Logger")
    .SetSize(800, 600)
    .SetWebSecurityEnabled(false)          // On non-Windows, no warning
    .SetIgnoreCertificateErrorsEnabled(true)) // On non-Windows, no warning
{
    window2.Load("data:text/html,<h1>No warnings - logger not configured</h1>");
    Console.WriteLine("Window created without ILogger. Warnings are suppressed.");
    Console.WriteLine("Press Enter to close...");
    Console.ReadLine();
}

Console.WriteLine("\nSample complete!");
