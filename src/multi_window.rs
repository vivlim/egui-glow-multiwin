use glutin::{
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

use crate::tracked_window::{DisplayCreationError, TrackedWindowContainer};
use crate::windows::MyWindows;

/// Manages multiple `TrackedWindow`s by forwarding events to them.
pub struct MultiWindow {
    windows: Vec<TrackedWindowContainer>,
}

impl MultiWindow {
    /// Creates a new `MultiWindow`.
    pub fn new() -> Self {
        MultiWindow { windows: vec![] }
    }

    /// Adds a new `TrackedWindow` to the `MultiWindow`.
    pub fn add<TE>(
        &mut self,
        window: NewWindowRequest,
        event_loop: &glutin::event_loop::EventLoopWindowTarget<TE>,
    ) -> Result<(), DisplayCreationError> {
        Ok(self.windows.push(TrackedWindowContainer::create(
            window.window_state,
            window.builder,
            event_loop,
        )?))
    }

    /// Runs the event loop until all `TrackedWindow`s are closed.
    pub fn run(mut multi_window: MultiWindow, event_loop: EventLoop<()>) {
        event_loop.run(move |event, event_loop_window_target, flow| {
            println!("handling event {:?}", event);
            let mut handled_windows = vec![];
            let mut window_control_flow = vec![];
            while let Some(mut window) = multi_window.windows.pop() {
                if window.is_event_for_window(&event) {
                    // Collect all the other windows.
                    let other_windows = multi_window.windows.iter_mut().chain(handled_windows.iter_mut()).map(|container| &mut container.window).collect();
                    let window_control = window.handle_event_outer(&event, &event_loop_window_target, other_windows);
                    match window_control.requested_control_flow {
                        ControlFlow::Exit => {
                            println!("window requested exit. Instead of sending the exit for everyone, just get rid of this one.");
                            window_control_flow.push(ControlFlow::Exit);
                            continue;
                            //*flow = ControlFlow::Exit
                        },
                        requested_flow => {
                            window_control_flow.push(requested_flow);
                        }
                    }

                    for new_window_request in window_control.windows_to_create {
                        multi_window.add(new_window_request, event_loop_window_target);

                    }
                }
                handled_windows.push(window);
            }

            // Move them back.
            handled_windows.reverse();
            multi_window.windows.append(&mut handled_windows);

            // If any window requested polling, we should poll.
            // Precedence: Poll > WaitUntil(smallest) > Wait.
            if let ControlFlow::Exit = *flow {
            } else {
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
                        ControlFlow::ExitWithCode(n) => (),
                        ControlFlow::Exit => (), // handle differently, only exit if all windows are gone?? what do about a closed root window
                    }
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

            if multi_window.windows.is_empty() {
                println!("no more windows running, exiting event loop.");
                *flow = ControlFlow::Exit;

            }
        });
    }
}

pub struct NewWindowRequest {
    pub window_state: MyWindows,
    pub builder: glutin::window::WindowBuilder,
}
