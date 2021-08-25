use bevy::{
    prelude::*, 
    window::WindowMode
};
use bevy_rapier2d::prelude::*;
use nalgebra::Vector2;

mod player;
mod level;
mod particles;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Rusty Jam".to_string(),
            width: 1024.,
            height: 720.,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_startup_system(setup.system().label("physics"))
        .add_startup_system(player::setup_player.system().after("physics"))
        .add_startup_system(level::setup_environment.system().after("physics"))
        .add_system(player::player_movement_system.system())
        .add_system(player::player_shoot_system.system())
        .add_system(level::level_builder_system.system())
        .add_system(particles::particle_emission_system.system())
        .add_system(particles::burst_particle_emission_system.system())
        .add_system(particles::particle_update_system.system())
        .run();
}


fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // Spawn cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn()
        .insert(particles::ContinuousParticleEmitter {
            rate: 10.0,
            emit_fractional_build: 0.0,
        })
        .insert(particles::ParticleEmissionParams {
            speed_min: 1.0,
            speed_max: 10.0,
            particle_drag: 0.001,
            particle_size: Vec2::new(10.0, 10.0),
            lifetime_min: 1.0,
            lifetime_max: 5.0,
            material: materials.add(Color::rgb(1.0, 0.3, 1.0).into()),
        })
        .insert(Transform::from_xyz(50.0, 0.0, 0.0));

    // Configure Physics
    rapier_config.scale = 40.0;
    rapier_config.gravity = Vector2::zeros();
}
