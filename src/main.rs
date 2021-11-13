//! Example how to use pure `egui_glow` without [`epi`].
pub mod multi_window;
pub mod tracked_window;
pub mod windows;

use egui_glow::EguiGlow;
use glutin::event::Event;
use multi_window::MultiWindow;
use tracked_window::{EventHandlingWindowContainer, TrackedWindow};
use windows::root;


fn main() {
    let mut event_loop = glutin::event_loop::EventLoop::with_user_event();
    let mut multi_window = MultiWindow::new();
    let mut windows: Vec<Box<dyn EventHandlingWindowContainer>> = Default::default();
    let mut root_window = root::RootWindow::new(&event_loop).unwrap();
    let mut root_window2 = root::RootWindow::new(&event_loop).unwrap();

    multi_window.add(Box::new(root_window));
    multi_window.add(Box::new(root_window2));
    multi_window.run(&mut event_loop);

    /*
    event_loop.run(move |event, _, control_flow| {
        for window in &mut windows {
            let EguiWindow { gl_window, gl, egui } = window;
            let mut redraw = || {
                egui.begin_frame(gl_window.window());

                let mut quit = false;

                egui::SidePanel::left("my_side_panel").show(egui.ctx(), |ui| {
                    ui.heading("Hello World!");
                    if ui.button("Quit").clicked() {
                        quit = true;
                    }
                });

                let (needs_repaint, shapes) = egui.end_frame(gl_window.window());

                *control_flow = if quit {
                    glutin::event_loop::ControlFlow::Exit
                } else if needs_repaint {
                    gl_window.window().request_redraw();
                    glutin::event_loop::ControlFlow::Poll
                } else {
                    glutin::event_loop::ControlFlow::Wait
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
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }

                    if let glutin::event::WindowEvent::Resized(physical_size) = event {
                        gl_window.resize(physical_size);
                    }

                    egui.on_event(&event);

                    gl_window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
                }
                glutin::event::Event::LoopDestroyed => {
                    egui.destroy(&gl);
                }

                _ => (),
            }
        }
    });
    */
}
