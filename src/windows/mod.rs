use glutin::{ContextWrapper, PossiblyCurrent, WindowedContext, event::Event, event_loop::ControlFlow, window::Window};
use crate::tracked_window::{TrackedWindow};
use egui_glow::EguiGlow;

pub mod popup_window;
pub mod root;

#[enum_dispatch(TrackedWindow, EventHandlingTrackedWindow)]
pub enum MyWindows {
    Root(root::RootWindow),
    Popup(popup_window::PopupWindow)
}