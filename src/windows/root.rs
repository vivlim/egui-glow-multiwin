

use crate::{multi_window::NewWindowRequest, tracked_window::{DisplayCreationError, TrackedWindow, TrackedWindowContainer, TrackedWindowControl}, windows::popup_window::PopupWindow};
use egui_glow::EguiGlow;
use glutin::{PossiblyCurrent, event_loop::ControlFlow};
use crate::MultiWindow;

use crate::windows::MyWindows;


pub struct RootWindow {
    pub button_press_count: u32,
    pub num_popups_created: u32,
}

impl RootWindow {
    pub fn new() -> NewWindowRequest {
        NewWindowRequest {
            window_state: RootWindow {
                button_press_count: 0,
                num_popups_created: 0,
            }.into(),
            builder: glutin::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(glutin::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title("egui-multiwin root window")
        }
    }
}

impl TrackedWindow for RootWindow {
    fn handle_event(&mut self, event: &glutin::event::Event<()>, other_windows: Vec<&mut MyWindows>, egui: &mut EguiGlow, gl_window: &mut glutin::WindowedContext<PossiblyCurrent>, gl: &mut glow::Context) -> TrackedWindowControl {

        // Child window's requested control flow.
        let mut control_flow = ControlFlow::Wait; // Unless this changes, we're fine waiting until the next event comes in.

        let mut windows_to_create = vec![];

        let mut redraw = || {
            egui.begin_frame(gl_window.window());

            let mut quit = false;

            egui::SidePanel::left("my_side_panel").show(egui.ctx(), |ui| {
                ui.heading("Hello World!");
                if ui.button("New popup").clicked() {
                    windows_to_create.push(PopupWindow::new(format!("popup window #{}", self.num_popups_created)));
                    self.num_popups_created += 1;
                }
                if ui.button("Quit").clicked() {
                    quit = true;
                }
            });
            egui::CentralPanel::default().show(egui.ctx(), |ui| {
                ui.heading(format!("number {}", self.button_press_count));

                for window in other_windows {
                    match window {
                        MyWindows::Popup(popup_window) => {
                            ui.add(egui::TextEdit::singleline(&mut popup_window.input));
                        },
                        _ => ()
                    }
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

        TrackedWindowControl {
            requested_control_flow: control_flow,
            windows_to_create
        }
    }
}
