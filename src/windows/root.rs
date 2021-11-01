use std::mem;

use crate::{multi_window::MultiWindow, tracked_window::{DisplayCreationError, TrackedWindow, WindowedContext}};
use glutin::{event::Event, event_loop::ControlFlow};
use thiserror::Error;
use crate::tracked_window::EguiWindow;


pub struct RootWindow {
    egui_window: EguiWindow,

}

impl RootWindow {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Result<Self, DisplayCreationError> {
        let display = <dyn TrackedWindow>::create_display(
            glutin::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(glutin::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title("egui-multiwin root window"),
                event_loop)?;
        
        Ok(Self {
            egui_window: display
        })

    }

}

impl TrackedWindow for RootWindow {
    fn handle_event(&mut self, event: &glutin::event::Event<()>) -> Option<ControlFlow> {
        match (event, &self.egui_window.gl_window) {
            (Event::WindowEvent { window_id: id, event, .. }, WindowedContext::PossiblyCurrent(gl_window))=> {
                if gl_window.window().id() != *id {
                    println!("skip event for other window");
                    return None
                }
            },
            (Event::WindowEvent { window_id: id, event, .. }, WindowedContext::NotCurrent(gl_window))=> {
                if gl_window.window().id() != *id {
                    println!("skip event for other window");
                    return None
                }
            },
            _ => ()
        }

        let gl_window = mem::replace(&mut self.egui_window.gl_window, WindowedContext::None);
        let gl_window = match gl_window {
            WindowedContext::PossiblyCurrent(w) => unsafe {w.make_current().unwrap()},
            WindowedContext::NotCurrent(w) => unsafe {w.make_current().unwrap()},
            WindowedContext::None => panic!("there's no window context???"),
        };

        match &mut self.egui_window {
            EguiWindow { gl_window: _, gl: None, egui: None } => {
                //let gl_window = unsafe {self.egui_window.gl_window.make_current().unwrap()};
                let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

                unsafe {
                    use glow::HasContext as _;
                    gl.enable(glow::FRAMEBUFFER_SRGB);
                }

                let egui = egui_glow::EguiGlow::new(&gl_window, &gl);
                self.egui_window.gl = Some(gl);
                self.egui_window.egui = Some(egui);

            },
            EguiWindow { gl_window: _, gl: Some(_), egui: Some(_) } => (),
            _ => {
                panic!("oh no");
            }
        }

        //let RootWindow { egui_window: EguiWindow {gl_window, gl, egui} } = self;
        //let gl_window = unsafe {self.egui_window.gl_window.make_current().unwrap()};
        //let gl = gl.unwrap();
        //let egui = egui.unwrap();

        // child window's control flow
        let mut control_flow = ControlFlow::Poll; // is this a reasonable default?
        let mut redraw = || {
            self.egui_window.egui.as_mut().unwrap().begin_frame(gl_window.window());

            let mut quit = false;

            egui::SidePanel::left("my_side_panel").show(self.egui_window.egui.as_mut().unwrap().ctx(), |ui| {
                ui.heading("Hello World!");
                if ui.button("Quit").clicked() {
                    quit = true;
                }
            });

            let (needs_repaint, shapes) = self.egui_window.egui.as_mut().unwrap().end_frame(gl_window.window());

            if quit {
                control_flow = glutin::event_loop::ControlFlow::Exit;
            } else if needs_repaint {
                gl_window.window().request_redraw();
                control_flow = glutin::event_loop::ControlFlow::Poll;
            } else {
                control_flow = glutin::event_loop::ControlFlow::Wait;
            };

            {
                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                unsafe {
                    use glow::HasContext as _;
                    self.egui_window.gl.as_mut().unwrap().clear_color(color[0], color[1], color[2], color[3]);
                    self.egui_window.gl.as_mut().unwrap().clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                self.egui_window.egui.as_mut().unwrap().paint(&gl_window, &self.egui_window.gl.as_mut().unwrap(), shapes);

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
            }

        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                if self.egui_window.egui.as_mut().unwrap().is_quit_event(&event) {
                    control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = event {
                    gl_window.resize(*physical_size);
                }

                self.egui_window.egui.as_mut().unwrap().on_event(&event);

                gl_window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            glutin::event::Event::LoopDestroyed => {
                self.egui_window.egui.as_mut().unwrap().destroy(self.egui_window.gl.as_ref().unwrap());
            }

            _ => (),
        }

        mem::replace(&mut self.egui_window.gl_window, WindowedContext::PossiblyCurrent(gl_window));
        Some(control_flow)
    }
}
