use std::mem;

use crate::{multi_window::MultiWindow, tracked_window::{DisplayCreationError, TrackedWindow, TrackedWindowContainer, IndeterminateWindowedContext}};
use egui_glow::EguiGlow;
use glutin::{PossiblyCurrent, event::Event, event_loop::ControlFlow};
use thiserror::Error;
use crate::windows::MyWindows;


pub struct PopupWindow {
}

impl PopupWindow {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Result<TrackedWindowContainer, DisplayCreationError> {
        Ok(TrackedWindowContainer::create(
            PopupWindow {}.into(),
            glutin::window::WindowBuilder::new()
                .with_resizable(false)
                .with_inner_size(glutin::dpi::LogicalSize {
                    width: 400.0,
                    height: 400.0,
                })
                .with_title("egui-multiwin popup window"),
                event_loop)?)
    }
}

impl TrackedWindow for PopupWindow {
    fn handle_event(&mut self, event: &glutin::event::Event<()>, other_windows: Vec<&mut MyWindows>, egui: &mut EguiGlow, gl_window: &mut glutin::WindowedContext<PossiblyCurrent>, gl: &mut glow::Context) -> Option<ControlFlow> {

        // Child window's requested control flow.
        let mut control_flow = ControlFlow::Wait; // Unless this changes, we're fine waiting until the next event comes in.
        let mut redraw = || {
            egui.begin_frame(gl_window.window());

            let mut quit = false;

            egui::CentralPanel::default().show(egui.ctx(), |ui| {
                ui.heading("I'm different");
                if ui.button("Increment").clicked() {
                    for window in other_windows {
                        match window {
                            MyWindows::Root(root_window) => {
                                root_window.button_press_count += 1
                            }
                            _ => ()
                        }
                    }
                }
                if ui.button("Quit").clicked() {
                    quit = true;
                }
            });

            let (needs_repaint, shapes) = egui.end_frame(gl_window.window());

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
                    gl.clear_color(color[0], color[1], color[2], color[3]);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                egui.paint(&gl_window, &gl, shapes);

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
                if egui.is_quit_event(&event) {
                    control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = event {
                    gl_window.resize(*physical_size);
                }

                egui.on_event(&event);

                gl_window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            glutin::event::Event::LoopDestroyed => {
                egui.destroy(gl);
            }

            _ => (),
        }

        Some(control_flow)
    }
}
