use glutin::{event_loop::{ControlFlow, EventLoop}, platform::run_return::EventLoopExtRunReturn};

use crate::tracked_window::{TrackedWindow, TrackedWindowContainer};




/// Manages multiple `TrackedWindow`s by forwarding events to them.
pub struct MultiWindow<T> where T: TrackedWindow {
    windows: Vec<TrackedWindowContainer<T>>,
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
        self.windows.push(window)
    }

    /// Runs the event loop until all `TrackedWindow`s are closed.
    pub fn run(&mut self, event_loop: &mut EventLoop<()>) {
        if !self.windows.is_empty() {
            event_loop.run_return(|event, _, flow| {
                println!("handling event {:?}", event);
                let mut handled_windows = vec![];
                let mut window_control_flow = vec![];
                while let Some(mut window) = self.windows.pop() {
                    if window.is_event_for_window(&event) {
                        // Collect all the other windows.
                        let other_windows = self.windows.iter_mut().chain(handled_windows.iter_mut()).map(|container| &mut container.window).collect();
                        match window.handle_event_outer(&event, other_windows) {
                            Some(ControlFlow::Exit) => {
                                println!("window requested exit. Instead of sending the exit for everyone, just get rid of this one.");
                                window_control_flow.push(ControlFlow::Exit);
                                continue;
                                //*flow = ControlFlow::Exit
                            },
                            Some(requested_flow) => {
                                window_control_flow.push(requested_flow);
                            },
                            None => {
                                println!("window going away");
                                continue; // skip pushing this window back into the list
                            }
                        }
                    }
                    handled_windows.push(window);
                }

                // Move them back.
                handled_windows.reverse();
                self.windows.append(&mut handled_windows);

                // If any window requested polling, we should poll.
                // Precedence: Poll > WaitUntil(smallest) > Wait.
                *flow = ControlFlow::Wait;
                for flow_request in window_control_flow {
                    match flow_request {
                        ControlFlow::Poll => {
                            *flow = ControlFlow::Poll;
                        },
                        ControlFlow::Wait => (), // do nothing, if untouched it will be wait
                        ControlFlow::WaitUntil(when_new) => {
                            if let ControlFlow::Poll = *flow {
                                continue; // Polling takes precedence, so ignore this.
                            }

                            // The current flow is already WaitUntil. If this one is sooner, use it instead.
                            if let ControlFlow::WaitUntil(when_current) = *flow {
                                if when_new < when_current {
                                    *flow = ControlFlow::WaitUntil(when_new);
                                }
                            }
                            else { // The current flow is lower precedence, so replace it with this.
                                *flow = ControlFlow::WaitUntil(when_new);
                            }
                        },
                        ControlFlow::Exit => (), // handle differently, only exit if all windows are gone?? what do about a closed root window
                    }
                }

                // for window in &mut self.windows {
                //     if window.is_event_for_window(&event){
                //         match window.handle_event_outer(&event, vec![]) {
                //             Some(ControlFlow::Exit) => {
                //                 *flow = ControlFlow::Exit
                //             },
                //             Some(_) => (),
                //             None => ()
                //         }
                //     }
                // }


                // for option in &mut self.windows {
                //     if let Some(window) = option.as_mut() {
                //         match window.handle_event_outer(&event) {
                //             Some(ControlFlow::Exit) => {
                //                 *flow = ControlFlow::Exit
                //             },
                //             Some(_) => (),
                //             None => ()
                //         }
                //     }
                // }

                if self.windows.is_empty() {
                    *flow = ControlFlow::Exit;
                }
            });
        }
    }
}

