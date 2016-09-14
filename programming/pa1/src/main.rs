#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate cgmath;

mod imgui_support;
mod bezier;
mod camera2d;

use std::ops::{Mul, Add};

use glium::{DisplayBuild, Surface, DrawParameters};
use glium::vertex::VertexBuffer;
use glium::index::{NoIndices, PrimitiveType};
use glium::glutin::{self, ElementState, Event, MouseButton, VirtualKeyCode};

use imgui_support::ImGuiSupport;
use bezier::Bezier;
use camera2d::Camera2d;

#[derive(Copy, Clone, Debug)]
struct Point {
    pos: [f32; 2],
}
impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point { pos: [x, y] }
    }
}
implement_vertex!(Point, pos);

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Point {
        Point { pos: [self.pos[0] * rhs, self.pos[1] * rhs] }
    }
}
impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point { pos: [self.pos[0] + rhs.pos[0], self.pos[1] + rhs.pos[1]] }
    }
}

fn main() {
    let target_gl_versions = glutin::GlRequest::GlThenGles {
        opengl_version: (3, 3),
        opengles_version: (3, 2),
    };
    let width = 1280;
    let height = 720;
    let display = glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_gl(target_gl_versions)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_title("CS6670 Programming Assignment 1 - Will Usher")
        .with_vsync()
        .build_glium().unwrap();

    println!("Got OpenGL: {:?}", display.get_opengl_version());
    println!("Got GLSL: {:?}", display.get_supported_glsl_version());

    let mut imgui = ImGuiSupport::init();
    let mut imgui_renderer = imgui::glium_renderer::Renderer::init(&mut imgui.imgui, &display).unwrap();

    let control_points = vec![Point::new(1.0, 0.0), Point::new(1.0, 1.0), Point::new(0.0, 1.0)];
    let curve = Bezier::new(control_points);
    let step_size = 0.1;
    let t_range = (0.0, 1.0);
    let steps = ((t_range.1 - t_range.0) / step_size) as usize;
    let mut points = Vec::with_capacity(steps);
    for s in 0..steps + 1 {
        let t = step_size * s as f32 + t_range.0;
        points.push(curve.point(t));
    }

    let mut camera = Camera2d::new();
    let projection = cgmath::ortho(width as f32 / -200.0, width as f32 / 200.0, height as f32 / -200.0,
                                   height as f32 / 200.0, -1.0, -10.0);
    let vertex_buffer = VertexBuffer::new(&display, &points[..]).unwrap();
    let indices = NoIndices(PrimitiveType::LineStrip);
    let draw_params = Default::default();
    let shader_program = program!(&display,
        330 => {
            vertex: "
                #version 330 core
                uniform mat4 view;
                uniform mat4 projection;
                in vec2 pos;
                void main(void) {
                    gl_Position = projection * view * vec4(pos, 2.0, 1.0);
                }
                ",
            fragment: "
                #version 330 core
                out vec4 color;
                void main(void) {
                    color = vec4(0.7, 0.7, 0.1, 1);
                }
            "
        },
    ).unwrap();

    'outer: loop {
        for e in display.poll_events() {
            match e {
                glutin::Event::Closed => break 'outer,
                Event::KeyboardInput(state, _, code) => {
                    let pressed = state == ElementState::Pressed;
                    match code {
                        Some(VirtualKeyCode::Escape) if pressed => break 'outer,
                        Some(VirtualKeyCode::W) if pressed => camera.zoom(-0.01),
                        Some(VirtualKeyCode::S) if pressed => camera.zoom(0.01),
                        _ => {}
                    }
                },
                Event::MouseMoved(x, y) if imgui.mouse_pressed.0 => {
                    let fbscale = imgui.imgui.display_framebuffer_scale();
                    let delta = ((x - imgui.mouse_pos.0) as f32 / (fbscale.0 * 100.0),
                                 -(y - imgui.mouse_pos.1) as f32 / (fbscale.1 * 100.0));
                    camera.translate(delta.0, delta.1);
                },
                _ => {}
            }
            imgui.update_event(&e);
            if imgui.mouse_wheel != 0.0 {
                let fbscale = imgui.imgui.display_framebuffer_scale();
                camera.zoom(imgui.mouse_wheel / (fbscale.1 * 2.0));
            }
        }
        imgui.update_mouse();

        let mut target = display.draw();
        target.clear_color(0.2, 0.2, 0.2, 1.0);

        let cam: [[f32; 4]; 4] = camera.get_mat4().into();
        let proj: [[f32; 4]; 4] = projection.into();
        let uniforms = uniform! {
            projection: proj,
            view: cam,
        };

        // Draw the control points
        target.draw(&vertex_buffer, &indices, &shader_program, &uniforms, &draw_params).unwrap();

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

