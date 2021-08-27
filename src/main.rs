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

pub struct MainCam;

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
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(level::LevelPlugin)
        .add_plugin(ai::AiPlugin)
        .add_plugin(lighting::LightingPlugin)
        .add_plugin(particles::ParticlePlugin)
        .add_startup_system(setup.system().label("physics"))
        // REGION OF DEBUG STARTUP SYSTEMS THAT ARE SETTING UP THE GAME STUFF THAT NEEDS TO HAPPEN IN NOT STARTUP
        .add_startup_system(player::setup_player.system().after("physics").after("graphics_init"))
        .add_startup_system(ai::setup_test_ai_perception.system().after("physics").after("graphics_init"))
        // END
        .run();
}


fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // Spawn cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCam );
    commands.spawn_bundle(UiCameraBundle::default());

    // Configure Physics
    rapier_config.scale = 40.0;
    rapier_config.gravity = Vector2::zeros();
}
