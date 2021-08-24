use bevy::{
    prelude::*, 
    window::WindowMode
};
use bevy_rapier2d::prelude::*;
use nalgebra::Vector2;

mod player;
mod level;
mod particles;
mod ai;
mod lighting;

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
        .insert_resource(lighting::LightRenderData {
            pipeline_handle: None,
            base_mesh: None
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_startup_system(setup.system().label("physics"))
        .add_startup_system(player::setup_player.system().after("physics").after("graphics_init"))
        .add_startup_system(level::setup_environment.system().after("physics"))
        .add_startup_system(ai::setup_test_ai_perception.system().after("physics").after("graphics_init"))
        .add_system(player::player_movement_system.system())
        .add_system(player::player_shoot_system.system())
        .add_system(level::level_builder_system.system())
        .add_system(particles::particle_emission_system.system())
        .add_system(particles::burst_particle_emission_system.system())
        .add_system(particles::particle_update_system.system())
        .add_system(ai::ai_perception_system.system())
        .add_system(ai::ai_movement_system.system())
        .add_system(ai::ai_chase_behavior_system.system())
        .add_system(ai::ai_perception_debug_system.system())
        .add_startup_system(lighting::light_setup_system.system().label("graphics_init"))
        .add_system(lighting::point_light_mesh_builder.system())
        .add_system(lighting::spotlight_mesh_builder.system())
        .add_system(lighting::test_spin_system.system())
        .run();
}


fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // Spawn cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Configure Physics
    rapier_config.scale = 40.0;
    rapier_config.gravity = Vector2::zeros();
}
