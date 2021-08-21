use bevy::{
    prelude::*, 
    window::WindowMode
};
use bevy_rapier2d::prelude::*;
use nalgebra::Vector2;

mod player;

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
        .add_startup_system(setup.system())
        .add_startup_system(player::setup_player.system())
        .add_system(player::player_movement_system.system())
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
    rapier_config.scale = 100.0;
    rapier_config.gravity = Vector2::zeros();
}