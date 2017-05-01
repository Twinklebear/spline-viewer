#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate imgui_sys;
extern crate imgui_glium_renderer;
extern crate cgmath;
extern crate docopt;
extern crate num_traits;
extern crate rulinalg;
extern crate serde;
extern crate serde_json;
extern crate arcball;

mod imgui_support;
mod bezier;
mod bspline;
mod point;
mod camera2d;
mod display_curve;
mod display_curve3d;
mod polyline;
mod bspline_surf;
mod display_surf;
mod display_surf_interp;
mod bspline_basis;

use std::fs::File;
use std::io::BufReader;
use std::f32;

use glium::{DisplayBuild, Surface, DrawParameters};
use glium::glutin::{self, ElementState, Event, VirtualKeyCode, MouseButton};
use cgmath::{SquareMatrix, Transform, Vector2, Matrix4};
use docopt::Docopt;
use imgui_glium_renderer::Renderer;
use arcball::ArcballCamera;

use imgui_support::ImGuiSupport;
use bspline::BSpline;
use bspline_surf::BSplineSurf;
use point::Point;
use camera2d::Camera2d;
use display_curve::DisplayCurve;
use display_curve3d::DisplayCurve3D;
use display_surf::DisplaySurf;
use display_surf_interp::DisplaySurfInterpolation;

/// Import a 2D BSpline curve from the file
fn import_bspline(json: &serde_json::Value) -> BSpline<Point> {
    let degree = json["degree"].as_u64().expect("A curve degree must be specified") as usize;
    let points = json["points"].as_array().expect("A list of points must be specified").iter()
        .map(|p| Point::new(p["x"].as_f64().expect("Invalid x coord") as f32,
                            p["y"].as_f64().expect("Invalid y coord") as f32,
                            p["z"].as_f64().unwrap_or(0.0) as f32)).collect();
    let mut knots = Vec::new();
    if let Some(k) = json["knots"].as_array() {
        knots = k.iter().map(|x| x.as_f64().expect("Invalid knot value") as f32).collect();
    }
    println!("degree = {:?}", degree);
    println!("points = {:?}", points);
    println!("knots = {:?}", knots);
    BSpline::new(degree, points, knots)
}

/// Import a B-spline surface file
fn import_surf(json: &serde_json::Value) -> BSplineSurf<Point> {
    let u_data = json["u"].as_object().expect("Surface u component is required");
    let v_data = json["v"].as_object().expect("Surface v component is required");

    let degree_u = u_data["degree"].as_u64().expect("Surface u degree is required") as usize;
    let degree_v = v_data["degree"].as_u64().expect("Surface v degree is required") as usize;

    let knots_u = u_data["knots"].as_array().expect("Surface u knots are required").iter()
        .map(|x| x.as_f64().expect("Invalid knot value") as f32).collect();
    let knots_v = v_data["knots"].as_array().expect("Surface v knots are required").iter()
        .map(|x| x.as_f64().expect("Invalid knot value") as f32).collect();

    let mut mesh = Vec::new();
    for r in json["mesh"].as_array().expect("Surface control mesh is required") {
        let points = r.as_array().expect("A list of points must be specified").iter()
            .map(|p| Point::new(p["x"].as_f64().expect("Invalid x coord") as f32,
                                p["y"].as_f64().expect("Invalid y coord") as f32,
                                p["z"].as_f64().expect("Invalid z coord") as f32)).collect();
        mesh.push(points);
    }
    BSplineSurf::new((degree_u, degree_v), (knots_u, knots_v), mesh)
}

/// Import a B-spline nodal interpolation data file
/// Note: for the assignment we only did interpolation on one axis, so it assumes
/// the passed control points are the curve along v's control points
fn import_surf_interpolation(json: &serde_json::Value) -> Vec<BSpline<Point>> {
    let u_data = json["u"].as_object().expect("Surface u component is required");
    let degree_u = u_data["degree"].as_u64().expect("Surface u degree is required") as usize;
    let knots_u: Vec<f32> = u_data["knots"].as_array().expect("Surface u knots are required").iter()
        .map(|x| x.as_f64().expect("Invalid knot value") as f32).collect();

    let mut splines = Vec::new();
    for r in json["mesh"].as_array().expect("Surface control mesh is required") {
        let points = r.as_array().expect("A list of points must be specified").iter()
            .map(|p| Point::new(p["x"].as_f64().expect("Invalid x coord") as f32,
                                p["y"].as_f64().expect("Invalid y coord") as f32,
                                p["z"].as_f64().expect("Invalid z coord") as f32)).collect();
        splines.push(BSpline::new(degree_u, points, knots_u.clone()));
    }
    splines
}

const USAGE: &'static str = "
Usage:
    bezier [<file>...]
    bezier (-h | --help)

Options:
    -h, --help      Show this message.
";

fn main() {
    let args = Docopt::new(USAGE).and_then(|d| d.parse()).unwrap_or_else(|e| e.exit());
    let target_gl_versions = glutin::GlRequest::GlThenGles {
        opengl_version: (3, 3),
        opengles_version: (3, 2),
    };
    let mut width = 1280;
    let mut height = 720;
    let display = glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_gl(target_gl_versions)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_title("CS6670 Programming Assignment 1 - Will Usher")
        .with_vsync()
        .build_glium().unwrap();

    let mut curves = Vec::new();
    let mut curves3d = Vec::new();
    let mut surfaces = Vec::new();
    let mut surface_interpolations = Vec::new();
    for f in args.get_vec("<file>") {
        let file = match File::open(&f) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open file: {}", e),
        };
        let reader = BufReader::new(file);
        let json: serde_json::Value  = serde_json::from_reader(reader).expect("Failed to read input file");
        let ty = json["type"].as_str().expect("A curve type must be specified");
        if ty == "bspline2d" {
            curves.push(DisplayCurve::new(import_bspline(&json), &display));
        } else if ty == "bspline3d" {
            curves3d.push(DisplayCurve3D::new(import_bspline(&json), &display));
        } else if ty == "surface" {
            surfaces.push(DisplaySurf::new(import_surf(&json), &display));
        } else if ty == "interpolation_u" {
            surface_interpolations.push(DisplaySurfInterpolation::new(import_surf_interpolation(&json), &display));
        } else {
            println!("Unrecognized file type header {}", ty);
        }
    }

    println!("Got OpenGL: {:?}", display.get_opengl_version());
    println!("Got GLSL: {:?}", display.get_supported_glsl_version());

    let mut imgui = ImGuiSupport::init();
    let mut imgui_renderer = Renderer::init(&mut imgui.imgui, &display).unwrap();

    let mut camera_2d = Camera2d::new();
    let mut arcball_camera = {
        use cgmath::{Point3, Vector3};
        let look_at = Matrix4::<f32>::look_at(Point3::new(0.0, 0.0, 6.0),
                                              Point3::new(0.0, 0.0, 0.0),
                                              Vector3::new(0.0, 1.0, 0.0));
        let inv_screen = [1.0 / width as f32, 1.0 / height as f32];
        ArcballCamera::new(&look_at, 1.0, 5.0, inv_screen)
    };

    let mut ortho_proj = cgmath::ortho(width as f32 / -200.0, width as f32 / 200.0, height as f32 / -200.0,
                                       height as f32 / 200.0, -0.01, -100.0);
    let mut persp_proj = cgmath::perspective(cgmath::Deg(65.0), width as f32 / height as f32, 0.01, 100.0);
    let draw_params = DrawParameters {
        point_size: Some(6.0),
        .. Default::default()
    };
    let shader_program = program!(&display,
        330 => {
            vertex: "
                #version 330 core
                uniform mat4 proj_view;
                in vec3 pos;
                void main(void) {
                    gl_Position = proj_view * vec4(pos, 1.0);
                }
                ",
            fragment: "
                #version 330 core
                uniform vec3 pcolor;
                out vec4 color;
                void main(void) {
                    color = vec4(pcolor, 1);
                }
            "
        },
    ).unwrap();

    let mut shift_down = false;
    let mut selected_curve: i32 = 0;
    let mut ui_interaction = false;
    let mut color_attenuation = true;
    let mut render_3d = true;
    'outer: loop {
        let fbscale = imgui.imgui.display_framebuffer_scale();
        for e in display.poll_events() {
            match e {
                glutin::Event::Closed => break 'outer,
                Event::KeyboardInput(state, _, code) => {
                    let pressed = state == ElementState::Pressed;
                    match code {
                        Some(VirtualKeyCode::Escape) if pressed => break 'outer,
                        Some(VirtualKeyCode::RShift) => shift_down = pressed,
                        Some(VirtualKeyCode::LShift) => shift_down = pressed,
                        _ => {}
                    }
                },
                Event::MouseMoved(x, y) if imgui.mouse_pressed.1 && !ui_interaction && !render_3d => {
                    let delta = ((x - imgui.mouse_pos.0) as f32 / (fbscale.0 * 100.0),
                    -(y - imgui.mouse_pos.1) as f32 / (fbscale.1 * 100.0));
                    camera_2d.translate(delta.0, delta.1);
                },
                Event::MouseMoved(x, y) if !ui_interaction && render_3d => {
                    if imgui.mouse_pressed.0 {
                        arcball_camera.rotate(Vector2::new(imgui.mouse_pos.0 as f32, imgui.mouse_pos.1 as f32),
                                              Vector2::new(x as f32, y as f32));
                    } else if imgui.mouse_pressed.1 {
                        let mouse_delta = Vector2::new((x - imgui.mouse_pos.0) as f32,
                                                       -(y - imgui.mouse_pos.1) as f32);
                        arcball_camera.pan(mouse_delta, 0.16);
                    }
                },
                Event::MouseInput(state, button) => {
                    if !render_3d && state == ElementState::Released
                        && button == MouseButton::Left && selected_curve < curves.len() as i32
                        {
                            curves[selected_curve as usize].release_point();
                        }
                },
                Event::Resized(w, h) => {
                    width = w;
                    height = h;
                    ortho_proj = cgmath::ortho(width as f32 / -200.0, width as f32 / 200.0,
                                               height as f32 / -200.0, height as f32 / 200.0, -1.0, -1000.0);
                    persp_proj = cgmath::perspective(cgmath::Deg(65.0), width as f32 / height as f32, 1.0, 1000.0);
                    arcball_camera.update_screen(width as f32, height as f32);
                },
                Event::DroppedFile(ref p) => {
                    let file = match File::open(p) {
                        Ok(f) => f,
                        Err(e) => panic!("Failed to open file: {}", e),
                    };
                    let reader = BufReader::new(file);
                    let json: serde_json::Value  = serde_json::from_reader(reader).expect("Failed to read input file");
                    let ty = json["type"].as_str().expect("A curve type must be specified");
                    if ty == "bspline2d" {
                        curves.push(DisplayCurve::new(import_bspline(&json), &display));
                    } else if ty == "bspline3d" {
                        curves3d.push(DisplayCurve3D::new(import_bspline(&json), &display));
                    } else if ty == "surface" {
                        surfaces.push(DisplaySurf::new(import_surf(&json), &display));
                    } else if ty == "interpolation_u" {
                        surface_interpolations.push(DisplaySurfInterpolation::new(import_surf_interpolation(&json), &display));
                    } else {
                        println!("Unrecognized file type header {}", ty);
                    }
                },
                _ => {}
            }
            imgui.update_event(&e);
        }
        if !ui_interaction {
            if render_3d {
                if imgui.mouse_wheel != 0.0 {
                    arcball_camera.zoom(imgui.mouse_wheel / (fbscale.1 * 10.0), 0.16);
                }
            } else {
                if imgui.mouse_wheel != 0.0 {
                    camera_2d.zoom(imgui.mouse_wheel / (fbscale.1 * 10.0));
                }
                if imgui.mouse_pressed.0 && selected_curve < curves.len() as i32 {
                    let unproj = (ortho_proj * camera_2d.get_mat4()).invert().expect("Uninvertable proj * view!?");
                    let click_pos =
                        cgmath::Point3::<f32>::new(2.0 * imgui.mouse_pos.0 as f32 / width as f32 - 1.0,
                                                   -2.0 * imgui.mouse_pos.1 as f32 / height as f32 + 1.0,
                                                   0.0);
                    let pos = unproj.transform_point(click_pos);
                    let pos = Point::new(pos.x, pos.y, 0.0);
                    curves[selected_curve as usize].handle_click(pos, shift_down, camera_2d.zoom);
                }
            }
        }
        imgui.update_mouse();

        ui_interaction = imgui_support::is_mouse_hovering_any_window() || imgui_support::is_any_item_active();

        let mut target = display.draw();
        target.clear_color(0.05, 0.05, 0.05, 1.0);

        let proj_view: [[f32; 4]; 4] =
            if !render_3d {
                (ortho_proj * camera_2d.get_mat4()).into()
            } else {
                (persp_proj * arcball_camera.get_mat4()).into()
            };
        let attenuation = if color_attenuation { 0.4 } else { 1.0 };

        for (i, c) in curves.iter().enumerate() {
            c.render(&mut target, &shader_program, &draw_params, &proj_view, i as i32 == selected_curve,
                     attenuation);
        }
        for (i, c) in curves3d.iter().enumerate() {
            let sel_curve = selected_curve - curves.len() as i32;
            c.render(&mut target, &shader_program, &draw_params, &proj_view, i as i32 == sel_curve,
                     attenuation);
        }
        for (i, s) in surfaces.iter().enumerate() {
            let sel_curve = selected_curve - curves.len() as i32 - curves3d.len() as i32;
            s.render(&mut target, &shader_program, &draw_params, &proj_view, i as i32 == sel_curve,
                     attenuation);
        }
        for (i, s) in surface_interpolations.iter().enumerate() {
            let sel_curve = selected_curve - curves.len() as i32 - curves3d.len() as i32 - surfaces.len() as i32;
            s.render(&mut target, &shader_program, &draw_params, &proj_view, i as i32 == sel_curve,
                     attenuation);
        }

        let ui = imgui.render_ui(&display);
        ui.window(im_str!("Curve Control Panel"))
            .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                let fps = ui.framerate();
                let frame_time = 1000.0 / fps;
                let gl_version = display.get_opengl_version();
                let glsl_version = display.get_supported_glsl_version();
                ui.text(im_str!("Framerate: {:.3} FPS ({:.3} ms)", fps, frame_time));
                ui.text(im_str!("OpenGL Version: {}.{}", gl_version.1, gl_version.2));
                ui.text(im_str!("GLSL Version: {}.{}", glsl_version.1, glsl_version.2));
                ui.checkbox(im_str!("Fade Unselected Curves"), &mut color_attenuation);
                ui.checkbox(im_str!("Render 3D"), &mut render_3d);

                let mut removing = None;
                for (i, c) in curves.iter_mut().enumerate() {
                    ui.separator();
                    imgui_support::push_id_int(i as i32);
                    imgui_support::radio_button(im_str!("Select Curve"), &mut selected_curve, i as i32);
                    c.draw_ui(&ui);
                    if ui.small_button(im_str!("Remove Curve")) {
                        removing = Some(i);
                    }
                    imgui_support::pop_id();
                }
                for (i, c) in curves3d.iter_mut().enumerate() {
                    let id = i + curves.len();
                    ui.separator();
                    imgui_support::push_id_int(id as i32);
                    imgui_support::radio_button(im_str!("Select Curve"), &mut selected_curve, id as i32);
                    c.draw_ui(&ui);
                    if ui.small_button(im_str!("Remove Curve")) {
                        removing = Some(id);
                    }
                    imgui_support::pop_id();
                }
                for (i, c) in surfaces.iter_mut().enumerate() {
                    let id = i + curves.len() + curves3d.len();
                    ui.separator();
                    imgui_support::push_id_int(id as i32);
                    imgui_support::radio_button(im_str!("Select Surface"), &mut selected_curve, id as i32);
                    c.draw_ui(&ui);
                    if ui.small_button(im_str!("Remove Surface")) {
                        removing = Some(id);
                    }
                    imgui_support::pop_id();
                }
                for (i, c) in surface_interpolations.iter_mut().enumerate() {
                    let id = i + curves.len() + curves3d.len() + surfaces.len();
                    ui.separator();
                    imgui_support::push_id_int(id as i32);
                    imgui_support::radio_button(im_str!("Select Surface"), &mut selected_curve, id as i32);
                    c.draw_ui(&ui);
                    if ui.small_button(im_str!("Remove Surface")) {
                        removing = Some(id);
                    }
                    imgui_support::pop_id();
                }

                if let Some(i) = removing {
                    if selected_curve as usize >= i && selected_curve != 0 {
                        selected_curve -= 1;
                    }
                    if i >= curves.len() + curves3d.len() + surfaces.len() {
                        surface_interpolations.remove(i - curves.len() - curves3d.len() - surfaces.len());
                    } else if i >= curves.len() + curves3d.len() {
                        surfaces.remove(i - curves.len() - curves3d.len());
                    } else if i >= curves.len() {
                        curves3d.remove(i - curves.len());
                    } else {
                        curves.remove(i);
                    }
                }
                if ui.small_button(im_str!("Add Curve")) {
                    curves.push(DisplayCurve::new(BSpline::empty(), &display));
                    selected_curve = (curves.len() - 1) as i32;
                }
            });
        imgui_renderer.render(&mut target, ui).unwrap();

        target.finish().unwrap();
    }
}

