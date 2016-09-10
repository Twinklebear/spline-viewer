#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;

use glium::{DisplayBuild, Surface, glutin};
use imgui::ImGui;

fn main() {

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("CS6670 Programming Assignment 1 - Will Usher")
        .with_vsync()
        .build_glium().unwrap();
    // TODO: Use build_glium_debug on debug builds

    let mut imgui = ImGui::init();
    let mut imgui_renderer = imgui::glium_renderer::Renderer::init(&mut imgui, &display).unwrap();

    // TODO: Setup key bindings for imgui

    'outer: loop {
        for e in display.poll_events() {
            match e {
                glutin::Event::Closed => break 'outer,
                _ => {}
            }
        }

        let mut target = display.draw();
        target.clear_color(0.2, 0.2, 0.2, 1.0);

        let window = display.get_window().unwrap();
        let size_pts = window.get_inner_size_points().unwrap();
        let size_pixels = window.get_inner_size_pixels().unwrap();
        let ui = imgui.frame(size_pts, size_pixels, 0.16);
        ui.window(im_str!("This is a programming assignment"))
            .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                ui.text(im_str!("some ui text"));
            });
        imgui_renderer.render(&mut target, ui).unwrap();

        target.finish().unwrap();
    }
}

