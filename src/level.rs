use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub enum TileValue {
    Empty,
    Wall
}

pub struct LevelTiles {
    width: usize,
    height: usize,
    tiles: Vec<TileValue>
}

pub struct LevelState {
    built: bool
}

pub fn setup_environment(
    mut commands: Commands,
) {
    commands.spawn()
        .insert(gen_level_tiles(10, 10))
        .insert(LevelState{built:false});

    //create_static_box(&mut commands, &mut materials, &rapier_config, Vec2::new(200.0, 200.0), Vec2::new(100.0, 100.0));
    //create_static_box(&mut commands, &mut materials, &rapier_config, Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
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

pub fn level_builder_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: Res<RapierConfiguration>,
    mut level_query: Query<(&mut LevelState, &LevelTiles)>
) {
    if let Ok((mut level_state, level_data)) = level_query.single_mut() {
        if level_state.built { return; }

        let tile_size = 50.0;

        let offset = Vec2::new((level_data.width / 2) as f32 * -tile_size, (level_data.height / 2) as f32 * -tile_size);

        for y in 0..level_data.height {
            for x in 0..level_data.width {
                if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Wall) {
                    create_static_box(
                        &mut commands, 
                        &mut materials, 
                        &rapier_config, 
                        offset + Vec2::new(tile_size * x as f32, tile_size * y as f32), 
                        Vec2::new(tile_size, tile_size));
                }
            }
        }

        level_state.built = true;
    }
}

fn gen_level_tiles(width: usize, height: usize) -> LevelTiles {
    let mut tiles = Vec::<TileValue>::new();
    for y in 0..height {
        for x in 0..width {
            tiles.push(if (x * y) % 2 == 1 || x * y == 0 {TileValue::Wall} else {TileValue::Empty});
        }
    }
    LevelTiles { width, height, tiles }
}