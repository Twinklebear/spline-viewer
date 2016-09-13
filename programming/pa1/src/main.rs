#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate image;
extern crate cgmath;

mod imgui_support;
mod bezier;

use std::ops::{Mul, Add};
use std::iter;

use glium::{DisplayBuild, Surface};
use glium::glutin::{self, ElementState, Event, MouseButton, VirtualKeyCode};

use imgui_support::ImGuiSupport;
use bezier::Bezier;

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point { x: x, y: y }
    }
}
impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Point {
        Point { x: self.x * rhs, y: self.y * rhs }
    }
}
impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

/// Evaluate the Bezier curve and plot it to the image buffer passed
fn plot_2d(spline: &Bezier<Point>, plot: &mut [u8], plot_dim: (usize, usize), scale: (f32, f32),
           offset: (f32, f32)) {
    let step_size = 0.001;
    let t_range = (0.0, 1.0);
    let steps = ((t_range.1 - t_range.0) / step_size) as usize;
    for s in 0..steps + 1 {
        let t = step_size * s as f32 + t_range.0;
        let pt = spline.point(t);
        let ix = ((pt.x + offset.0) * scale.0) as isize;
        let iy = ((pt.y + offset.1) * scale.1) as isize;
        for y in iy - 1..iy + 1 {
            for x in ix - 1..ix + 1 {
                if y >= 0 && y < plot_dim.1 as isize && x >= 0 && x < plot_dim.0 as isize {
                    let px = (plot_dim.1 - 1 - y as usize) * plot_dim.0 * 3 + x as usize * 3;
                    for i in 0..3 {
                        plot[px + i] = 0;
                    }
                }
            }
        }
    }
    // Draw the control points
    for pt in spline.control_points() {
        let ix = ((pt.x + offset.0) * scale.0) as isize;
        let iy = ((pt.y + offset.1) * scale.1) as isize;
        for y in iy - 3..iy + 3 {
            for x in ix - 3..ix + 3 {
                if y >= 0 && y < plot_dim.1 as isize && x >= 0 && x < plot_dim.0 as isize {
                    let px = (plot_dim.1 - 1 - y as usize) * plot_dim.0 * 3 + x as usize * 3;
                    plot[px] = 255;
                    plot[px + 1] = 0;
                    plot[px + 2] = 0;
                }
            }
        }
    }
}

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

    let points = vec![Point::new(1.0, 0.0), Point::new(1.0, 1.0), Point::new(0.0, 1.0)];

    let plot_dim = (800, 800);
    let scale = (plot_dim.0 as f32 / 4.0, plot_dim.1 as f32 / 4.0);
    let offset = (2.0, 2.0);

    let mut plot: Vec<_> = iter::repeat(255u8).take(plot_dim.0 * plot_dim.1 * 3).collect();
    let curve = Bezier::new(points);
    plot_2d(&curve, &mut plot[..], plot_dim, scale, offset);
    match image::save_buffer("test.png", &plot[..], plot_dim.0 as u32, plot_dim.1 as u32, image::RGB(8)) {
        Ok(_) => println!("Test B-spline saved to test.png"),
        Err(e) => println!("Error saving test.png,  {}", e),
    }


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

