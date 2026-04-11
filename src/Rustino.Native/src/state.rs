use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct SharedState {
    pub is_minimized: AtomicBool,
    pub is_maximized: AtomicBool,
    pub is_fullscreen: AtomicBool,
    pub is_visible: AtomicBool,
    pub is_focused: AtomicBool,
    position: AtomicU64,
    size: AtomicU64,
}

impl SharedState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            is_minimized: AtomicBool::new(false),
            is_maximized: AtomicBool::new(false),
            is_fullscreen: AtomicBool::new(false),
            is_visible: AtomicBool::new(true),
            is_focused: AtomicBool::new(true),
            position: AtomicU64::new(0),
            size: AtomicU64::new(pack_u32(width, height)),
        }
    }

    pub fn store_position(&self, x: i32, y: i32) {
        self.position.store(pack_i32(x, y), Ordering::Release);
    }

    pub fn load_position(&self) -> (i32, i32) {
        unpack_i32(self.position.load(Ordering::Acquire))
    }

    pub fn store_size(&self, w: u32, h: u32) {
        self.size.store(pack_u32(w, h), Ordering::Release);
    }

    pub fn load_size(&self) -> (u32, u32) {
        unpack_u32(self.size.load(Ordering::Acquire))
    }
}

fn pack_i32(a: i32, b: i32) -> u64 {
    ((a as u32 as u64) << 32) | (b as u32 as u64)
}

fn unpack_i32(v: u64) -> (i32, i32) {
    ((v >> 32) as u32 as i32, v as u32 as i32)
}

fn pack_u32(a: u32, b: u32) -> u64 {
    ((a as u64) << 32) | (b as u64)
}

fn unpack_u32(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_size() {
        let state = SharedState::new(1024, 768);
        assert_eq!(state.load_size(), (1024, 768));
    }

    #[test]
    fn new_defaults_are_correct() {
        let state = SharedState::new(800, 600);
        assert!(!state.is_minimized.load(Ordering::Acquire));
        assert!(!state.is_maximized.load(Ordering::Acquire));
        assert!(!state.is_fullscreen.load(Ordering::Acquire));
        assert!(state.is_visible.load(Ordering::Acquire));
        assert!(state.is_focused.load(Ordering::Acquire));
        assert_eq!(state.load_position(), (0, 0));
    }

    #[test]
    fn atomic_updates_visible() {
        let state = SharedState::new(800, 600);
        state.is_minimized.store(true, Ordering::Release);
        assert!(state.is_minimized.load(Ordering::Acquire));
        state.is_minimized.store(false, Ordering::Release);
        assert!(!state.is_minimized.load(Ordering::Acquire));
    }

    #[test]
    fn position_pack_unpack() {
        let state = SharedState::new(800, 600);
        state.store_position(100, 200);
        assert_eq!(state.load_position(), (100, 200));
        state.store_position(-50, -100);
        assert_eq!(state.load_position(), (-50, -100));
    }

    #[test]
    fn size_pack_unpack() {
        let state = SharedState::new(800, 600);
        state.store_size(1920, 1080);
        assert_eq!(state.load_size(), (1920, 1080));
    }

    #[test]
    fn pack_i32_roundtrip_extremes() {
        assert_eq!(unpack_i32(pack_i32(i32::MIN, i32::MAX)), (i32::MIN, i32::MAX));
        assert_eq!(unpack_i32(pack_i32(0, 0)), (0, 0));
        assert_eq!(unpack_i32(pack_i32(-1, -1)), (-1, -1));
    }

    #[test]
    fn pack_u32_roundtrip_extremes() {
        assert_eq!(unpack_u32(pack_u32(0, u32::MAX)), (0, u32::MAX));
        assert_eq!(unpack_u32(pack_u32(u32::MAX, 0)), (u32::MAX, 0));
    }
}
