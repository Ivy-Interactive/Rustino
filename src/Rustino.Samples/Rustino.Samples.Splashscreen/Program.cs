using Rustino.NET;

Console.WriteLine("Starting app with splash...");

// Show splash for 3 seconds
using var splash = new RustinoSplashscreen("splash.png", 500, 300);

Console.WriteLine("Splash displayed, simulating initialization...");
Thread.Sleep(3000); // Simulate initialization

Console.WriteLine("Creating main window...");
var window = new RustinoWindow()
    .SetTitle("Splashscreen Sample")
    .SetSize(800, 600)
    .Center()
    .Load("data:text/html,<h1 style='font-family: sans-serif; text-align: center; margin-top: 200px;'>Main Window Ready!</h1>");

Console.WriteLine("Closing splash...");
splash.Close();

Console.WriteLine("Running main window...");
window.WaitForClose();
