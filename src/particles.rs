use bevy::prelude::*;
use rand::Rng;

pub struct Particle {
    velocity: Vec2,
    drag: f32,
    lifetime: f32
}

pub struct ContinuousParticleEmitter {
    pub rate: f32,
    pub emit_fractional_build: f32,
}

pub struct ParticleEmissionParams {
    pub speed_min: f32,
    pub speed_max: f32,
    pub particle_drag: f32,
    pub particle_size: Vec2,
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub material: Handle<ColorMaterial>
}

pub fn particle_emission_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut ContinuousParticleEmitter, &ParticleEmissionParams, &Transform)>
) {
    for (mut emitter, params, transform) in query.iter_mut() {
        let mut to_emit = emitter.rate * time.delta_seconds() + emitter.emit_fractional_build;
        let mut rng = rand::thread_rng();
        while to_emit > 1.0 {
            let angle = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
            let direction = Vec2::new(f32::sin(angle), f32::cos(angle));
            let emit_vel = direction * rng.gen_range(params.speed_min..params.speed_max);
            spawn_particle(
                &mut commands, 
                &params.material, 
                transform.translation, 
                emit_vel, 
                params.particle_drag, 
                params.particle_size,
                rng.gen_range(params.lifetime_min..params.lifetime_max)
            );
            to_emit -= 1.0;
        }
        emitter.emit_fractional_build = to_emit;
    } 
    
}

pub fn particle_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Particle, &mut Transform, Entity)>
) {
    for (mut part, mut transform, entity) in query.iter_mut() {
        part.lifetime -= time.delta_seconds();
        if part.lifetime < 0.0 {
            commands.entity(entity).despawn_recursive();
        }
        else {
            let translation = &mut transform.translation;
            translation.x += part.velocity.x * time.delta_seconds();
            translation.y += part.velocity.y * time.delta_seconds();

            part.velocity.x *= 1.0 - part.drag;
            part.velocity.y *= 1.0 - part.drag;
        }
    }
}

fn spawn_particle(commands: &mut Commands, material: &Handle<ColorMaterial>, position: Vec3, velocity: Vec2, drag: f32, size: Vec2, lifetime: f32) {
    commands.spawn_bundle(SpriteBundle {
        material: material.clone(), 
        transform: Transform::from_translation(position),
        sprite: Sprite::new(size), 
        ..Default::default()
    })
    .insert(Particle {velocity, drag, lifetime});
}