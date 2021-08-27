use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Pickup {
    pub value: i32
}

pub fn spawn_pickup(
    position: Vec2,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rapier_scale: f32,
    asset_server: &Res<AssetServer>,
) {
    let circle_texture_handle: Handle<Texture> = asset_server.load("sprites/circle.png");

    let sprite_size_x = 20.0;
    let sprite_size_y = 20.0;

    let collider_size = sprite_size_x / rapier_scale;

    commands
    .spawn()
    .insert_bundle(SpriteBundle {
        material: materials.add(circle_texture_handle.into()),
        sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        position: [position.x / rapier_scale, position.y / rapier_scale].into(),
        shape: ColliderShape::ball(collider_size * 0.5),
        collider_type: ColliderType::Sensor,
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete)
    .insert(Pickup {value: 1})
    ;
}