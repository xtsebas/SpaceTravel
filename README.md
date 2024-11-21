# 🌌 **Solar System Simulation**

Una simulación interactiva del sistema solar que permite enfocar planetas, explorar desde diferentes perspectivas y observar los anillos de Saturno con precisión. 

## 📋 **Requisitos previos**

Asegúrate de tener instalado lo siguiente en tu máquina:

- **Rust**: Puedes instalar Rust desde [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Cargo**: Se incluye con la instalación de Rust.
- **Git**: Para clonar este repositorio.

---

## 🚀 **Instalación**

1. **Clona el repositorio**:
   ```bash
   git clone https://github.com/tu-usuario/solar-system-simulation.git
   cd solar-system-simulation
   ```
2. **Instala las dependencias**:
   Este proyecto usa las siguientes librerías de Rust. Cargo gestionará automáticamente su instalación:
   nalgebra
    - minifb
    - image
    - fastnoise-lite
   ```bash
   cargo build
   ```

## 🏃‍♂️ **Ejecutar el poyecto**
1. Para compilar y ejecutar la simulación, ejecuta:
   ```bash
   cargo run
   ```
2. La ventana de la simulación se abrirá y mostrará el sistema solar en 3D.

## 🎮 **Controles**
Mouse:
  - Movimiento horizontal: Mueve lateralmente la cámara.
  - Movimiento vertical: Ajusta la inclinación de la cámara.
  
Teclado:
  - W/S: Acercar/alejar la cámara (Zoom).
  - A/D: Rotar la cámara alrededor del punto de enfoque.
  - Teclas de planetas:
  - M: Mercurio.
  - V: Venus.
  - E: Tierra.
  - R: Marte.
  - J: Júpiter.
  - N: Saturno.
  - U: Urano.
  - B: Alternar entre la vista normal y "Bird's Eye View".
  - ESC: Salir del programa.

## 🌟 **Características destacadas**
- Transiciones suaves: La cámara se mueve fluidamente al enfocar planetas o regresar a la vista general.
- Anillos de Saturno: Los anillos están perfectamente centrados y proporcionados en relación con el planeta.
- Vista "Bird's Eye": Cambia a una vista superior para observar todo el sistema solar.

## 📽️ **Video de prueba**
[final grafica.webm](https://github.com/user-attachments/assets/f3a63b9f-73d4-4c68-b246-c13b07a70997)

   
