use ::rand::random_range;
use macroquad::prelude::*;

const GRAVITATIONAL_CONSTANT: f32 = 15.0;

const PARTICLE_COUNT: usize = 100;

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
        let rng = -1.0..1.0;
        Particle::new_with_velocity(
            x,
            y,
            random_range(rng.clone()),
            random_range(rng),
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
}

#[derive(Debug, Clone)]
struct State {
    particles: Vec<Particle>,
    current_time: f32,
}

fn init_state() -> State {
    let mut particles = Vec::with_capacity(PARTICLE_COUNT);
    for _ in 0..PARTICLE_COUNT {
        let x = random_range(0.0..1920.0);
        let y = random_range(0.0..1080.0);
        let mass = random_range(10.0..100.0);
        particles.push(Particle::new(x, y, mass));
    }
    State {
        particles,
        current_time: 0.0,
    }
}

fn calculate_acceleration(axial_displacement: f32, distance: f32, mass: f32) -> f32 {
    if axial_displacement == 0.0 {
        return 0.0; // Avoid division by zero
    }
    GRAVITATIONAL_CONSTANT * mass * axial_displacement / distance.powi(3)
}

fn calculate_distance(first: &Particle, second: &Particle) -> f32 {
    ((first.x - second.x).powi(2) + (first.y - second.y).powi(2)).sqrt()
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

fn update_state(mut state: State) -> State {
    let delta_time = get_frame_time();

    // Handle collisions and merge particles
    merge_colliding_particles(&mut state.particles);

    let mut accelerations = vec![[0.0_f32, 0.0_f32]; state.particles.len()];
    // Calculate gravitational forces between particles
    for i in 0..state.particles.len() {
        let a = &state.particles[i];
        let [x_acceleration, y_acceleration] =
            state.particles.iter().fold([0.0_f32, 0.0_f32], |acc, e| {
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

    //println!("Accelerations: {:?}", accelerations);
    // Update particle positions based on their velocities
    for i in 0..state.particles.len() {
        let particle = &mut state.particles[i];

        particle.x_velocity += accelerations[i][0] * delta_time;
        particle.y_velocity += accelerations[i][1] * delta_time;

        particle.x += particle.x_velocity * delta_time;
        particle.y += particle.y_velocity * delta_time;
    }

    state.current_time += delta_time;
    state
}

fn render_state(state: State) {
    clear_background(BLACK);

    for particle in state.particles {
        draw_circle(particle.x, particle.y, particle.radius, WHITE);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Gravity Simulation".to_owned(),
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = init_state();

    loop {
        render_state(state.clone());

        //println!("CurrentState: {:?}", state);
        state = update_state(state);
        //println!("PostState: {:?}", state);

        next_frame().await
    }
}
