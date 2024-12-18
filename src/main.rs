use nalgebra_glm::{Vec3, Mat4, Vec4};
use nalgebra::{Vector4};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::sync::Arc;
use std::path::Path;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use image::{open, DynamicImage, GenericImageView};

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

#[derive(PartialEq)]
struct Planet {
    name: &'static str,
    distance_from_sun: f32,
    radius: f32,
    orbit_speed: f32,
    color_index: usize,
}

fn load_texture(file_path: &str) -> DynamicImage {
    image::open(Path::new(file_path)).expect("Failed to load texture")
}


fn render_skybox(framebuffer: &mut Framebuffer, skybox_texture: &DynamicImage) {
    let (texture_width, texture_height) = skybox_texture.dimensions();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // Mapear las coordenadas del framebuffer a las coordenadas de la textura
            let tex_x = (x as f32 / (framebuffer.width - 1) as f32 * (texture_width - 1) as f32) as u32;
            let tex_y = (y as f32 / (framebuffer.height - 1) as f32 * (texture_height - 1) as f32) as u32;

            // Obtener el color del píxel de la textura
            let pixel = skybox_texture.get_pixel(tex_x, tex_y);
            let color = (pixel[0] as u32) << 16 | (pixel[1] as u32) << 8 | (pixel[2] as u32);

            // Escribir el color en el framebuffer con profundidad máxima
            let index = y * framebuffer.width + x;
            framebuffer.buffer[index] = color;
            framebuffer.zbuffer[index] = std::f32::INFINITY; // Profundidad máxima
        }
    }
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

fn draw_orbit(
    framebuffer: &mut Framebuffer,
    planet: &Planet,
    uniforms: &Uniforms,
    segments: usize,
    color: u32,
) {
    let mut previous_screen_point = None;

    for i in 0..=segments {
        let angle = 2.0 * PI * (i as f32 / segments as f32);

        // Calcular la posición 3D del punto en la órbita
        let orbit_point = Vec3::new(
            planet.distance_from_sun * angle.cos(),
            0.0,
            planet.distance_from_sun * angle.sin(),
        );


        // Transformar el punto
        let transformed_point = uniforms.viewport_matrix
            * uniforms.projection_matrix
            * uniforms.view_matrix
            * Vec4::new(orbit_point.x, orbit_point.y, orbit_point.z, 1.0);

        // Convertir a coordenadas de pantalla
        if transformed_point.w != 0.0 {
            let screen_x = ((transformed_point.x / transformed_point.w + 1.0) * 0.5 * framebuffer.width as f32) as isize;
            let screen_y = ((1.0 - (transformed_point.y / transformed_point.w + 1.0) * 0.5) * framebuffer.height as f32) as isize;


            // Validar y dibujar
            if screen_x >= 0 && screen_y >= 0 {
                let screen_x = screen_x as usize;
                let screen_y = screen_y as usize;

                if let Some((prev_x, prev_y)) = previous_screen_point {
                    framebuffer.draw_line(prev_x, prev_y, screen_x, screen_y, color);
                }

                previous_screen_point = Some((screen_x, screen_y));
            }
        }
    }
}

fn lerp(start: Vec3, end: Vec3, t: f32) -> Vec3 {
    start * (1.0 - t) + end * t
}


fn is_in_camera_view(camera: &Camera, object_position: Vec3, object_radius: f32) -> bool {
    let view_vector = (object_position - camera.eye).normalize();
    let camera_forward = (camera.center - camera.eye).normalize();
    let dot_product = view_vector.dot(&camera_forward);

    // Convertir el FOV de grados a radianes y calcular el coseno del ángulo
    let fov_radians = camera.fov.to_radians() / 2.0;
    dot_product > fov_radians.cos()
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
        Vec3::new(50.0, 100.0, 250.0), // Posición inicial
        Vec3::new(0.0, 0.0, 0.0),     // Centro (sol)
        Vec3::new(0.0, 1.0, 0.0),
    );

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
        Planet { name: "Sol", distance_from_sun: 0.0, radius: 3.0, orbit_speed: 0.0, color_index: 0 },
        Planet { name: "Mercurio", distance_from_sun: 20.0, radius: 0.5, orbit_speed: 0.003, color_index: 1 },
        Planet { name: "Venus", distance_from_sun: 40.0, radius: 0.8, orbit_speed: 0.005, color_index: 2 },
        Planet { name: "Tierra", distance_from_sun: 60.0, radius: 1.0, orbit_speed: 0.007, color_index: 3 },
        Planet { name: "Marte", distance_from_sun: 80.0, radius: 0.7, orbit_speed: 0.009, color_index: 4 },
        Planet { name: "Júpiter", distance_from_sun: 100.0, radius: 2.0, orbit_speed: 0.001, color_index: 5 },
        Planet { name: "Saturno", distance_from_sun: 120.0, radius: 1.8, orbit_speed: 0.003, color_index: 6 },
        Planet { name: "Urano", distance_from_sun: 140.0, radius: 1.5, orbit_speed: 0.005, color_index: 7 },
    ];

    let mut focused_planet: Option<&Planet> = None;
    let mut bird_eye_view = false;
    let skybox_texture = load_texture("assets/space.png");
    let mut prev_mouse_x = None;
    let mut mouse_active = false;
    let mut transitioning = false;
    let mut transition_target_eye = camera.eye;
    let mut transition_target_center = camera.center;
    let mut transition_speed = 0.05;
    let mut time = 0.0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Alternar entre la vista normal y la "bird's eye view"
        if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) {
            bird_eye_view = !bird_eye_view;
            if bird_eye_view {
                transition_target_eye = Vec3::new(0.0, 500.0, 200.0);
                transition_target_center = Vec3::new(0.0, 0.0, 0.0);
                transitioning = true;
            } else {
                transition_target_eye = Vec3::new(50.0, 100.0, 250.0);
                transition_target_center = Vec3::new(0.0, 0.0, 0.0);
                transitioning = true;
            }
        }

        if !bird_eye_view && !transitioning {
            // Permitir el control de la cámara solo si no estamos en "bird's eye view" y no estamos en transición
            handle_input(&window, &mut camera, &planets, &mut prev_mouse_x, &mut mouse_active);
        }

        // Detectar teclas para enfoque en un planeta
        let planet_key_map = vec![
            (Key::M, &planets[1]), // Mercurio
            (Key::V, &planets[2]), // Venus
            (Key::E, &planets[3]), // Tierra
            (Key::R, &planets[4]), // Marte
            (Key::J, &planets[5]), // Júpiter
            (Key::N, &planets[6]), // Saturno
            (Key::U, &planets[7]), // Urano
        ];

        for (key, planet) in planet_key_map {
            if window.is_key_pressed(key, minifb::KeyRepeat::No) {
                if focused_planet == Some(planet) {
                    // Si ya está enfocado, volver a la vista general
                    focused_planet = None;
                    transition_target_eye = Vec3::new(50.0, 100.0, 250.0);
                    transition_target_center = Vec3::new(0.0, 0.0, 0.0);
                    transitioning = true;
                } else {
                    // Enfocar en el planeta seleccionado
                    focused_planet = Some(planet);
                    transition_target_eye = Vec3::new(
                        planet.distance_from_sun + 20.0,
                        planet.radius * 2.0,
                        0.0,
                    );
                    transition_target_center = Vec3::new(
                        planet.distance_from_sun,
                        0.0,
                        0.0,
                    );
                    transitioning = true;
                }
            }
        }

        // Interpolar la posición de la cámara durante la transición
        if transitioning {
            camera.eye = lerp(camera.eye, transition_target_eye, transition_speed);
            camera.center = lerp(camera.center, transition_target_center, transition_speed);

            if (camera.eye - transition_target_eye).magnitude() < 0.1
                && (camera.center - transition_target_center).magnitude() < 0.1
            {
                transitioning = false;
            }
        }

        framebuffer.clear();
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        render_skybox(&mut framebuffer, &skybox_texture);

        if let Some(planet) = focused_planet {
            // Renderizar solo el planeta enfocado
            uniforms.model_matrix = create_model_matrix(
                Vec3::new(planet.distance_from_sun, 0.0, 0.0),
                planet.radius,
                Vec3::new(0.0, 0.0, 0.0),
            );

            render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, planet.color_index);

            // Renderizar anillos si es Saturno
            if planet.name == "Saturno" {
                uniforms.model_matrix = create_model_matrix(
                    Vec3::new(planet.distance_from_sun, 0.0, 0.0),
                    3.5, // Tamaño de los anillos
                    Vec3::new(0.0, 0.0, 0.0),
                );
                render_saturn_rings(&mut framebuffer, &uniforms, &rings_vertex_arrays, 8);
            }
        } else {
            // Renderizar todo el sistema solar
            for planet in &planets {
                draw_orbit(&mut framebuffer, planet, &uniforms, 100, 0xAAAAAA);

                let angle = planet.orbit_speed * time;
                let translation = Vec3::new(
                    planet.distance_from_sun * angle.cos(),
                    0.0,
                    planet.distance_from_sun * angle.sin(),
                );

                if is_in_camera_view(&camera, translation, planet.radius) {
                    uniforms.model_matrix = create_model_matrix(translation, planet.radius, Vec3::new(0.0, 0.0, 0.0));
                    render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, planet.color_index);

                    // Renderizar los anillos de Saturno si el planeta es visible
                    if planet.name == "Saturno" {
                        let y_offset = 6.0;
                        let rings_translation = Vec3::new(
                            translation.x,
                            translation.y + y_offset,
                            translation.z,
                        );
                        let rings_scale = 3.5;

                        uniforms.model_matrix = create_model_matrix(rings_translation, rings_scale, Vec3::new(0.0, 0.0, 0.0));
                        render(&mut framebuffer, &uniforms, &rings_vertex_arrays, 8);
                    }
                }
            }
        }

        time += 1.0;

        // Determinar la vista actual
        let current_view = if let Some(planet) = focused_planet {
            planet.name.to_string()
        } else if bird_eye_view {
            "BIRD EYE".to_string()
        } else {
            "NAVE".to_string()
        };

        // Dibujar el texto en la esquina superior izquierda
        let text_color = 0xFFFFFF; // Blanco
        framebuffer.draw_text(10, 10, &current_view, text_color, 3);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }

}


fn handle_input(window: &Window, camera: &mut Camera, planets: &[Planet],  prev_mouse_pos: &mut Option<(f32, f32)>, mouse_active: &mut bool) {
    let movement_speed = 0.022;
    let zoom_speed = 0.5;
    let rotation_speed = PI / 200.0;

    if window.is_key_down(Key::Left) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(rotation_speed, 0.0);
    }


    // Calcular el movimiento lateral (A y D)
    let forward = (camera.center - camera.eye).normalize();
    let right = forward.cross(&camera.up).normalize();
    let mut movement = Vec3::new(0.0, 0.0, 0.0);

    // Alternar el estado de `mouse_active` al hacer clic
    if window.get_mouse_down(minifb::MouseButton::Left) {
        if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if mouse_x >= 0.0 && mouse_x <= window.get_size().0 as f32
                && mouse_y >= 0.0 && mouse_y <= window.get_size().1 as f32
            {
                *mouse_active = !*mouse_active;
            }
        }
    }

    if *mouse_active {
        // Obtener la posición del mouse solo si está activo
        if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if mouse_x >= 0.0 && mouse_x <= window.get_size().0 as f32
                && mouse_y >= 0.0 && mouse_y <= window.get_size().1 as f32
            {
                if let Some((prev_x, prev_y)) = *prev_mouse_pos {
                    let delta_x = mouse_x - prev_x;
                    let delta_y = mouse_y - prev_y;

                    // Movimiento lateral según el desplazamiento horizontal del mouse
                    let forward = (camera.center - camera.eye).normalize();
                    let right = forward.cross(&camera.up).normalize();
                    let lateral_movement = right * (-delta_x) * movement_speed;

                    camera.move_center(lateral_movement); // Actualizar la posición de la cámara

                    // Rotación hacia arriba/abajo según el desplazamiento vertical del mouse
                    camera.orbit(0.0, delta_y * rotation_speed);
                }

                // Actualizar la posición previa del mouse
                *prev_mouse_pos = Some((mouse_x, mouse_y));
            } else {
                // Si el mouse está fuera de la ventana, desactivar
                *mouse_active = false;
                *prev_mouse_pos = None;
            }
        }
    } else {
        // Resetear posición previa si no está activo
        *prev_mouse_pos = None;
    }
    // Mover la cámara solo si no hay colisión
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    if window.is_key_down(Key::W) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::S) {
        camera.zoom(-zoom_speed);
    }
}
