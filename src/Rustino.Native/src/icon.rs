use tao::window::Icon;

pub fn load_icon(path: &str) -> Option<Icon> {
    let img = image::open(path).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Icon::from_rgba(rgba, width, height).ok()
}
