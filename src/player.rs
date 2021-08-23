use bevy::{math::Vec3Swizzles, prelude::*, render::{mesh, pipeline::{PipelineDescriptor, RenderPipeline}, shader::{ShaderStage, ShaderStages}}};
use bevy_rapier2d::prelude::*;
use nalgebra::{Vector2, vector};

use crate::particles;
use crate::lighting;

pub struct PlayerMovement {
    pub speed: f32,
}

pub struct PlayerShooting {
    smoke_mat: Handle<ColorMaterial>,
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
    query: Query<(&PlayerShooting, &Transform)>
) {
    if let Ok((player, transform)) = query.single() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("player position: {}", transform.translation);

            commands.spawn()
                .insert(particles::BurstParticleEmitter {
                    quantity: 50
                })
                .insert(particles::ParticleEmissionParams {
                    speed_min: 10.0,
                    speed_max: 100.0,
                    particle_drag: 0.001,
                    particle_size: Vec2::new(10.0, 10.0),
                    lifetime_min: 0.1,
                    lifetime_max: 0.5,
                    material: player.smoke_mat.clone(),
                })
                .insert(Transform::from_translation(transform.translation));
            };
    }
}

pub fn setup_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shaders: ResMut<Assets<Shader>>,
    rapier_config: Res<RapierConfiguration>,
    asset_server: Res<AssetServer>,
) {
    // Load sprite
    let circle_texture_handle: Handle<Texture> = asset_server.load("sprites/circle.png");

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, lighting::VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, lighting::FRAGMENT_SHADER))),
    }));

    let mut light_mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    let v_pos = vec![[0.0, 0.0, 0.0], [200.0, 0.0, 0.0], [0.0, 200.0, 0.0]];
    let indices = vec![0, 1, 2];

    light_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    light_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    let mesh = meshes.add(light_mesh);

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
        position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
        shape: ColliderShape::ball(collider_size_x * 0.5),
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete)
    .insert(PlayerMovement {speed: 200.0})
    .insert(PlayerShooting {smoke_mat: materials.add(Color::rgb(0.0, 0.3, 0.5).into())})
    .insert_bundle(MeshBundle {
        mesh: mesh,
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),

        ..Default::default()
    })
    .insert(lighting::PointLight::new(Color::YELLOW, 300.0));
}
