use glutin::{ContextWrapper, PossiblyCurrent, WindowedContext, event::Event, event_loop::ControlFlow, window::Window};
use crate::TrackedWindow;
use egui_glow::EguiGlow;

pub mod root;

#[enum_dispatch(TrackedWindow)]
pub enum MyWindows {
    Root(root::RootWindow)
}