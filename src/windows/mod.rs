use crate::tracked_window::{TrackedWindow, TrackedWindowControl};
use crate::MultiWindow;
use egui_glow::EguiGlow;
use glutin::{event_loop::ControlFlow, PossiblyCurrent};
use std::sync::Arc;

pub mod popup_window;
pub mod root;

#[enum_dispatch(TrackedWindow)]
pub enum MyWindows {
    Root(root::RootWindow),
    Popup(popup_window::PopupWindow),
}
