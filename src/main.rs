#![windows_subsystem = "windows"]
use bevy::{
    prelude::*, 
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
            title: "Smoke and Mirrors".to_string(),
            width: 1024.,
            height: 720.,
            vsync: true,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(gamestate::Score{value: 0, max: 0})
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
            SystemSet::on_enter(GameState::Playing).with_system(setup_playing.system()),
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
) {
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Smoke & Mirrors".to_string(),
                    style: TextStyle {
                                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                            font_size: 80.0,
                            color: Color::rgb(0.6, 0.6, 1.0)
                        },
                    },
                    TextSection {
                        value: "\nBuilt by Aevek in 1 week".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Roboto-Regular.ttf"),
                            font_size: 60.0,
                            color: Color::rgb(0.4, 0.4, 1.0)
                        },
                    },
                    TextSection {
                        value: "\n[Space] to start game\n[Esc] to quit".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Roboto-Regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.4, 0.4, 1.0)
                        },
                    },
                    TextSection {
                        value: "\n\nObjective:\nCollect as many of the dropped cards\nas you can without being caught.".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Roboto-Regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.4, 0.4, 1.0)
                        },
                    },
                    TextSection {
                        value: "\n\nControls:\n[WASD] to move\n[Space] to drop smoke bomb".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Roboto-Regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.4, 0.4, 1.0)
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
    });
}

struct DiagText;
struct Preserve;
fn screen_text(
    diagnostics: Res<Diagnostics>,
    score: Res<Score>,
    mut query: Query<&mut Text, With<DiagText>>,
    player_query: Query<&player::PlayerShooting>
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            if let Ok(player) = player_query.single() {
                let bombs_text = (0..player.bombs).map(|_| "O").collect::<String>() + &(player.bombs..3).map(|_| "-").collect::<String>();
                for mut text in query.iter_mut() {
                    text.sections[1].value = format!("{}/{}", score.value, score.max);
                    text.sections[3].value = bombs_text.clone();
                    text.sections[5].value = format!("{:.1}", average);
                }
            }
            
        }
    };
}


fn gameover_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    commands
        .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Game Over!".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                                font_size: 80.0,
                                color: Color::rgb(0.6, 0.6, 1.0)
                            },
                        },
                        TextSection {
                            value: format!("\nCards Found: {}/{}", score.value, score.max),
                            style: TextStyle {
                                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                                font_size: 60.0,
                                color: Color::rgb(0.4, 0.4, 1.0)
                            },
                        },
                        TextSection {
                            value: "\n[Space] to try again\n[Esc] to quit".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.4, 0.4, 1.0)
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
            });

        score.value = 0;
}

fn all_setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
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
}

fn setup_playing(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
) {
    
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Cards Collected: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 0.7, 0.1),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 0.7, 0.1),
                    },
                },
                TextSection {
                    value: "\nSmoke Bombs: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 0.7, 0.1),
                    },
                },
                TextSection {
                    value: "ooo".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 0.7, 0.1),
                    },
                },
                TextSection {
                    value: "\nAverage FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 10.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Roboto-Regular.ttf"),
                        font_size: 10.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
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
    .insert(DiagText);
}

fn teardown(mut commands: Commands, entities: Query<Entity, Without<Preserve>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}