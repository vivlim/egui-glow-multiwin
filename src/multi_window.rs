use glutin::{event_loop::{ControlFlow, EventLoop}, platform::run_return::EventLoopExtRunReturn};

use crate::tracked_window::{TrackedWindow, TrackedWindowContainer};




/// Manages multiple `TrackedWindow`s by forwarding events to them.
pub struct MultiWindow<T> where T: TrackedWindow {
    windows: Vec<Option<TrackedWindowContainer<T>>>,
}

impl<T> MultiWindow<T> where T: TrackedWindow {
    /// Creates a new `MultiWindow`.
    pub fn new() -> Self {
        MultiWindow {
            windows: vec![],
        }
    }

    /// Adds a new `TrackedWindow` to the `MultiWindow`.
    pub fn add(&mut self, window: TrackedWindowContainer<T>) {
        self.windows.push(Some(window))
    }

    /// Runs the event loop until all `TrackedWindow`s are closed.
    pub fn run(&mut self, event_loop: &mut EventLoop<()>) {
        if !self.windows.is_empty() {
            event_loop.run_return(|event, _, flow| {
                *flow = ControlFlow::Poll;

                for option in &mut self.windows {
                    if let Some(window) = option.as_mut() {
                        match window.handle_event_outer(&event) {
                            Some(ControlFlow::Exit) => {
                                *flow = ControlFlow::Exit
                            },
                            Some(_) => (),
                            None => ()
                        }
                    }
                }

                self.windows.retain(Option::is_some);

                if self.windows.is_empty() {
                    *flow = ControlFlow::Exit;
                }
            });
        }
    }
}

