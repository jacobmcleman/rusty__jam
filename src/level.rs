use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn setup_environment(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: Res<RapierConfiguration>,
) {
    create_static_box(&mut commands, &mut materials, &rapier_config, Vec2::new(200.0, 200.0), Vec2::new(100.0, 100.0));
    create_static_box(&mut commands, &mut materials, &rapier_config, Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
}

fn create_static_box(commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rapier_config: &Res<RapierConfiguration>,
    position: Vec2, size: Vec2
) {
    let collider_size_x = size.x / rapier_config.scale;
    let collider_size_y = size.y / rapier_config.scale;

    commands.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        sprite: Sprite::new(size),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        shape: ColliderShape::cuboid(collider_size_x / 2.0, collider_size_y / 2.0),
        collider_type: ColliderType::Solid,
        position: (Vec2::new(position.x / rapier_config.scale, position.y / rapier_config.scale), 0.0).into(),
        material: ColliderMaterial { friction: 0.7, restitution: 0.3, ..Default::default() },
        mass_properties: ColliderMassProps::Density(2.0),
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete);
}