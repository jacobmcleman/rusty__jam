use bevy::{
    prelude::*, 
    window::WindowMode
};

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
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_startup_system(setup.system())
        .add_system(player::player_movement_system.system())
        .run();
}


fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load sprites
    let circle_texture_handle: Handle<Texture> = asset_server.load("sprites/circle.png");

    // Spawn cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Create an object with a sprite to be the player
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(circle_texture_handle.into()),
            transform: Transform::from_xyz(0.0, -215.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(player::PlayerMovement {speed: 100.0});
}
