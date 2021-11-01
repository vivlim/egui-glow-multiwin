use egui_glow::EguiGlow;
use glutin::event_loop::ControlFlow;
use thiserror::Error;


/// A window being tracked by a `MultiWindow`. All tracked windows will be forwarded all events
/// received on the `MultiWindow`'s event loop.
pub trait TrackedWindow {
    /// Handles one event from the event loop. Returns true if the window needs to be kept alive,
    /// otherwise it will be closed. Window events should be checked to ensure that their ID is one
    /// that the TrackedWindow is interested in.
    fn handle_event(&mut self, event: &glutin::event::Event<()>) -> Option<ControlFlow>;

}

impl dyn TrackedWindow {

    pub fn create_display(
        window_builder: glutin::window::WindowBuilder,
        event_loop: &glutin::event_loop::EventLoop<()>,
    ) -> Result<EguiWindow, DisplayCreationError> {
        // let window_builder = glutin::window::WindowBuilder::new()
        //     .with_resizable(true)
        //     .with_inner_size(glutin::dpi::LogicalSize {
        //         width: 800.0,
        //         height: 600.0,
        //     })
        //     .with_title("egui_glow example");

        let gl_window =
            glutin::ContextBuilder::new()
                .with_depth_buffer(0)
                .with_srgb(true)
                .with_stencil_buffer(0)
                .with_vsync(true)
                .build_windowed(window_builder, event_loop)?;

        Ok(EguiWindow {
            gl_window: WindowedContext::NotCurrent(gl_window),
            gl: None,
            egui: None
        })
    }
}


pub struct EguiWindow {
    pub gl_window: WindowedContext,
    pub gl: Option<glow::Context>,
    pub egui: Option<EguiGlow>
}

pub enum WindowedContext {
    PossiblyCurrent(glutin::WindowedContext<glutin::PossiblyCurrent>),
    NotCurrent(glutin::WindowedContext<glutin::NotCurrent>),
    None
}

#[derive(Error, Debug)]
pub enum DisplayCreationError {
    #[error("couldn't create window {0}")]
    Creation(#[from] glutin::CreationError),
    #[error("couldn't create context {0:?}")]
    Context(#[from] glutin::ContextError)

}