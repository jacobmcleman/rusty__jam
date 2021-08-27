use bevy::{
    prelude::*, 
    render::camera::Camera, 
    window::WindowMode,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
};
use bevy_rapier2d::prelude::*;
use nalgebra::Vector2;

mod player;
mod level;
mod particles;
mod ai;
mod lighting;
mod gamestate;
mod pickup;

use gamestate::{GameState, Score};

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
        .insert_resource(gamestate::Score{value: 0})
        .insert_resource(gamestate::CurrentLevel{name: "game".to_string()})
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Startup)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(level::LevelPlugin)
        .add_plugin(ai::AiPlugin)
        .add_plugin(lighting::LightingPlugin)
        .add_plugin(particles::ParticlePlugin)
        .add_startup_system(all_setup.system().label("physics"))
        .add_system_set(SystemSet::on_enter(GameState::Playing)
            .with_system(level::setup_environment.system())
        )
        .add_system_set(SystemSet::on_exit(GameState::Startup).with_system(teardown.system()))
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown.system()))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_enter(GameState::Startup).with_system(startup_setup.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Startup).with_system(gamestate::startgame_keyboard.system()),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(gameover_setup.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(gamestate::startgame_keyboard.system()),
        )
        .add_system(screen_text.system())
        // END
        .run();
}

fn startup_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    format!("Press [Space] to Start Game\n[Esc] to quit"),
                    TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 80.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });

        
}

struct DiagText;
struct Preserve;
fn screen_text(
    diagnostics: Res<Diagnostics>,
    score: Res<Score>,
    mut query: Query<&mut Text, With<DiagText>>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[1].value = format!("{}", score.value);
                text.sections[3].value = format!("{:.2}", average);
            }
        }
    };
}


fn gameover_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score: ResMut<Score>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    format!("Game Over!\nScore: {}\n[Space] to try again\n[Esc] to quit", score.value),
                    TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 80.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });

        score.value = 0;
}

fn all_setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    asset_server: Res<AssetServer>,
) {
    // Spawn cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCam )
        .insert(Preserve);
    commands.spawn_bundle(UiCameraBundle::default())
        .insert(Preserve);

    // Configure Physics
    rapier_config.scale = 40.0;
    rapier_config.gravity = Vector2::zeros();

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Collected Count: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 1.0),
                    },
                },
                TextSection {
                    value: "\nAverage FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 1.0),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(DiagText)
    .insert(Preserve);
}

fn teardown(mut commands: Commands, entities: Query<Entity, Without<Preserve>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}