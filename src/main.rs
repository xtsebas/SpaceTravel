use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::sync::Arc;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod uniforms;
mod light;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use shaders::{vertex_shader, select_shader};
use uniforms::{Uniforms, create_noise, create_model_matrix, create_view_matrix, create_perspective_matrix, create_viewport_matrix};

struct Planet {
    name: &'static str,
    distance_from_sun: f32,
    radius: f32,
    orbit_speed: f32,
    color_index: usize,
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], index: usize) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle::triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = select_shader(index, &fragment, &uniforms);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn render_saturn_rings(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], index: usize) {
    let num_rings = 50;
    let radius = 10.0;
    let y_offset = 3.0;

    let saturn_position = Vec3::new(0.0, 0.0, 0.0);

    for i in 0..num_rings {
        let angle = 2.0 * PI * i as f32 / num_rings as f32;
        let ring_translation = Vec3::new(radius * angle.cos(), y_offset, radius * angle.sin()) + saturn_position;

        let mut ring_uniforms = uniforms.clone();
        ring_uniforms.model_matrix = create_model_matrix(ring_translation, 0.2, Vec3::new(0.0, 0.0, 0.0));

        render(framebuffer, &ring_uniforms, vertex_array, index);
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Solar System Simulation",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    framebuffer.set_background_color(0x000000);

    let mut camera = Camera::new(
        Vec3::new(50.0, 100.0, 250.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Cargar modelos para los planetas y Saturno
    let sphere_obj = Obj::load("assets/model/sphere.obj").expect("Failed to load sphere.obj");
    let sphere_vertex_arrays = sphere_obj.get_vertex_array();

    let rings_obj = Obj::load("assets/model/rings.obj").expect("Failed to load rings.obj");
    let rings_vertex_arrays = rings_obj.get_vertex_array();

    let noise = Arc::new(create_noise());
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

    let mut uniforms = Uniforms {
        model_matrix: Mat4::identity(),
        view_matrix: Mat4::identity(),
        projection_matrix,
        viewport_matrix,
        time: 0,
        noise: noise.clone(),
    };

    let planets = vec![
        Planet { name: "Sol", distance_from_sun: 0.0, radius: 3.0, orbit_speed: 0.001, color_index: 0 },
        Planet { name: "Mercurio", distance_from_sun: 20.0, radius: 0.5, orbit_speed: 0.003, color_index: 1 },
        Planet { name: "Venus", distance_from_sun: 40.0, radius: 0.8, orbit_speed: 0.005, color_index: 2 },
        Planet { name: "Tierra", distance_from_sun: 60.0, radius: 1.0, orbit_speed: 0.007, color_index: 3 },
        Planet { name: "Marte", distance_from_sun: 80.0, radius: 0.7, orbit_speed: 0.009, color_index: 4 },
        Planet { name: "Júpiter", distance_from_sun: 100.0, radius: 2.0, orbit_speed: 0.001, color_index: 5 },
        Planet { name: "Saturno", distance_from_sun: 120.0, radius: 1.8, orbit_speed: 0.003, color_index: 6 },
        Planet { name: "Urano", distance_from_sun: 140.0, radius: 1.5, orbit_speed: 0.005, color_index: 7 },
    ];

    let mut time = 0.0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(&window, &mut camera);

        framebuffer.clear();

        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);

        for planet in &planets {
            let angle = planet.orbit_speed * time;
            let translation = Vec3::new(
                planet.distance_from_sun * angle.cos(),
                0.0,
                planet.distance_from_sun * angle.sin(),
            );

            uniforms.model_matrix = create_model_matrix(translation, planet.radius, Vec3::new(0.0, 0.0, 0.0));

            // Usar el modelo correcto para Saturno
            if planet.name == "Saturno" {
                render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, planet.color_index);

                let y_offset = 6.0;

                // Ajustar la posición de los anillos
                let rings_translation = Vec3::new(
                    translation.x, 
                    translation.y + y_offset, // Mismo centro en el eje Y
                    translation.z,
                );
                let rings_scale = 3.5; // Escalar los anillos para ajustarse alrededor de la esfera

                uniforms.model_matrix = create_model_matrix(rings_translation, rings_scale, Vec3::new(0.0, 0.0, 0.0));
                render(&mut framebuffer, &uniforms, &rings_vertex_arrays, 8);
            } else {
                render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, planet.color_index);
            }
        }

        time += 1.0;

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}


fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 1.0;

    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}