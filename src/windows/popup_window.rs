

use crate::{multi_window::{MultiWindow, NewWindowRequest}, tracked_window::{DisplayCreationError, TrackedWindow, TrackedWindowContainer, TrackedWindowControl}};
use egui_glow::EguiGlow;
use glutin::{PossiblyCurrent, event_loop::ControlFlow};

use crate::windows::MyWindows;


pub struct PopupWindow {
    pub input: String,
}

impl PopupWindow {
    pub fn new(label: String) -> NewWindowRequest {
        NewWindowRequest {
            window_state: PopupWindow {
                input: label.clone()
            }.into(),
            builder: glutin::window::WindowBuilder::new()
                .with_resizable(false)
                .with_inner_size(glutin::dpi::LogicalSize {
                    width: 400.0,
                    height: 200.0,
                })
                .with_title(label)
        }
    }
}

impl TrackedWindow for PopupWindow {
    fn handle_event(&mut self, event: &glutin::event::Event<()>, other_windows: Vec<&mut MyWindows>, egui: &mut EguiGlow, gl_window: &mut glutin::WindowedContext<PossiblyCurrent>, gl: &mut glow::Context) -> TrackedWindowControl {

        // Child window's requested control flow.
        let mut control_flow = ControlFlow::Wait; // Unless this changes, we're fine waiting until the next event comes in.

        // Check if there are any root windows left, and exit if there are none.
        let mut root_window_exists = false;
        for other in &other_windows {
            if let MyWindows::Root(_) = other {
                root_window_exists = true;
            }
        }

        let redraw = || {
            egui.begin_frame(gl_window.window());

            let mut quit = false;

            egui::CentralPanel::default().show(egui.ctx(), |ui| {
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
                let response = ui.add(egui::TextEdit::singleline(&mut self.input));
                if response.changed() {
                    // …
                }
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    // …
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

                egui.paint(gl_window, gl, shapes);

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
                if egui.is_quit_event(event) {
                    control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = event {
                    gl_window.resize(*physical_size);
                }

                egui.on_event(event);

                gl_window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            glutin::event::Event::LoopDestroyed => {
                egui.destroy(gl);
            }

            _ => (),
        }

        if !root_window_exists {
            println!("Root window is gone, exiting popup.");
            control_flow = ControlFlow::Exit;
        }

        TrackedWindowControl {
            requested_control_flow: control_flow,
            windows_to_create: vec![]
        }
    }
}
