#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate imgui_sys;
extern crate cgmath;
extern crate docopt;
extern crate rustc_serialize;
extern crate regex;
extern crate num_traits;

mod imgui_support;
mod bezier;
mod bspline;
mod point;
mod camera2d;
mod display_curve;
mod polyline;
mod arcball_camera;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::f32;
use std::iter;
use std::ffi::OsStr;

use glium::{DisplayBuild, Surface, DrawParameters};
use glium::glutin::{self, ElementState, Event, VirtualKeyCode, MouseButton};
use glium::backend::glutin_backend::GlutinFacade;
use cgmath::{SquareMatrix, Transform, Vector2, Matrix4};
use docopt::Docopt;
use regex::Regex;

use imgui_support::ImGuiSupport;
use bezier::Bezier;
use bspline::BSpline;
use point::Point;
use camera2d::Camera2d;
use display_curve::DisplayCurve;
use polyline::Polyline;
use arcball_camera::ArcballCamera;

/// Import a list of BSpline curves from the file
fn import<P: AsRef<Path>>(path: P) -> BSpline<Point> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };
    let reader = BufReader::new(file);
    let mut points = Vec::new();
    let mut knots = Vec::new();
    let mut degree: Option<usize> = None;
    let mut num_points = 0;
    let mut pts_read = 0;
    let mut read_knots = false;
    for line in reader.lines() {
        let l = line.unwrap();
        // Skip empty lines and comments
        if l.is_empty() || l.starts_with("#") {
            continue;
        }
        if degree.is_none() {
            degree = Some(l.trim().parse().unwrap());
            println!("Curve has degree {}", degree.expect("no degree set"));
            continue;
        }
        if num_points == 0 {
            num_points = l.trim().parse().unwrap();
            println!("Expecting {} points for control polygon", num_points);
            continue;
        }
        if pts_read < num_points {
            let coords: Vec<_> = l.split(',').collect();
            assert!(coords.len() >= 2);
            let x = coords[0].trim().parse().unwrap();
            let y = coords[1].trim().parse().unwrap();
            points.push(Point::new(x, y));
            pts_read += 1;
            continue;
        }
        if read_knots {
            let coords: Vec<_> = l.split(',').collect();
            for k in coords {
                knots.push(k.trim().parse().unwrap());
            }
            break;
        }
        let knots_provided: usize = l.trim().parse().unwrap();
        println!("knots provided? {}", knots_provided == 1);
        if knots_provided == 0 {
            break;
        }
        read_knots = true;
    }
    BSpline::new(degree.expect("No degree specified"), points, knots)
}
/// Import a list of points from the file
fn import_points<P: AsRef<Path>>(path: P) -> Vec<Point> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file: {}", e),
    };
    let reader = BufReader::new(file);
    let curve_start = Regex::new("(P|Q), *(\\d+)").unwrap();
    let mut points = Vec::new();
    let mut rational_points = false;
    let mut num_curves = 0;
    for line in reader.lines() {
        let l = line.unwrap();
        // Skip empty lines and comments
        if l.is_empty() || l.starts_with("#") {
            continue;
        }
        if num_curves == 0 {
            num_curves = l.parse().unwrap();
            println!("Expecting {} set of points from the file", num_curves);
            if num_curves > 1 {
                println!("More than 1 set of point!? Makes no sense?");
            }
            continue;
        }
        if let Some(caps) = curve_start.captures(&l[..]) {
            if caps.at(1) == Some("Q") {
                rational_points = true;
                println!("Expecting {} rational points in file", caps.at(2).unwrap());
            } else {
                rational_points = false;
                println!("Expecting {} polynomial points in file", caps.at(2).unwrap());
            }
            continue;
        }
        let coords: Vec<_> = l.split(',').collect();
        assert!(coords.len() >= 2);
        let x = coords[0].trim().parse().unwrap();
        let y = coords[1].trim().parse().unwrap();
        if rational_points {
            //let w = coords[2].trim().parse().unwrap();
            //x /= w;
            //y /= w;
        }
        points.push(Point::new(x, y));
    }
    points
}

const USAGE: &'static str = "
Usage:
    bezier [<file>...]
    bezier (-h | --help)

Options:
    -h, --help      Show this message.
";

#[derive(RustcDecodable)]
struct Args {
    arg_file: Option<Vec<String>>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
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

    let mut polyline_data = None;
    let mut curves = Vec::new();
    if let Some(files) = args.arg_file {
        for f in files {
            let p = Path::new(&f);
            if p.extension() == Some(OsStr::new("dat")) {
                curves.push(DisplayCurve::new(import(p), &display));;
            } else if p.extension() == Some(OsStr::new("crv")) {
                let imported_points = import_points(p);
                polyline_data = Some(Polyline::new(imported_points, &display));
            } else {
                println!("Unrecognized file type {}", f);
            }
        }
    }

    println!("Got OpenGL: {:?}", display.get_opengl_version());
    println!("Got GLSL: {:?}", display.get_supported_glsl_version());

    let mut imgui = ImGuiSupport::init();
    let mut imgui_renderer = imgui::glium_renderer::Renderer::init(&mut imgui.imgui, &display).unwrap();

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
                                       height as f32 / 200.0, -1.0, -1000.0);
    let mut persp_proj = cgmath::perspective(cgmath::Deg(65.0), width as f32 / height as f32, 1.0, 1000.0);
    let draw_params = DrawParameters {
        point_size: Some(6.0),
        .. Default::default()
    };
    let shader_program = program!(&display,
        330 => {
            vertex: "
                #version 330 core
                uniform mat4 proj_view;
                in vec2 pos;
                void main(void) {
                    gl_Position = proj_view * vec4(pos, 0.0, 1.0);
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
    let mut render_3d = false;
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
                                              Vector2::new(x as f32, y as f32), 0.16);
                    } else if imgui.mouse_pressed.1 {
                        let mouse_delta = Vector2::new((x - imgui.mouse_pos.0) as f32, -(y - imgui.mouse_pos.1) as f32);
                        arcball_camera.pan(mouse_delta, 0.16);
                    }
                },
                Event::MouseInput(state, button) => {
                    if !render_3d && state == ElementState::Released
                        && button == MouseButton::Left && !curves.is_empty()
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
                    if p.extension() == Some(OsStr::new("dat")) {
                        curves.push(DisplayCurve::new(import(p), &display));
                    } else if p.extension() == Some(OsStr::new("crv")) {
                        let imported_points = import_points(p);
                        polyline_data = Some(Polyline::new(imported_points, &display));
                    } else {
                        println!("Unrecognized file type {}", p.display());
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
                if imgui.mouse_pressed.0 && !curves.is_empty() {
                    let unproj = (ortho_proj * camera_2d.get_mat4()).invert().expect("Uninvertable proj * view!?");
                    let click_pos =
                        cgmath::Point3::<f32>::new(2.0 * imgui.mouse_pos.0 as f32 / width as f32 - 1.0,
                                                   -2.0 * imgui.mouse_pos.1 as f32 / height as f32 + 1.0,
                                                   0.0);
                    let pos = unproj.transform_point(click_pos);
                    let pos = Point::new(pos.x, pos.y);
                    curves[selected_curve as usize].handle_click(pos, shift_down, camera_2d.zoom);
                }
            }
        }
        imgui.update_mouse();

        ui_interaction = imgui_support::is_mouse_hovering_any_window() || imgui_support::is_any_item_active();

        let mut target = display.draw();
        target.clear_color(0.1, 0.1, 0.1, 1.0);

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
        if let Some(ref pl) = polyline_data {
            pl.render(&mut target, &shader_program, &draw_params, &proj_view);
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
                ui.popup(im_str!("curves_saved"), || ui.text(im_str!("Curves saved")));
                ui.popup(im_str!("need_file_name"), || ui.text(im_str!("A file name is required")));
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
                if let Some(i) = removing {
                    if selected_curve as usize >= i && selected_curve != 0 {
                        selected_curve -= 1;
                    }
                    curves.remove(i);
                }
                if ui.small_button(im_str!("Add Curve")) {
                    curves.push(DisplayCurve::new(BSpline::empty(), &display));
                    selected_curve = (curves.len() - 1) as i32;
                }
            });
        if let Some(ref mut pl) = polyline_data {
            ui.window(im_str!("Polyline Data"))
                .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
                .build(|| pl.draw_ui(&ui));
        }
        imgui_renderer.render(&mut target, ui).unwrap();

        target.finish().unwrap();
    }
}

