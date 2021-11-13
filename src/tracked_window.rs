use std::mem;

use egui_glow::EguiGlow;
use glutin::{PossiblyCurrent, event::Event, event_loop::ControlFlow};
use thiserror::Error;
use crate::windows::MyWindows;


/// A window being tracked by a `MultiWindow`. All tracked windows will be forwarded all events
/// received on the `MultiWindow`'s event loop.
#[enum_dispatch]
pub trait TrackedWindow {
    /// Handles one event from the event loop. Returns true if the window needs to be kept alive,
    /// otherwise it will be closed. Window events should be checked to ensure that their ID is one
    /// that the TrackedWindow is interested in.
    fn handle_event(&mut self, event: &glutin::event::Event<()>, other_windows: Vec<&mut crate::windows::MyWindows>, egui: &mut EguiGlow, gl_window: &mut glutin::WindowedContext<PossiblyCurrent>, gl: &mut glow::Context) -> Option<ControlFlow>;
}

pub struct TrackedWindowContainer {
    pub gl_window: IndeterminateWindowedContext,
    pub gl: Option<glow::Context>,
    pub egui: Option<EguiGlow>,
    pub window: MyWindows
}

impl TrackedWindowContainer {
    pub fn create(
        window: MyWindows,
        window_builder: glutin::window::WindowBuilder,
        event_loop: &glutin::event_loop::EventLoop<()>,
    ) -> Result<TrackedWindowContainer, DisplayCreationError> {
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

        Ok(TrackedWindowContainer {
            window,
            gl_window: IndeterminateWindowedContext::NotCurrent(gl_window),
            gl: None,
            egui: None
        })
    }

    pub fn is_event_for_window(&self, event: &glutin::event::Event<()>) -> bool {
        // Check if the window ID matches, if not then this window can pass on the event.
        match (event, &self.gl_window) {
            (Event::WindowEvent { window_id: id, event: _, .. }, IndeterminateWindowedContext::PossiblyCurrent(gl_window))=> {
                gl_window.window().id() == *id
            },
            (Event::WindowEvent { window_id: id, event: _, .. }, IndeterminateWindowedContext::NotCurrent(gl_window))=> {
                gl_window.window().id() == *id
            },
            _ => true // we weren't able to check the window ID, maybe this window is not initialized yet. we should run it.
        }
    }

    pub fn handle_event_outer(&mut self, event: &glutin::event::Event<()>, other_windows:Vec<&mut crate::windows::MyWindows>) -> Option<ControlFlow> {

        // Activate this gl_window so we can use it.
        // We cannot activate it without full ownership, so temporarily move the gl_window into the current scope.
        // It *must* be returned at the end.
        let gl_window = mem::replace(&mut self.gl_window, IndeterminateWindowedContext::None);
        let mut gl_window = match gl_window {
            IndeterminateWindowedContext::PossiblyCurrent(w) => unsafe {w.make_current().unwrap()},
            IndeterminateWindowedContext::NotCurrent(w) => unsafe {w.make_current().unwrap()},
            IndeterminateWindowedContext::None => panic!("there's no window context???"),
        };

        // Now that the window is active, create a context if it is missing.
        match (self.gl.as_ref(), self.egui.as_ref()) {
            (None, None) => {
                let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

                unsafe {
                    use glow::HasContext as _;
                    gl.enable(glow::FRAMEBUFFER_SRGB);
                }

                let egui = egui_glow::EguiGlow::new(&gl_window, &gl);
                self.gl = Some(gl);
                self.egui = Some(egui);
            },
            (Some(_), Some(_)) => (),
            _ => { panic!("Partially initialized window"); }
        };

        let result = match (self.gl.as_mut(), self.egui.as_mut()) {
            (Some(gl), Some(egui)) => {
                match self.window.handle_event(event, other_windows, egui, &mut gl_window, gl) {
                    Some(ControlFlow::Exit) | None => {
                        // This window wants to go away. Close it.
                        egui.destroy(gl);
                        None
                    },
                    result => result
                }
            },
            _ => {
                panic!("Window wasn't fully initialized");
            }
        };

         
        match mem::replace(&mut self.gl_window, IndeterminateWindowedContext::PossiblyCurrent(gl_window)){
            IndeterminateWindowedContext::None => (),
            _ => {
                panic!("Window had a GL context while we were borrowing it?");
            }
        }
        result


        // self.gl_window.makecurr
        // We have to take ownership of it, because make_current()

        // let gl_window = mem::replace(&mut self.egui_window.gl_window, WindowedContext::None);
        // let gl_window = match gl_window {
        //     WindowedContext::PossiblyCurrent(w) => unsafe {w.make_current().unwrap()},
        //     WindowedContext::NotCurrent(w) => unsafe {w.make_current().unwrap()},
        //     WindowedContext::None => panic!("there's no window context???"),
        // };

    }
}

pub enum IndeterminateWindowedContext {
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