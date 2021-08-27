use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use geo::{Coordinate, MultiPolygon, Polygon};
use geo_visibility::Visibility;
use pathfinding::prelude::{absdiff, astar};


#[derive(Clone, PartialEq)]
pub enum TileValue {
    Empty,
    Wall,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(level_builder_system.system())
        ;
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GridPos {
    pub x: i32, pub y: i32
}

impl GridPos {
    fn distance(&self, other: &GridPos) -> u32 {
        (absdiff(self.x, other.x) + absdiff(self.y, other.y)) as u32
    }
}

pub struct LevelTiles {
    width: usize,
    height: usize,
    tile_size: f32,
    tiles: Vec<TileValue>
}

impl LevelTiles {
    pub fn get_path(&self, from: Vec2, to: Vec2) -> Option<Vec<Vec2>> {
        let goal = self.world_to_grid(to);
        let start = self.world_to_grid(from);
        let result = astar(
            &start, 
            |pos| self.successors(pos),
            |pos|  pos.distance(&goal) / 3,
            |pos| *pos == goal
        );

        if let Some((grid_points, _path_length)) = result {
            return Some(grid_points.into_iter().map(|pos| self.grid_to_world(pos)).collect::<Vec<Vec2>>());
        }
        else {
            return None;
        }
    }


    fn grid_to_world(&self, pos: GridPos) -> Vec2 {
        Vec2::new(
            (self.width / 2) as f32 * -self.tile_size + (pos.x as f32 * self.tile_size), 
            (self.height / 2) as f32 * -self.tile_size + (pos.y as f32 * self.tile_size)
        )
    }

    fn world_to_grid(&self, pos: Vec2) -> GridPos {
        GridPos {
            x: ((pos.x) / self.tile_size).round() as i32 + (self.width as i32 / 2),
            y: ((pos.y) / self.tile_size).round() as i32 + (self.height as i32 / 2),
        }
    }

    fn get_tile(&self, pos: &GridPos) -> TileValue {
        if pos.x < 0 || pos.y < 0 || pos.x as usize > self.width || pos.y as usize > self.height {
             return TileValue::Wall; 
        }

        return self.tiles[pos.x as usize + (pos.y as usize * self.width)].clone();
    }

    fn test_successor(&self, pos_test: &GridPos, successor_vec: &mut Vec<(GridPos, u32)>, cost: u32) -> bool{
        if self.get_tile(pos_test) == TileValue::Empty {
            successor_vec.push((pos_test.clone(), cost));
            return true;
        }
        return false;
    }

    fn successors(&self, pos: &GridPos) -> Vec<(GridPos, u32)> {
        let mut successors = Vec::<(GridPos, u32)>::new();
        let east = self.test_successor(&GridPos{x: pos.x + 1, y: pos.y}, &mut successors, 2);
        let west = self.test_successor(&GridPos{x: pos.x - 1, y: pos.y}, &mut successors, 2);
        let north = self.test_successor(&GridPos{x: pos.x, y: pos.y + 1}, &mut successors, 2);
        let south = self.test_successor(&GridPos{x: pos.x, y: pos.y - 1}, &mut successors, 2);
        if east && north { self.test_successor(&GridPos{x: pos.x + 1, y: pos.y + 1}, &mut successors, 3); }
        if east && south { self.test_successor(&GridPos{x: pos.x + 1, y: pos.y - 1}, &mut successors, 3); }
        if west && north { self.test_successor(&GridPos{x: pos.x - 1, y: pos.y + 1}, &mut successors, 3); }
        if west && south { self.test_successor(&GridPos{x: pos.x - 1, y: pos.y - 1}, &mut successors, 3); }
        return successors;
    }
}

pub struct LevelGeo {
    level_blocks: Vec<Polygon<f64>>,
    temp_blocks: Vec<Polygon<f64>>,
}

impl LevelGeo {
    pub fn temp_block(&mut self, block: Polygon<f64>) {
        self.temp_blocks.push(block);
    }

    pub fn get_geo_multipoly(&self) -> MultiPolygon<f64> {
        let mut all_blocks = self.temp_blocks.clone();
        all_blocks.append(&mut self.level_blocks.clone());
        return MultiPolygon(all_blocks);
    }

    pub fn reset_temps_for_next_frame(&mut self) {
        self.temp_blocks.clear();
    }
}

pub struct LevelState {
    built: bool
}

pub fn setup_environment(
    mut commands: Commands,
) {
    spawn_level(&mut commands);
}

pub fn spawn_level(commands: &mut Commands) {
    commands.spawn()
        .insert(gen_level_tiles(24, 24))
        .insert(LevelState{built:false})
        .insert(LevelGeo{level_blocks: vec![], temp_blocks: vec![]});
}

pub fn bevy_vec2_to_geo_coord(bv: Vec2) -> Coordinate<f64> {
    Coordinate{x: bv.x as f64, y: bv.y as f64}
}

fn create_static_box(commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rapier_config: &Res<RapierConfiguration>,
    level_geo: &mut Vec<Polygon<f64>>,
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
    
    let min_point = position + (-0.5 * size);
    let max_point = position + (0.5 * size);

    level_geo.push(geo::Rect::new(bevy_vec2_to_geo_coord(min_point), bevy_vec2_to_geo_coord(max_point)).into());
}

pub fn level_builder_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: Res<RapierConfiguration>,
    mut level_query: Query<(&mut LevelState, &LevelTiles, &mut LevelGeo)>
) {
    if let Ok((mut level_state, level_data, mut level_geo)) = level_query.single_mut() {
        if level_state.built { return; }

        let mut level_polygons = Vec::<Polygon<f64>>::new();

        let offset = Vec2::new((level_data.width / 2) as f32 * -level_data.tile_size, (level_data.height / 2) as f32 * -level_data.tile_size);

        for y in 0..level_data.height {
            for x in 0..level_data.width {
                if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Wall) {
                    create_static_box(
                        &mut commands, 
                        &mut materials, 
                        &rapier_config, 
                        &mut level_polygons,
                        offset + Vec2::new(level_data.tile_size * x as f32, level_data.tile_size * y as f32), 
                        Vec2::new(level_data.tile_size, level_data.tile_size));
                }
            }
        }

        level_geo.level_blocks = level_polygons;

        level_state.built = true;
    }
}

pub fn get_visibility_polygon(level_geo: &mut LevelGeo, from_point: Vec2) -> Polygon<f64>{
    let point = geo::Point::new(from_point.x as f64, from_point.y as f64);
    return point.visibility(&level_geo.get_geo_multipoly());
}

fn gen_level_tiles(width: usize, height: usize) -> LevelTiles {
    let mut tiles = Vec::<TileValue>::new();
    for y in 0..height {
        for x in 0..width {
            tiles.push(
                if (((x * y) % 3 == 1) && ((x * y / 3) % 4 == 1)) || x * y == 0 || x == width - 1 || y == height - 1 {TileValue::Wall} 
                else {TileValue::Empty}
            );
        }
    }
    LevelTiles { width, height, tile_size: 50.0, tiles }
}