use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use nalgebra_glm::Vec2;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use crate::light::Light;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use fastnoise_lite::FastNoiseLite;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Transformación de posición base
  let position = Vec4::new(
      vertex.position.x,
      vertex.position.y,
      vertex.position.z,
      1.0,
  );

  // Zoom para el relieve
  let zoom = 5.0;
  let displacement_amount = uniforms.noise.get_noise_3d(
      vertex.position.x * zoom,
      vertex.position.y * zoom,
      vertex.position.z * zoom,
  );

  // Desplazamiento a lo largo de la normal del vértice
  let displaced_position = vertex.position + vertex.normal * displacement_amount * 0.5;

  // Transformación del vértice desplazado
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * Vec4::new(
      displaced_position.x,
      displaced_position.y,
      displaced_position.z,
      1.0,
  );

  // División en perspectiva
  let w = transformed.w;
  let ndc_position = Vec4::new(
      transformed.x / w,
      transformed.y / w,
      transformed.z / w,
      1.0,
  );

  // Aplicar la matriz de viewport
  let screen_position = uniforms.viewport_matrix * ndc_position;

  // Transformar la normal
  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
  let transformed_normal = normal_matrix * vertex.normal;

  // Crear un nuevo vértice con atributos transformados
  Vertex {
      position: vertex.position,
      normal: vertex.normal,
      tex_coords: vertex.tex_coords,
      color: vertex.color,
      transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
      transformed_normal,
  }
}


pub fn select_shader(index: usize, fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let sun_position = Vec3::new(0.0, 0.0, 0.0);
    let sun_light = Light::new(
        sun_position,                // Posición del Sol
        Color::new(255, 255, 200),   // Color amarillo claro
        3.0,                         // Intensidad de la luz
    );

    match index {
        0 => sun_shader().0,                           // El Sol
        1 => apply_lighting(mercury_shader(fragment, uniforms), fragment, &sun_light),
        2 => apply_lighting(venus_shader(fragment, uniforms), fragment, &sun_light),
        3 => apply_lighting(earth_shader(fragment, uniforms), fragment, &sun_light),
        4 => apply_lighting(mars_shader(fragment, uniforms).0, fragment, &sun_light),
        5 => apply_lighting(jupiter_shader(fragment, uniforms), fragment, &sun_light),
        6 => apply_lighting(saturn_shader(fragment, uniforms), fragment, &sun_light),
        7 => apply_lighting(uranus_shader(fragment, uniforms), fragment, &sun_light),
        8 => ring_shader(fragment).0,                 // Anillos de Saturno (sin iluminación)
        _ => sun_shader().0,                          // Por defecto: el Sol
    }
}

fn apply_lighting(base_color: Color, fragment: &Fragment, light: &Light) -> Color {
    // Vector desde el fragmento hasta la fuente de luz
    let light_direction = (light.position - fragment.vertex_position).normalize();

    // Producto punto para determinar la intensidad de la luz en este fragmento
    let intensity = fragment.normal.dot(&light_direction).max(0.0);

    // Atenuación de la luz según la distancia
    let distance = (light.position - fragment.vertex_position).magnitude();
    let attenuation = 1.0 / (1.0 + 0.1 * distance + 0.01 * distance * distance);

    // Color final con iluminación aplicada
    let light_effect = light.color * (intensity * light.intensity * attenuation);
    base_color.lerp(&light_effect, intensity as f32)
}

fn ring_shader(fragment: &Fragment) -> (Color, u32) {
    // Coordenadas en 2D para determinar la distancia desde el centro de los anillos
    let position = Vec2::new(fragment.vertex_position.x as f32, fragment.vertex_position.z as f32); // Usar X y Z para planos
    let distance_from_center = position.magnitude(); // Calcular la distancia desde el centro

    // Definir el número de bandas y su ancho
    let num_bands = 4; // Número total de bandas en los anillos
    let max_distance = 1.0_f32; // Distancia máxima para las bandas (ajustar según el tamaño de los anillos)
    let band_width = max_distance / num_bands as f32; // Ancho de cada banda

    // Calcular en qué banda está el fragmento actual
    let band_index = (distance_from_center / band_width).floor() as i32;

    // Variar el color de los anillos en función de su índice
    let band_colors = [
        Color::from_hex(0xB0C4DE), // Azul claro
        Color::from_hex(0x708090), // Gris pizarra
        Color::from_hex(0xA9A9A9), // Gris claro
        Color::from_hex(0xF5F5DC), // Beige
    ];

    // Seleccionar el color basado en el índice de la banda y el número de bandas
    let color = band_colors[(band_index.abs() % num_bands) as usize % band_colors.len()];

    // Aplicar un efecto de difuminado en los bordes de las bandas
    let edge_distance = (distance_from_center % band_width) / band_width;
    let smooth_edge = (1.0_f32 - edge_distance).clamp(0.0_f32, 1.0_f32);

    // Modificar la opacidad para dar un efecto de transparencia a los anillos
    let final_color = color * smooth_edge;

    (final_color, 0)
}


fn sun_shader() -> (Color, u32) {
    let base_color = Color::from_float(1.0, 0.9, 0.5); // Color amarillo/dorado para el Sol
    let emission = 100; // Máxima emisión para el efecto de glow/bloom
  
    (base_color, emission)
}



fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores para diferentes biomas
  let land_color = Color::new(34, 139, 34);       // Verde para continentes
  let ocean_color = Color::new(30, 144, 255);     // Azul para océanos
  let snow_color = Color::new(255, 250, 250);     // Blanco para zonas polares
  let cloud_color = Color::new(255, 255, 255);    // Blanco para las nubes

  // Zoom para el ruido que genera los biomas
  let zoom = 15.0;
  let noise_value = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
      fragment.vertex_position.z * zoom,
  );

  // Capa base para la superficie terrestre
  let base_color = if noise_value < -0.3 {
      ocean_color.lerp(&Color::new(25, 105, 210), (noise_value + 0.3) / 0.3)
  } else if noise_value > 0.7 {
      land_color.lerp(&snow_color, (noise_value - 0.7) / 0.3)
  } else {
      ocean_color.lerp(&land_color, (noise_value + 0.3) / 1.0)
  };

  // Primera capa de nubes en movimiento
  let cloud_zoom1 = 10.0;
  let displacement_x1 = uniforms.noise.get_noise_2d(fragment.vertex_position.x * cloud_zoom1, fragment.vertex_position.y * cloud_zoom1) * 0.3;
  let displacement_z1 = uniforms.noise.get_noise_2d(fragment.vertex_position.z * cloud_zoom1, fragment.vertex_position.y * cloud_zoom1) * 0.3;
  let cloud_noise_value1 = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * cloud_zoom1 + displacement_x1,
      fragment.vertex_position.y * cloud_zoom1,
      fragment.vertex_position.z * cloud_zoom1 + displacement_z1,
  );

  // Opacidad de la primera capa de nubes
  let cloud_opacity1 = (cloud_noise_value1 * 0.5 + 0.5).min(1.0).max(0.0);

  // Segunda capa de nubes en movimiento (opcional, para mayor complejidad)
  let cloud_zoom2 = 8.0;
  let displacement_x2 = uniforms.noise.get_noise_2d(fragment.vertex_position.x * cloud_zoom2, fragment.vertex_position.y * cloud_zoom2) * 0.4;
  let displacement_z2 = uniforms.noise.get_noise_2d(fragment.vertex_position.z * cloud_zoom2, fragment.vertex_position.y * cloud_zoom2) * 0.4;
  let cloud_noise_value2 = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * cloud_zoom2 + displacement_x2,
      fragment.vertex_position.y * cloud_zoom2,
      fragment.vertex_position.z * cloud_zoom2 + displacement_z2,
  );

  // Opacidad de la segunda capa de nubes
  let cloud_opacity2 = (cloud_noise_value2 * 0.5 + 0.5).min(1.0).max(0.0);

  // Combinación de las capas de nubes con la superficie
  let combined_clouds = cloud_color * cloud_opacity1 + cloud_color * cloud_opacity2;
  let final_color = base_color.lerp(&combined_clouds, 0.5); // Ajusta la opacidad general de las nubes

  final_color
}


fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, u32) {
    let noise_value = uniforms.noise.get_noise_2d(fragment.vertex_position.x, fragment.vertex_position.y);
    
    let dark_red = Color::from_float(0.4, 0.1, 0.1); // Color oscuro para áreas en sombra
    let bright_orange = Color::from_float(0.8, 0.4, 0.1); // Color brillante para áreas iluminadas
    let terracotta = Color::from_float(0.6, 0.3, 0.1); // Color intermedio, típico de Marte
  
    // Usar lerp para mezclar colores basado en el valor del ruido
    let lerp_factor = noise_value.clamp(0.0, 1.0); // Asegurar que esté entre 0 y 1
    let base_color = if lerp_factor < 0.5 {
      dark_red.lerp(&terracotta, lerp_factor * 2.0) // Interpola entre rojo oscuro y terracotta
    } else {
      terracotta.lerp(&bright_orange, (lerp_factor - 0.5) * 2.0) // Interpola entre terracotta y naranja brillante
    };
  
    // Definir la posición y dirección de la luz
    let light_pos = Vec3::new(0.0, 8.0, 9.0);  // Posición de la fuente de luz
    let light_dir = (light_pos - fragment.vertex_position).normalize(); // Dirección de la luz desde la posición del fragmento
  
    // Normalizar la normal del fragmento
    let normal = fragment.normal.normalize();
  
    // Calcular la intensidad de la luz difusa
    let diffuse_intensity = normal.dot(&light_dir).max(0.0);
  
    // Modificar el color final basado en la intensidad de la luz
    let lit_color = base_color * diffuse_intensity;  // Modula el color por la intensidad de la luz
  
    // Añadir un término ambiental para evitar que las partes no iluminadas sean completamente oscuras
    let ambient_intensity = 0.15;  // Intensidad de luz ambiental, ajusta según necesites
    let ambient_color = base_color * ambient_intensity;
  
    // Suma del componente ambiental y difuso
    let combined_color = ambient_color + lit_color;
  
    (combined_color, 0)
}  

fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Valores de ruido para las bandas y la superficie gaseosa
    let noise_value = uniforms.noise.get_noise_2d(fragment.vertex_position.x, fragment.vertex_position.y);

    // Colores pastel para las bandas gaseosas de Júpiter
    let pastel_pink = Color::from_float(1.0, 0.71, 0.76);  // Rosa pastel
    let soft_lilac = Color::from_float(0.87, 0.63, 0.87);  // Lila suave
    let white = Color::from_float(1.0, 1.0, 1.0);          // Blanco

    // Usar `lerp` para mezclar colores basado en el valor del ruido
    let lerp_factor = noise_value.clamp(0.0, 1.0); // Asegurar que esté entre 0 y 1
    let base_color = if lerp_factor < 0.5 {
        pastel_pink.lerp(&soft_lilac, lerp_factor * 2.0) // Interpola entre rosa pastel y lila suave
    } else {
        soft_lilac.lerp(&white, (lerp_factor - 0.5) * 2.0) // Interpola entre lila suave y blanco
    };

    // Definir la posición y dirección de la luz (el Sol)
    let light_pos = Vec3::new(-10.0, 8.0, 9.0); // Posición del Sol
    let light_dir = (light_pos - fragment.vertex_position).normalize(); // Dirección de la luz desde la posición del fragmento

    // Normalizar la normal del fragmento
    let normal = fragment.normal.normalize();

    // Calcular la intensidad de la luz difusa
    let diffuse_intensity = normal.dot(&light_dir).max(0.0);

    // Modificar el color final basado en la intensidad de la luz
    let lit_color = base_color * diffuse_intensity; // Modula el color por la intensidad de la luz

    // Añadir un término ambiental para evitar que las partes no iluminadas sean completamente oscuras
    let ambient_intensity = 0.15; // Intensidad de luz ambiental
    let ambient_color = base_color * ambient_intensity;

    // Suma del componente ambiental y difuso
    let combined_color = ambient_color + lit_color;

    combined_color
}


fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores base para las bandas gaseosas de Saturno
    let warm_yellow = Color::new(255, 225, 180);  // Amarillo cálido
    let soft_orange = Color::new(255, 200, 150);  // Naranja suave
    let light_beige = Color::new(240, 230, 210);  // Beige claro

    // Configuración del ruido para simular variaciones en la superficie
    let zoom = 10.0;
    let noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * zoom,
        fragment.vertex_position.y * zoom,
    );

    // Mezclar colores basado en el ruido
    let lerp_factor = noise_value.clamp(0.0, 1.0);
    let base_color = if lerp_factor < 0.5 {
        warm_yellow.lerp(&soft_orange, lerp_factor * 2.0)
    } else {
        soft_orange.lerp(&light_beige, (lerp_factor - 0.5) * 2.0)
    };

    // Normalizar la normal del fragmento
    let normal = fragment.normal.normalize();

    // Modificar el color final basado en la intensidad y atenuación de la luz
    let lit_color = base_color;

    // Añadir un término ambiental para evitar que las partes no iluminadas sean completamente oscuras
    let ambient_intensity = 0.2; // Intensidad de luz ambiental
    let ambient_color = base_color * ambient_intensity;

    // Combinar el color ambiental y difuso
    let planet_color = ambient_color + lit_color;

    planet_color
}

fn mercury_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para la superficie de Mercurio
    let base_color = Color::new(169, 169, 169);  // Gris claro
    let crater_color = Color::new(105, 105, 105);  // Gris oscuro para los cráteres
    let highlight_color = Color::new(200, 200, 200); // Gris claro brillante para áreas iluminadas

    // Configuración del ruido para los cráteres
    let zoom = 20.0;
    let noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * zoom,
        fragment.vertex_position.y * zoom,
    );

    // Decidir el color del fragmento basándose en el ruido
    let base_fragment_color = if noise_value < -0.2 {
        crater_color.lerp(&base_color, (noise_value + 0.2).clamp(0.0, 1.0))
    } else {
        base_color.lerp(&highlight_color, noise_value.clamp(0.0, 1.0))
    };

    // Calcular la dirección de la luz desde el fragmento hacia el Sol
    let sun_position = Vec3::new(0.0, 0.0, 0.0); // Posición del Sol en el centro
    let light_dir = (sun_position - fragment.vertex_position).normalize();

    // Normalizar la normal del fragmento
    let normal = fragment.normal.normalize();

    // Calcular la intensidad de la luz difusa
    let diffuse_intensity = normal.dot(&light_dir).max(0.0);

    // Calcular la atenuación de la luz basada en la distancia al Sol
    let distance = (sun_position - fragment.vertex_position).magnitude();
    let attenuation = 1.0 / (1.0 + 0.1 * distance + 0.02 * distance * distance);

    // Modificar el color final basado en la intensidad y atenuación de la luz
    let lit_color = base_fragment_color * diffuse_intensity * attenuation;

    // Añadir un término ambiental para evitar que las partes no iluminadas sean completamente oscuras
    let ambient_intensity = 0.15; // Intensidad de luz ambiental
    let ambient_color = base_fragment_color * ambient_intensity;

    // Suma del componente ambiental y difuso
    let combined_color = ambient_color + lit_color;

    combined_color
}


fn venus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(218, 165, 32);     // Color cálido para la superficie
  let cloud_color = Color::new(255, 228, 181);   // Color crema para las nubes

  let zoom = 8.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );

  base_color.lerp(&cloud_color, noise_value.abs())
}

fn uranus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para las capas gaseosas de Urano
    let light_blue = Color::new(173, 216, 230);   // Azul claro
    let cyan = Color::new(0, 255, 255);          // Cian brillante
    let white = Color::new(240, 248, 255);       // Blanco hielo

    // Configuración del ruido para las capas de gas
    let zoom = 8.0;
    let noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * zoom + uniforms.time as f32 * 0.1, // Añade tiempo para simular movimiento
        fragment.vertex_position.y * zoom,
    );

    // Patrón de bandas gaseosas basado en el ruido
    let lerp_factor = noise_value.clamp(0.0, 1.0); // Asegurar que esté en rango [0, 1]
    let base_color = if lerp_factor < 0.5 {
        light_blue.lerp(&cyan, lerp_factor * 2.0) // Interpola entre azul claro y cian
    } else {
        cyan.lerp(&white, (lerp_factor - 0.5) * 2.0) // Interpola entre cian y blanco hielo
    };

    // Definir la posición del Sol (el centro del sistema solar)
    let sun_position = Vec3::new(-20.0, 8.0, 9.0);

    // Calcular la dirección de la luz desde el fragmento hacia el Sol
    let light_dir = (sun_position - fragment.vertex_position).normalize();

    // Normalizar la normal del fragmento
    let normal = fragment.normal.normalize();

    // Calcular la intensidad de la luz difusa
    let diffuse_intensity = normal.dot(&light_dir).max(0.0);

    // Calcular la atenuación de la luz basada en la distancia al Sol
    let distance = (sun_position - fragment.vertex_position).magnitude();
    let attenuation = 1.0 / (1.0 + 0.1 * distance + 0.02 * distance * distance);

    // Modificar el color final basado en la intensidad y atenuación de la luz
    let lit_color = base_color * diffuse_intensity * attenuation;

    // Añadir un término ambiental para evitar que las partes no iluminadas sean completamente oscuras
    let ambient_intensity = 0.2; // Intensidad de luz ambiental (ligeramente mayor para un planeta gaseoso)
    let ambient_color = base_color * ambient_intensity;

    // Combinar el color ambiental y difuso
    let combined_color = ambient_color + lit_color;

    combined_color
}