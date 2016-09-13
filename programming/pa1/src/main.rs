#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;

mod imgui_support;

use glium::{DisplayBuild, Surface};
use glium::glutin::{self, ElementState, Event, MouseButton, VirtualKeyCode};

use imgui_support::ImGuiSupport;

fn main() {
    let target_gl_versions = glutin::GlRequest::GlThenGles {
        opengl_version: (3, 3),
        opengles_version: (3, 2),
    };
    let display = glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_gl(target_gl_versions)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_title("CS6670 Programming Assignment 1 - Will Usher")
        .with_vsync()
        .build_glium().unwrap();

    println!("Got OpenGL: {:?}", display.get_opengl_version());
    println!("Got GLSL: {:?}", display.get_supported_glsl_version());

    let mut imgui = ImGuiSupport::init();
    let mut imgui_renderer = imgui::glium_renderer::Renderer::init(&mut imgui.imgui, &display).unwrap();

    'outer: loop {
        for e in display.poll_events() {
            imgui.update_event(&e);
            match e {
                glutin::Event::Closed => break 'outer,
                Event::KeyboardInput(state, _, code) => {
                    let pressed = state == ElementState::Pressed;
                    match code {
                        Some(VirtualKeyCode::Escape) if pressed => break 'outer,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        imgui.update_mouse();

        let mut target = display.draw();
        target.clear_color(0.2, 0.2, 0.2, 1.0);

        let ui = imgui.render_ui(&display);
        ui.window(im_str!("Control Panel"))
            .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                let fps = ui.framerate();
                let gl_version = display.get_opengl_version();
                let glsl_version = display.get_supported_glsl_version();
                ui.text(im_str!("Framerate: {:.3} FPS ({:.3} ms)", fps, 1000.0 / fps));
                ui.text(im_str!("OpenGL Version: {}.{}", gl_version.1, gl_version.2));
                ui.text(im_str!("GLSL Version: {}.{}", glsl_version.1, glsl_version.2));
            });
        imgui_renderer.render(&mut target, ui).unwrap();

        target.finish().unwrap();
    }
}

