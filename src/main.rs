#![allow(dead_code)]

use winit::{event::{WindowEvent}, event::{Event, VirtualKeyCode, ElementState}, event_loop::ControlFlow };

mod gpu;
mod chip8;
mod display;

use chip8::Chip8;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let mut display = display::Chip8Display::new(&event_loop);
    let mut chip8 = Chip8::new(include_bytes!("../roms/trip8.rom"));

    event_loop.run(move |event, _, control_flow| -> () {
        let window = display.window();
        control_flow.set_wait();

        if let Event::WindowEvent {event, ..} = event {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    let key = match input.virtual_keycode {
                        Some(VirtualKeyCode::Key0) => 0,
                        Some(VirtualKeyCode::Key1) => 1,
                        Some(VirtualKeyCode::Key2) => 2,
                        Some(VirtualKeyCode::Key3) => 3,
                        Some(VirtualKeyCode::Key4) => 4,
                        Some(VirtualKeyCode::Key5) => 5,
                        Some(VirtualKeyCode::Key6) => 6,
                        Some(VirtualKeyCode::Key7) => 7,
                        Some(VirtualKeyCode::Key8) => 8,
                        Some(VirtualKeyCode::Key9) => 9,
                        Some(VirtualKeyCode::A) => 10,
                        Some(VirtualKeyCode::B) => 11,
                        Some(VirtualKeyCode::C) => 12,
                        Some(VirtualKeyCode::D) => 13,
                        Some(VirtualKeyCode::E) => 14,
                        Some(VirtualKeyCode::F) => 15,
                        Some(VirtualKeyCode::Tab) => { chip8.reset(); 255 },
                        Some(VirtualKeyCode::Return) => { chip8::dump_display(&chip8); 255 }
                        Some(VirtualKeyCode::Escape) => { *control_flow = ControlFlow::Exit; 255 }
                        _ => 255
                    };

                    if key != 255 {
                        chip8.set_key_state(key, input.state == ElementState::Pressed);
                    }
                }
                _ => ()
            }
        } else if let Event::RedrawRequested(_) = event {
            window.request_redraw();
            chip8.tick_60hz();
            chip8.step(10);
            display.update(chip8.pixels());
        }
    });

}
