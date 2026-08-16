use ::rand::random_range;
use macroquad::prelude::*;

const GRAVITATIONAL_CONSTANT: f32 = 15.0;

const PARTICLE_COUNT: usize = 500;

const SCREEN_WIDTH: usize = 1920;
const SCREEN_HEIGHT: usize = 1080;

const SCREEN_CENTRE_X: usize = SCREEN_WIDTH / 2;
const SCREEN_CENTRE_Y: usize = SCREEN_HEIGHT / 2;

// Sun will be 95% of the total mass of all particles
const SUN_MASS_RATIO: f32 = 0.50;

// Density of all particles is assumed to be 1.0 for simplicity
#[derive(Debug, Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    x_velocity: f32,
    y_velocity: f32,
    mass: f32,
    radius: f32,
}

impl Particle {
    fn new(x: f32, y: f32, mass: f32) -> Self {
        Particle::new_with_velocity(
            x,
            y,
            0.0,
            0.0,
            mass
        )
    }

    fn new_with_velocity(x: f32, y: f32, x_velocity: f32, y_velocity: f32, mass: f32) -> Self {
        let radius = (mass / std::f32::consts::PI).sqrt(); // Assuming density = 1.0
        Particle {
            x,
            y,
            x_velocity,
            y_velocity,
            mass,
            radius,
        }
    }

    fn position_vector(self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
    }

    fn velocity_vector(self) -> Vec2 {
        Vec2 { x: self.x_velocity, y: self.y_velocity }
    }
}

#[derive(Debug, Clone)]
struct State {
    particles: Vec<Particle>,
    current_time: f32,
    last_delta_time: f32,
    total_mass: f32,
}

fn calculate_initial_velocity(sun_position: Vec2, sun_mass: f32, position: Vec2) -> Vec2 {
    let delta = position - sun_position;
    let distance = delta.length();
    let tangential_speed = (GRAVITATIONAL_CONSTANT * sun_mass / distance).sqrt();

    // Perpendicular to the radius vector, scaled to the orbital speed
    Vec2 { x: -delta.y, y: delta.x } / distance * tangential_speed
}

fn init_state() -> State {
    let mut particles = Vec::with_capacity(PARTICLE_COUNT);
    let mut particles_no_velocity = Vec::with_capacity(PARTICLE_COUNT);

    let mut centre_of_mass = Vec2 { x: 0.0, y: 0.0 };
    let mut total_mass = 0.0_f32;
    let mut other_mass = 0.0_f32;

    let sun_position = Vec2 { x: SCREEN_CENTRE_X as f32, y: SCREEN_CENTRE_Y as f32 };

    for _ in 1..PARTICLE_COUNT {
        let position = Vec2 { x: random_range(0.0..SCREEN_WIDTH as f32), y: random_range(0.0..SCREEN_HEIGHT as f32) };
        let mass = random_range(10.0..100.0);

        other_mass += mass;
        particles_no_velocity.push((position, mass));
    }

    // Derive the sun's mass from the other particles so it actually makes up SUN_MASS_RATIO of the total
    let sun_mass = other_mass * SUN_MASS_RATIO / (1.0 - SUN_MASS_RATIO);
    particles_no_velocity.insert(0, (sun_position, sun_mass));

    for (position, mass) in &particles_no_velocity {
        total_mass += *mass;
        centre_of_mass += *position * *mass;
    }

    centre_of_mass /= total_mass;

    particles.push(Particle::new(particles_no_velocity[0].0.x, particles_no_velocity[0].0.y, particles_no_velocity[0].1));

    for particle in particles_no_velocity.iter().skip(1) {
        let velocity = calculate_initial_velocity(sun_position, sun_mass, particle.0);
        particles.push(Particle::new_with_velocity(particle.0.x, particle.0.y, velocity.x, velocity.y, particle.1));
    }

    State {
        particles,
        current_time: 0.0,
        last_delta_time: 0.0,
        total_mass,
    }
}

fn calculate_acceleration(axial_displacement: f32, distance: f32, mass: f32) -> f32 {
    if axial_displacement == 0.0 {
        return 0.0; // Avoid division by zero
    }
    GRAVITATIONAL_CONSTANT * mass * axial_displacement / distance.powi(3)
}

fn calculate_distance(first: &Particle, second: &Particle) -> f32 {
    first.position_vector().distance(second.position_vector())
}

fn find_collision(particles: &[Particle], first_index: usize) -> Option<usize> {
    let first = &particles[first_index];

    for second_index in (first_index + 1)..particles.len() {
        let second = &particles[second_index];

        let collision_distance = first.radius + second.radius;
        if calculate_distance(first, second).abs() < collision_distance.abs() {
            return Some(second_index);
        }
    }
    None
}

fn merge_pair(particles: &mut Vec<Particle>, first_index: usize, second_index: usize) {
    let first = particles[first_index];
    let second = particles[second_index];

    let resulting_mass = first.mass + second.mass;

    let x_position = (first.x * first.mass + second.x * second.mass) / resulting_mass;
    let y_position = (first.y * first.mass + second.y * second.mass) / resulting_mass;

    let x_velocity =
        (first.x_velocity * first.mass + second.x_velocity * second.mass) / resulting_mass;
    let y_velocity =
        (first.y_velocity * first.mass + second.y_velocity * second.mass) / resulting_mass;

    particles[first_index] = Particle::new_with_velocity(x_position, y_position, x_velocity, y_velocity, resulting_mass);

    particles.swap_remove(second_index);
}

fn merge_colliding_particles(particles: &mut Vec<Particle>) {
    let mut first_index = 0;

    while first_index < particles.len() {
        // Restart after every merge because the resulting object has a new
        // position, radius, velocity and mass.
        while let Some(second_index) = find_collision(particles, first_index) {
            merge_pair(particles, first_index, second_index);
        }

        first_index += 1;
    }
}

fn update_positions(particles: &mut Vec<Particle>, delta_time: f32) {
    let mut accelerations = vec![[0.0_f32, 0.0_f32]; particles.len()];

    for i in 0..particles.len() {
        let a = &particles[i];
        let [x_acceleration, y_acceleration] =
            particles.iter().fold([0.0_f32, 0.0_f32], |acc, e| {
                let x_displacement = e.x - a.x;
                let y_displacement = e.y - a.y;
                let distance = (x_displacement.powi(2) + y_displacement.powi(2)).sqrt();
                [
                    acc[0] + calculate_acceleration(x_displacement, distance, e.mass),
                    acc[1] + calculate_acceleration(y_displacement, distance, e.mass),
                ]
            });
        accelerations[i] = [x_acceleration, y_acceleration];
    }

    // Update particle positions based on their velocities
    for i in 0..particles.len() {
        let particle = &mut particles[i];

        particle.x_velocity += accelerations[i][0] * delta_time;
        particle.y_velocity += accelerations[i][1] * delta_time;

        particle.x += particle.x_velocity * delta_time;
        particle.y += particle.y_velocity * delta_time;
    }
}

fn update_state(state: &mut State) {
    let delta_time = if STEP_MODE { 0.016_f32 } else { get_frame_time() };

    // Handle collisions and merge particles
    merge_colliding_particles(&mut state.particles);
    update_positions(&mut state.particles, delta_time);

    state.current_time += delta_time;
    state.last_delta_time = delta_time;
}

fn draw_info_panel(state: &State) {
    let mut parts = vec![];
    parts.push(format!("Particle Count: {}", state.particles.len()));
    parts.push(format!("Total Mass: {:.2}", state.total_mass));
    //parts.push(format!("Current Time: {}", state.current_time));
    parts.push(format!("Current FPS: {}", (1.0 / state.last_delta_time).round()));
    let text = parts.join(", ");
    draw_text(&text, 10.0, 20.0, 30.0, WHITE);
}

fn render_state(state: &State) {
    clear_background(BLACK);

    let sun_particle = &state.particles[0];

    // Keep the sun centred on screen regardless of window size
    let camera = Camera2D::from_display_rect(Rect::new(
        sun_particle.x - screen_width() / 2.0,
        sun_particle.y - screen_height() / 2.0,
        screen_width(),
        screen_height(),
    ));
    set_camera(&camera);

    let mut centre_of_mass = Vec2 { x: 0.0, y: 0.0 };

    for particle in &state.particles {
        draw_circle(particle.x, particle.y, particle.radius, WHITE);
        centre_of_mass += particle.position_vector() * particle.mass;
    }

    centre_of_mass /= state.total_mass;

    draw_circle(centre_of_mass.x, centre_of_mass.y, 5.0, RED);

    // Draw UI in screen space, unaffected by the world camera
    set_default_camera();
    draw_info_panel(state);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Gravity Simulation".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        high_dpi: true,
        ..Default::default()
    }
}

const STEP_MODE: bool = false;

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = init_state();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::R) {
            state = init_state();
        }

        if is_key_pressed(KeyCode::Space) || !STEP_MODE {
            update_state(&mut state);
        }

        render_state(&state);

        next_frame().await
    }
}
