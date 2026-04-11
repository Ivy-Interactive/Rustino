using Rustino.NET;

var window = new RustinoWindow();
window
    .SetTitle("Hello Rustino!")
    .SetUseOsDefaultSize(false)
    .SetSize(1280, 800)
    .SetResizable(true)
    .Center()
    .Load(new Uri("https://example.com"));

window.WaitForClose();
