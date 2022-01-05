use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset}, 
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use bevy_rapier2d::prelude::*;
use geo::{Coordinate, MultiPolygon, Polygon};
use geo_visibility::Visibility;
use pathfinding::prelude::{absdiff, astar};


#[derive(Clone, PartialEq)]
pub enum TileValue {
    Empty,
    Wall,
    Pickup,
    Player,
    Enemy,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(level_builder_system.system())
            .add_asset::<LevelTiles>()
            .init_asset_loader::<LevelTiles>()
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

#[derive(TypeUuid, Default)]
#[uuid = "47a4a589-01e1-4c15-af08-98b2d0778f28"]
pub struct LevelTiles {
    width: usize,
    height: usize,
    tile_size: f32,
    tiles: Vec<TileValue>,
    pickups_total: i32,
    _next_level: String, // TODO: Add win condition so this can do something
}

impl AssetLoader for LevelTiles {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut tiles = Vec::<TileValue>::new();
            let mut width = 0;
            let mut height = 0;
            let mut index = 0;
            let mut next_level: String = "".to_string();
            let mut read_name = true;
            let mut pickups_total = 0;

            for byte in bytes {
                if read_name {
                    let character = *byte as char;
                    if character == '\n' {
                        read_name = false;
                        println!("Next Level will be {}", next_level);
                    }
                    else {
                        next_level.push( *byte as char);
                    }
                    
                }
                else{
                    match *byte as char {
                        ' ' => { tiles.push(TileValue::Empty); index += 1; },
                        '#' => { tiles.push(TileValue::Wall); index += 1; },
                        '$' => { 
                            tiles.push(TileValue::Pickup); 
                            index += 1; 
                            pickups_total += 1;
                        },
                        'V' => { tiles.push(TileValue::Player); index += 1; },
                        'X' => { tiles.push(TileValue::Enemy); index += 1; },
                        '\n' => {
                            if width == 0 { width = index; }
                            height += 1;
                        },
                        _ => ()
                    }
                }
            }

            load_context.set_default_asset(LoadedAsset::new(LevelTiles{width, height, tile_size: 50.0, tiles, pickups_total, _next_level: next_level}));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level"]
    }
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
        if self.get_tile(pos_test) != TileValue::Wall {
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

    /*fn to_level_objects(&self) -> LevelObjects {

    }*/
}

fn get_tile_index(x: usize, y: usize, width: usize) -> usize {
    x+ (y * width)
}

fn count_wall_continues_x(tiles: &Vec<bool>, width: usize, _height: usize, start: &IVec2) -> usize {
    let mut x = start.x as usize;
    let y = start.y as usize;
    let mut cur_length = 0;
    let mut index = get_tile_index(x, y, width);

    while x < width && tiles[index] {
        x += 1;
        cur_length += 1;
        index = get_tile_index(x, y, width); // May be same as +1 for now but safety
    }

    return cur_length;
}

fn count_wall_continues_y(tiles: &Vec<bool>, width: usize, height: usize, start: &IVec2) -> usize {
    let x = start.x as usize;
    let mut y = start.y as usize;
    let mut cur_length = 0;
    let mut index = get_tile_index(x, y, width);

    while y < height && tiles[index] {
        y += 1;
        cur_length += 1;
        index = get_tile_index(x, y, width);
    }

    return cur_length;
}

fn take_longest_wall(tiles: &mut Vec<bool>, width: usize, height: usize, root: &IVec2) -> Wall {
    let x_wall_length = count_wall_continues_x(&tiles, width, height, &root);
    let y_wall_length = count_wall_continues_y(&tiles, width, height, &root);

    if x_wall_length > y_wall_length {
        let mut x = root.x as usize;
        while x < root.x as usize + x_wall_length {
            tiles[get_tile_index(x, root.y as usize, width)] = false;
            x += 1;
        }
        return Wall{top_left: root.clone(), bottom_right: IVec2::new(x as i32 - 1 , root.y)};
    }
    else {
        let mut y = root.y as usize;
        while y < root.y as usize + y_wall_length {
            tiles[get_tile_index(root.x as usize, y, width)] = false;
            y += 1;
        }
        return Wall{top_left: root.clone(), bottom_right: IVec2::new(root.x, y as i32 - 1)};
    }
}

fn tile_vector_to_wall_set(tiles: &Vec<TileValue>, width: usize, height: usize) -> Vec<Wall> {
    let mut remaining_wall_tiles = Vec::<bool>::new();
    let mut walls = Vec::<Wall>::new();

    for tile in tiles {
        remaining_wall_tiles.push(tile.clone() == TileValue::Wall);
    }

    for x in 0..width {
        for y in 0..height {
            let index = get_tile_index(x, y, width);

            if remaining_wall_tiles[index] {
                walls.push(take_longest_wall(&mut remaining_wall_tiles, width, height, &IVec2::new(x as i32, y as i32)));
            }
        }
    }

    return walls;
}

#[derive(Eq, Debug)]
struct Wall {
    top_left: IVec2,
    bottom_right: IVec2,
}

impl Wall {
    fn get_center(&self, tile_size: f32) -> Vec2 {
        let top_left = self.top_left.as_f32() * tile_size;
        let bottom_right = self.bottom_right.as_f32() * tile_size;
        return 0.5 * (top_left + bottom_right);
    }

    fn get_size(&self, tile_size: f32) -> Vec2 {
        let tile_height = (self.top_left.y - self.bottom_right.y).abs();
        let tile_width = (self.top_left.x - self.bottom_right.x).abs();
        return Vec2::new(tile_width as f32 * tile_size, tile_height as f32 * tile_size);
    }
}

impl PartialEq for Wall {
    fn eq(&self, other: &Wall) -> bool {
        self.top_left == other.top_left && self.bottom_right == other.bottom_right
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
    asset_server: Res<AssetServer>,
    current_level: Res<crate::gamestate::CurrentLevel>,
) {
    let level_path = "levels/".to_string() + &current_level.name + ".level";
    println!("Preparing level: {}", level_path);
    let level_handle: Handle<LevelTiles> = asset_server.load(&level_path as &str);
    spawn_level(&mut commands, level_handle);
}

pub fn spawn_level(commands: &mut Commands, level: Handle<LevelTiles>) {
    commands.spawn()
        .insert(level)
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
        material: materials.add(Color::rgb(0.4, 0.3, 0.6).into()),
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
    levels: Res<Assets<LevelTiles>>,
    rapier_config: Res<RapierConfiguration>,
    asset_server: Res<AssetServer>,
    render_data: ResMut<crate::lighting::LightRenderData>,
    mut score: ResMut<crate::gamestate::Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut level_query: Query<(&mut LevelState, &Handle<LevelTiles>, &mut LevelGeo)>
) {
    if let Ok((mut level_state, level_data_handle, mut level_geo)) = level_query.single_mut() {
        if level_state.built { return; }

        if let Some(level_data) = levels.get(level_data_handle){
            let mut level_polygons = Vec::<Polygon<f64>>::new();

            score.max = level_data.pickups_total;

            let offset = Vec2::new((level_data.width / 2) as f32 * -level_data.tile_size, (level_data.height / 2) as f32 * -level_data.tile_size);

            for y in 0..level_data.height {
                for x in 0..level_data.width {
                    let tile_pos = offset + Vec2::new(level_data.tile_size * x as f32, level_data.tile_size * y as f32);
                    if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Wall) {
                        create_static_box(
                            &mut commands, 
                            &mut materials, 
                            &rapier_config, 
                            &mut level_polygons,
                            tile_pos, 
                            Vec2::new(level_data.tile_size, level_data.tile_size));
                    }
                    else if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Pickup) {
                        crate::pickup::spawn_pickup(tile_pos,
                            &mut commands,
                            &mut materials,
                            rapier_config.scale,
                            &asset_server,
                        );
                    }
                    else if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Player) {
                        crate::player::spawn_player(
                            tile_pos,
                            &mut commands,
                            &mut materials,
                            &rapier_config,
                            &asset_server,
                        );
                    }
                    else if matches!(level_data.tiles[x + (y * level_data.width)], TileValue::Enemy) {
                        crate::ai::spawn_enemy(
                            &mut commands, 
                            &mut materials, 
                            &rapier_config, 
                            &asset_server, 
                            &mut meshes, 
                            &render_data, 
                            tile_pos
                        );
                    }
                }
            }

            level_geo.level_blocks = level_polygons;

            level_state.built = true;
        }
    }
}

pub fn get_visibility_polygon(level_geo: &LevelGeo, from_point: Vec2) -> Polygon<f64>{
    let point = geo::Point::new(from_point.x as f64, from_point.y as f64);
    return point.visibility(&level_geo.get_geo_multipoly());
}

fn _gen_level_tiles(width: usize, height: usize) -> LevelTiles {
    let mut tiles = Vec::<TileValue>::new();
    for y in 0..height {
        for x in 0..width {
            tiles.push(
                if (((x * y) % 3 == 1) && ((x * y / 3) % 4 == 1)) || x * y == 0 || x == width - 1 || y == height - 1 {TileValue::Wall} 
                else {TileValue::Empty}
            );
        }
    }
    LevelTiles { width, height, tile_size: 50.0, tiles, _next_level: "".to_string(), pickups_total: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_rect_single_tile() {
        let wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(1, 1)};
        assert_eq!(wall.get_center(10.0), Vec2::new(5.0, 5.0));
        assert_eq!(wall.get_size(10.0), Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_wall_rect_long_x() {
        let wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(10, 1)};
        assert_eq!(wall.get_center(10.0), Vec2::new(50.0, 5.0));
        assert_eq!(wall.get_size(10.0), Vec2::new(100.0, 10.0));
    }

    #[test]
    fn test_wall_rect_long_y() {
        let wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(1, 10)};
        assert_eq!(wall.get_center(10.0), Vec2::new(5.0, 50.0));
        assert_eq!(wall.get_size(10.0), Vec2::new(10.0, 100.0));
    }

    #[test]
    fn test_wall_rect_square() {
        let wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(10, 10)};
        assert_eq!(wall.get_center(10.0), Vec2::new(50.0, 50.0));
        assert_eq!(wall.get_size(10.0), Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_wall_x_length_single() {
        let test_grid = vec![   true,  false, false, 
                                false, false, false,
                                false, false, false,
                            ];
        
        assert_eq!(count_wall_continues_x(&test_grid, 3, 3, &IVec2::new(0, 0)), 1);
    }

    #[test]
    fn test_wall_x_length_mid() {
        let test_grid = vec![   true,  true,  false, 
                                false, false, false,
                                false, false, false,
                            ];
        
        assert_eq!(count_wall_continues_x(&test_grid, 3, 3, &IVec2::new(0, 0)), 2);
    }

    #[test]
    fn test_wall_x_length_wholeside() {
        let test_grid = vec![   true,  true,  true, 
                                false, false, false,
                                false, false, false,
                            ];
        
        assert_eq!(count_wall_continues_x(&test_grid, 3, 3, &IVec2::new(0, 0)), 3);
    }

    #[test]
    fn test_wall_y_length_single() {
        let test_grid = vec![   true,  false, false, 
                                false, false, false,
                                false, false, false,
                            ];
        
        assert_eq!(count_wall_continues_y(&test_grid, 3, 3, &IVec2::new(0, 0)), 1);
    }

    #[test]
    fn test_wall_y_length_mid() {
        let test_grid = vec![   true,  false,  false, 
                                true,  false,  false,
                                false, false,  false,
                            ];
        
        assert_eq!(count_wall_continues_y(&test_grid, 3, 3, &IVec2::new(0, 0)), 2);
    }

    #[test]
    fn test_wall_y_length_wholeside() {
        let test_grid = vec![   true,  false, false, 
                                true,  false, false,
                                true,  false, false,
                            ];
        
        assert_eq!(count_wall_continues_y(&test_grid, 3, 3, &IVec2::new(0, 0)), 3);
    }

    #[test]
    fn test_takes_longest_wall_x() {
        let mut test_grid = vec![   true,  true,  true, 
                                true,  false, false,
                                false, false, false,
                            ];
        let expected_grid = vec![   false, false, false, 
                                    true,  false, false,
                                    false, false, false,
                            ];
        let expected_wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(2, 0)};
        
        let result_wall = take_longest_wall(&mut test_grid, 3, 3, &IVec2::new(0, 0));
        assert_eq!(result_wall.top_left, expected_wall.top_left, "Return correct wall top left bound");
        assert_eq!(result_wall.bottom_right, expected_wall.bottom_right, "Return correct wall bottom right bound");
        assert_eq!(test_grid, expected_grid, "Properly mutate grid");
    }

    #[test]
    fn test_takes_longest_wall_y() {
        let mut test_grid = vec![   true,  true, false, 
                                true,  false, false,
                                true,  false, false,
                            ];
        let expected_grid = vec![   false, true,  false, 
                                    false, false, false,
                                    false, false, false,
                            ];
        let expected_wall = Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(0, 2)};

        let result_wall = take_longest_wall(&mut test_grid, 3, 3, &IVec2::new(0, 0));
        assert_eq!(result_wall.top_left, expected_wall.top_left, "Return correct wall top left bound");
        assert_eq!(result_wall.bottom_right, expected_wall.bottom_right, "Return correct wall bottom right bound");
        assert_eq!(test_grid, expected_grid, "Properly mutate grid");
    }

    #[test]
    fn test_makes_walls_steps() {
        let mut test_grid = vec![   TileValue::Wall,  TileValue::Empty, TileValue::Empty, 
                                    TileValue::Wall,  TileValue::Wall, TileValue::Empty,
                                    TileValue::Wall,  TileValue::Wall, TileValue::Wall,
                            ];
        let expected_walls = vec! [
            Wall{top_left: IVec2::new(0, 0), bottom_right: IVec2::new(0, 2)},
            Wall{top_left: IVec2::new(1, 1), bottom_right: IVec2::new(1, 2)},   
            Wall{top_left: IVec2::new(2, 2), bottom_right: IVec2::new(2, 2)},   
        ];

        let result_walls = tile_vector_to_wall_set(&mut test_grid, 3, 3);
        
        assert_eq!(result_walls.len(), 3, "3 walls expected");

        for i in 0..3 {
            assert_eq!(result_walls[i], expected_walls[i], "Wall {} matches", i);
        }
    }

    #[test]
    fn test_makes_walls_empty() {
        let mut test_grid = vec![   TileValue::Empty,  TileValue::Empty, TileValue::Empty, 
                                    TileValue::Empty,  TileValue::Empty, TileValue::Empty,
                                    TileValue::Empty,  TileValue::Empty, TileValue::Empty,
                            ];
        let result_walls = tile_vector_to_wall_set(&mut test_grid, 3, 3);
        
        assert_eq!(result_walls.len(), 0, "Empty grid should produce no walls");
    }

    #[test]
    fn test_makes_walls_x() {
        let mut test_grid = vec![   TileValue::Empty,  TileValue::Wall, TileValue::Empty, 
                                    TileValue::Wall,   TileValue::Wall, TileValue::Wall,
                                    TileValue::Empty,  TileValue::Wall, TileValue::Empty,
                            ];
        let expected_walls = vec! [
            Wall{top_left: IVec2::new(0, 1), bottom_right: IVec2::new(2, 1)},
            Wall{top_left: IVec2::new(1, 0), bottom_right: IVec2::new(1, 0)},   
            Wall{top_left: IVec2::new(1, 2), bottom_right: IVec2::new(1, 2)},   
        ];
        let result_walls = tile_vector_to_wall_set(&mut test_grid, 3, 3);
        
        assert_eq!(result_walls.len(), 3, "3 walls expected");

        for i in 0..3 {
            assert_eq!(result_walls[i], expected_walls[i], "Wall {} matches", i);
        }
    }
}