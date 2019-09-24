mod buffer;
mod render;

use buffer::Buffer;
use minifb::{Key, MouseMode, Window, WindowOptions};
use render::{render_loop, RenderCommand, RenderParameters, RenderResult};
use std::sync::mpsc::channel;
use std::thread::spawn;

fn main() {
    // Create buffer for rendering.
    const WIDTH: usize = 800;
    const HEIGHT: usize = 600;

    let buffer = Buffer::new(WIDTH, HEIGHT);

    // Create window.
    let mut window = Window::new(
        "Fractal",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .expect("Cannot create a window.");

    // Create communication channels.
    let (request_tx, request_rx) = channel::<RenderCommand>();
    let (response_tx, response_rx) = channel::<RenderResult>();

    // Spawn rendering thread
    let render_thread = spawn(|| {
        render_loop(request_rx, response_tx);
    });

    // Let's describe out window's state
    #[derive(Debug)]
    enum WindowState {
        RenderRequest {
            buffer: Buffer,
            params: RenderParameters,
        },
        RequestPending {
            params: RenderParameters,
        },
        Idle {
            buffer: Buffer,
            params: RenderParameters,
        },
    }

    let mut state = WindowState::RenderRequest {
        buffer,
        params: RenderParameters {
            iterations: 110,
            cx: -0.6,
            cy: 0.5,
        },
    };

    // Main window loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Collect input
        let new_params = window
            .get_mouse_pos(MouseMode::Clamp)
            .map(|mouse_pos| RenderParameters {
                iterations: 110,
                cx: -0.6 + mouse_pos.0 / WIDTH as f32 * 0.2,
                cy: 0.5 + mouse_pos.1 / HEIGHT as f32 * 0.2,
            });

        // Handle state
        state = match state {
            // Send render request to render thread.
            WindowState::RenderRequest { buffer, params } => {
                request_tx
                    .send(RenderCommand::RenderRequest { buffer, params })
                    .expect("Cannot send request to a render thread");
                WindowState::RequestPending { params }
            }

            // Check if buffer is available again
            WindowState::RequestPending { params } => {
                if let Ok(RenderResult {
                    buffer,
                    render_time,
                }) = response_rx.try_recv()
                {
                    // Update screen and go to idle state
                    window.update_with_buffer(buffer.as_slice()).unwrap();
                    window.set_title(&format!(
                        "Fractal ({:?}, {}, {})",
                        render_time, params.cx, params.cy
                    ));
                    WindowState::Idle { buffer, params }
                } else {
                    // Work is still pending
                    WindowState::RequestPending { params }
                }
            }

            // Handle Idle
            WindowState::Idle { buffer, params } => match new_params {
                Some(new_params) if new_params != params => WindowState::RenderRequest {
                    buffer,
                    params: new_params,
                },
                _ => WindowState::Idle { buffer, params },
            },
        };

        // Update inputs.
        // Note: MiniFB library doesn't have "sleeping" functionality and the
        // main thread will spin CPU to 100%. It's OK as we want to keep code
        // to a minimum and there are other libraries, which provide this functionality.
        window.update();
    }

    // Close render thread.
    request_tx
        .send(RenderCommand::Quit)
        .expect("Cannot close render thread.");

    render_thread.join().expect("Cannot join render thread.")
}
