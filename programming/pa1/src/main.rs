#[macro_use]
extern crate glium;

use glium::{DisplayBuild, Surface, glutin};

fn main() {

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1024, 720)
        .with_title("CS6670 Programming Assignment 1 - Will Usher")
        .with_vsync()
        .build_glium().unwrap();
    // TODO: Use build_glium_debug on debug builds

    'outer: loop {
        for e in display.poll_events() {
            match e {
                glutin::Event::Closed => break 'outer,
                _ => {}
            }
        }

        let mut target = display.draw();
        target.clear_color(1.0, 0.0, 0.0, 0.0);
        target.finish().unwrap();
    }
}

