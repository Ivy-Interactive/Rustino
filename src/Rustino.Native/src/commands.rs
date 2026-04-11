#[derive(Debug)]
pub enum RustinoCommand {
    SetTitle(String),
    SetSize(u32, u32),
    SetMinimized(bool),
    SetMaximized(bool),
    SetFullscreen(bool),
    SetVisible(bool),
    SetFocus,
    SetDecorations(bool),
    SetPosition(i32, i32),
    Center,
    SetMinSize(Option<(u32, u32)>),
    SetMaxSize(Option<(u32, u32)>),
    SetResizable(bool),
    SetTopmost(bool),
    SetIconFile(String),

    EvaluateScript(String),
    SendWebMessage(String),
    LoadUrl(String),
    LoadHtml(String),
    SetZoom(f64),
    SetBackgroundColor(u8, u8, u8, u8),

    Close,
}
