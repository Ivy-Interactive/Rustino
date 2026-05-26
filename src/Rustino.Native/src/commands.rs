use std::sync::mpsc;

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

    ShowOpenFileDialog(DialogParams, mpsc::Sender<Option<Vec<String>>>),
    ShowSaveFileDialog(DialogParams, mpsc::Sender<Option<String>>),
    ShowSelectFolderDialog(DialogParams, mpsc::Sender<Option<Vec<String>>>),

    SetMenu(String),
    RemoveMenu,
    ShowContextMenu(String, Option<(f64, f64)>),
    SetTrayIcon(TrayParams),
    RemoveTrayIcon,

    SetBadgeCount {
        count: Option<u32>,
        bg_r: u8,
        bg_g: u8,
        bg_b: u8,
        fg_r: u8,
        fg_g: u8,
        fg_b: u8,
    },

    GetMonitors(mpsc::Sender<String>),
    GetCurrentMonitor(mpsc::Sender<String>),

    MenuEventFired(String),
    TrayIconClicked,

    Close,
}

#[derive(Debug)]
pub struct DialogParams {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Vec<(String, Vec<String>)>,
    pub multi_select: bool,
}

#[derive(Debug)]
pub struct TrayParams {
    pub icon_path: String,
    pub tooltip: Option<String>,
    pub menu_json: Option<String>,
}
