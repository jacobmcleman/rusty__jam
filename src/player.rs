use bevy::{math::Vec3Swizzles, prelude::*, };
use bevy_rapier2d::prelude::*;
use nalgebra::{Vector2, vector};

use crate::particles;
use crate::gamestate::{GameState, Score};
use crate::pickup::Pickup;

pub struct PlayerMovement {
    pub speed: f32,
}

pub struct PlayerShooting {
    smoke_mat: Handle<ColorMaterial>,
}

pub struct CamFollow {
    pub position: Vec2,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.add_system_set(SystemSet::on_update(GameState::Playing)
            .with_system(player_movement_system.system())
            .with_system(player_shoot_system.system())
            .with_system(follow_camera_objstep.system())
            .with_system(follow_camera_camstep.system())
            .with_system(process_collision_events.system())
        );
    }
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_parameters: Res<RapierConfiguration>,
    mut query: Query<(&PlayerMovement, &mut RigidBodyVelocity)>
) {
    if let Ok((player, mut rb_vels)) = query.single_mut() {
        let mut y_movement = 0.0;
        let mut x_movement = 0.0; 
        if keyboard_input.pressed(KeyCode::W) {
            y_movement += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            y_movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            x_movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            x_movement += 1.0;
        }

        let mut movement = vector![x_movement, y_movement];
        if movement != Vector2::zeros() 
        {
            movement = movement.normalize() * (1.0 / rapier_parameters.scale) * player.speed;
        }

        rb_vels.linvel = movement;

        //println!("Moving ({}, {}) based on input ({}, {}", movement.x, movement.y, x_movement, y_movement);
    }
}

pub fn player_shoot_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    rapier_config: Res<RapierConfiguration>,
    query: Query<(&PlayerShooting, &Transform)>
) {
    if let Ok((player, transform)) = query.single() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("player position: {}", transform.translation);

            let block_size = 100.0;

            commands.spawn()
                .insert(particles::BurstParticleEmitter {
                    quantity: 100,
                    existence_time: 0.0,
                })
                .insert(particles::ParticleEmissionParams {
                    speed_min: 20.0,
                    speed_max: 250.0,
                    particle_drag: 4.0,
                    particle_size: Vec2::new(30.0, 30.0),
                    lifetime_min: 8.0,
                    lifetime_max: 15.0,
                    material: player.smoke_mat.clone(),
                })
                .insert(Transform::from_translation(transform.translation))
                .insert(crate::lighting::DynamicLightBlocker{size: block_size})
                .insert_bundle(ColliderBundle {
                    position: [transform.translation.x / rapier_config.scale, transform.translation.y / rapier_config.scale].into(),
                    shape: ColliderShape::ball(block_size * 0.5 / rapier_config.scale),
                    collider_type: ColliderType::Sensor,
                    ..Default::default()
                })
                ;
            };
    }
}

pub fn follow_camera_camstep(
    follow_query: Query<&CamFollow>,
    mut camera_query: Query<&mut Transform, With<crate::MainCam>>,
) {
    if let Ok(follow) = follow_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation.x = follow.position.x;
            camera_transform.translation.y = follow.position.y;
        }
    }
}

pub fn follow_camera_objstep(
    mut follow_query: Query<(&mut CamFollow, &Transform)>,
) {
    if let Ok((mut follow, transform)) = follow_query.single_mut() {
        follow.position = transform.translation.xy();
    }
}

pub fn spawn_player(
    position: Vec2,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rapier_config: &Res<RapierConfiguration>,
    asset_server: &Res<AssetServer>,
) {
    // Load sprite
    let circle_texture_handle: Handle<Texture> = asset_server.load("sprites/circle.png");
    let smoke_texture_handle: Handle<Texture> = asset_server.load("sprites/smoke.png");

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    let collider_size = sprite_size_x / rapier_config.scale;

    commands
    .spawn()
    .insert_bundle(SpriteBundle {
        material: materials.add(circle_texture_handle.into()),
        sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
        ..Default::default()
    })
    .insert_bundle(RigidBodyBundle {
        mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        position: [position.x / rapier_config.scale, position.y / rapier_config.scale].into(),
        shape: ColliderShape::ball(collider_size * 0.5),
        flags: (ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS).into(),
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete)
    .insert(PlayerMovement {speed: 200.0})
    .insert(PlayerShooting {smoke_mat: materials.add(smoke_texture_handle.into())})
    .insert(crate::lighting::DynamicLightBlocker{size: 20.0})
    .insert( CamFollow{position: Vec2::default()})
    ;
}


fn process_collision_events(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut score: ResMut<Score>,
    mut intersection_events: EventReader<IntersectionEvent>,
    mut contact_events: EventReader<ContactEvent>,
    player_query: Query<Entity, With<PlayerMovement>>,
    enemy_query: Query<Entity, With<crate::ai::AiPerception>>,
    pickup_query: Query<(Entity, &Pickup), With<Pickup>>,
    asset_server: Res<AssetServer>, 
    audio: Res<Audio>
) {
    for intersection_event in intersection_events.iter() {
        if player_query.get(intersection_event.collider1.entity()).is_ok() {
            if let Ok(pair) = pickup_query.get(intersection_event.collider2.entity()) {
                score.value += pair.1.value;
                commands.entity(pair.0).despawn_recursive();
            }
        }
        else if player_query.get(intersection_event.collider2.entity()).is_ok() {
            if let Ok(pair) = pickup_query.get(intersection_event.collider1.entity()) {
                score.value += pair.1.value;
                commands.entity(pair.0).despawn_recursive();

                let fx = asset_server.load("audio/sfx/Stutter_Beep.mp3");
                audio.play(fx);
            }
        }
    }

    for contact_event in contact_events.iter() {
        match contact_event {
            ContactEvent::Started(collider1, collider2) => {
                let contact1_player = player_query.get(collider1.entity()).is_ok();
                let contact2_player = player_query.get(collider2.entity()).is_ok();
                let is_player_involved = contact1_player || contact2_player;
                let contact1_enemy = enemy_query.get(collider1.entity()).is_ok();
                let contact2_enemy = enemy_query.get(collider2.entity()).is_ok();
                let is_enemy_involved = contact1_enemy || contact2_enemy;
                if is_enemy_involved && is_player_involved {
                    state.set(GameState::GameOver).unwrap();
                    let fx = asset_server.load("audio/sfx/deathSound.mp3");
                    audio.play(fx);
                    return;
                }
            }
            _ => {}
        }
        
        
    }
}