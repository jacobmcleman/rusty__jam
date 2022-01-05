use bevy::{
    prelude::*, 
    math::Vec3Swizzles,
    render::pipeline::{RenderPipeline},
    tasks::{ComputeTaskPool,},
};
use bevy_rapier2d::prelude::*;
use nalgebra::{point, vector};
use rand::Rng;

use crate::player;
use crate::lighting;
use crate::level;
use crate::gamestate::GameState;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_update(GameState::Playing)
            .with_system(ai_perception_system.system())
            .with_system(ai_movement_system.system())
            .with_system(ai_chase_behavior_system.system())
            .with_system(ai_perception_debug_system.system())
        );
    }
}

pub struct Facing {
    pub angle: f32,
    pub turn_rate: f32
}

impl Facing {
    pub fn new(turn_rate: f32) -> Facing {
        Facing{ angle: 0.0, turn_rate }
    }
    pub fn forward(&self) -> Vec2 {
        Vec2::new(f32::cos(self.angle), f32::sin(self.angle))
    }
    pub fn turn_towards(&mut self, target_angle: f32, turn_rate_mult: f32) {
        let mut target = target_angle;
        if target < -std::f32::consts::PI { 
            target += std::f32::consts::TAU;
            self.angle +=  std::f32::consts::TAU;
        };
        if target > std::f32::consts::PI { 
            target -= std::f32::consts::TAU;
            self.angle -=  std::f32::consts::TAU;
        }
        let needed_turn = target - self.angle;
        if needed_turn.abs() > std::f32::consts::PI {
            self.angle += std::f32::consts::TAU.copysign(needed_turn);
        }
        let change_amt = needed_turn.abs().min(self.turn_rate * turn_rate_mult).copysign(needed_turn);
        self.angle += change_amt;
    }

    pub fn turn_towards_direction(&mut self, target_forward: Vec2, turn_rate_mult: f32) {
        self.turn_towards(target_forward.y.atan2(target_forward.x), turn_rate_mult);
    }

    pub fn _turn(&mut self, direction: f32, turn_rate_mult: f32) {
        let change_amt = direction.signum() * self.turn_rate * turn_rate_mult;
        self.angle += change_amt;

        if self.angle > std::f32::consts::PI { 
            self.angle -=  std::f32::consts::TAU;
        }
        else if self.angle < -std::f32::consts::PI { 
            self.angle +=  std::f32::consts::TAU;
        }
    }
}

pub struct AiPerception {
    pub visual_range: f32,
    pub vision_cone_angle: f32,
    can_see_target: bool,
    target_position: Vec2,
    target_direction: f32,
    last_seen_time: f64,
}

impl AiPerception {
    pub fn new(visual_range: f32, vision_cone_angle: f32, home_point: Vec2) -> AiPerception {
        AiPerception {
            visual_range,
            vision_cone_angle,
            can_see_target: false,
            target_position: home_point,
            target_direction: 0.0,
            last_seen_time: 0.0,
        }
    }
}

pub struct AiMovement {
    pub move_speed: f32,
    move_to_target: bool,
    target_position: Vec2,
    current_path: Vec<Vec2>,
    path_index: usize
}

impl AiMovement {
    pub fn new(move_speed: f32, start_dest: Vec2) -> AiMovement {
        AiMovement {
            move_speed,
            move_to_target: true,
            target_position: start_dest,
            current_path: vec![],
            path_index: 0,
        }
    }

    // Give an AI Movement component a new thing to move to
    pub fn move_to(&mut self, target: Vec2) {
        self.target_position = target;
        self.move_to_target = true;
    }
    
    pub fn is_moving(&self) -> bool {
        self.move_to_target
    }
    /*
    pub fn halt(&mut self) {
        self.move_to_target = false;
    }
    */
}

pub struct AiChaseBehavior;

pub struct AiPerceptionDebugIndicator;

pub fn spawn_enemy(commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rapier_config: &Res<RapierConfiguration>,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    render_data: & ResMut<lighting::LightRenderData>,
    pos: Vec2,
) {
    // Load sprite
    let circle_texture_handle: Handle<Texture> = asset_server.load("sprites/circle.png");

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    let test_enemy = commands
    .spawn()
    .insert_bundle(SpriteBundle {
        material: materials.add(circle_texture_handle.into()),
        sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
        visible: Visible { is_transparent: true, is_visible: true },
        ..Default::default()
    })
    .insert_bundle(RigidBodyBundle {
        mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        position: [(pos.x / rapier_config.scale) + collider_size_x / 2.0, (pos.y / rapier_config.scale) + collider_size_y / 2.0].into(),
        material: ColliderMaterial { friction: 0.0, restitution: 0.9, ..Default::default() },
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete)
    .insert(Facing::new(std::f32::consts::FRAC_PI_2))
    .insert(AiPerception::new(500.0, f32::to_radians(25.0), pos))
    .insert(AiMovement::new(150.0, pos))
    .insert(AiChaseBehavior{})
    .insert(AiPerceptionDebugIndicator{})
    .insert(crate::lighting::DynamicLightBlocker{size: 25.0})
    .id();

    let mesh = meshes.add(render_data.base_mesh.clone().unwrap());
    let vision_spotlight = commands.spawn_bundle(MeshBundle {
        mesh: mesh.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            render_data.pipeline_handle.clone().unwrap(),
        )]),
        transform: Transform::from_xyz(0.0, 0.0, rand::thread_rng().gen_range(0.1..0.2)),
        visible: Visible { is_transparent: true, is_visible: true },
        ..Default::default()
    })
    .insert(lighting::SpotLight::new(f32::to_radians(25.0), Color::RED, 500.0))
    .insert(lighting::LightMeshData::default())
    .insert(crate::visibility::VisChecker{radius: 500.0, visible: false})
    .id();

    commands.entity(test_enemy).push_children(&[vision_spotlight]);
}

pub fn ai_perception_system (
    query_pipeline: Res<QueryPipeline>,
    collider_query: QueryPipelineColliderComponentsQuery,
    rapier_config: Res<RapierConfiguration>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AiPerception, &Transform, &Facing)>,
    player_query: Query<(&player::PlayerMovement, &Transform, Entity)>
) {
    if let Ok((_player_movement, player_transform, player_entity)) = player_query.single() {
        let player_position = player_transform.translation;

        for (percieve_entity, mut perciever, transform, facing) in query.iter_mut() {
            let collider_set = QueryPipelineColliderComponentsSet(&collider_query);

            let vec_to_player =  player_position.xy() - transform.translation.xy();
            let dir_to_player = vec_to_player
                .try_normalize()
                .unwrap_or(-facing.forward());

            let angle = Vec2::angle_between(facing.forward(), dir_to_player).abs();

            // Easy escape on cheap math checks
            if angle <= perciever.vision_cone_angle && vec_to_player.length_squared() <= (perciever.visual_range * perciever.visual_range) {
                let ray = Ray::new(
                    point![transform.translation.x / rapier_config.scale, transform.translation.y / rapier_config.scale], 
                    vector![dir_to_player.x, dir_to_player.y]);
                let max_toi = perciever.visual_range / rapier_config.scale;
                let solid = true;
                let groups = InteractionGroups::all();
                let filter_func = |handle: ColliderHandle| {
                    handle.entity() != percieve_entity
                };
                let filter: Option<&dyn Fn(ColliderHandle) -> bool> = Some(&filter_func);

                
                if let Some((handle, toi)) = query_pipeline.cast_ray(
                    &collider_set, &ray, max_toi, solid, groups, filter
                ) {
                    let hit_point = ray.point_at(toi);
                    if let Ok((hit_entity, _coll_pos, _coll_shape, _coll_flags)) = collider_query.get(handle.entity()) {
                        // Bad way of telling if this is the player for now, since the player is the only ball
                        if hit_entity == player_entity {
                            perciever.can_see_target = true;
                            perciever.target_position = rapier_config.scale * Vec2::new(hit_point.x, hit_point.y);
                            perciever.target_direction = Vec2::angle_between(Vec2::new(0.0, 0.0), dir_to_player);
                            perciever.last_seen_time = time.seconds_since_startup();
                            continue;
                        }
                    }
                }
            }

            // If can see player we continued out of this iteration so if reached here we cannot see
            perciever.can_see_target = false;

            if perciever.last_seen_time == 0.0 {
                perciever.last_seen_time = time.seconds_since_startup();
            }
        }
    }
}

pub fn ai_movement_system(
    rapier_parameters: Res<RapierConfiguration>,
    time: Res<Time>,
    task_pool: Res<ComputeTaskPool>,
    levels: Res<Assets<level::LevelTiles>>,
    mut query: Query<(&mut AiMovement, &mut RigidBodyVelocity, &mut Facing, &Transform)>,
    level_query: Query<&Handle<level::LevelTiles>,>,
) {
    if let Ok(level_handle) = level_query.single() {
        if let Some(level) = levels.get(level_handle){
            query.par_for_each_mut(&task_pool, 1, |(mut mover, mut rb_vel, mut facing, transform)| {
                if !mover.move_to_target { 
                    rb_vel.linvel = vector![0.0, 0.0];
                    return; 
                }
            
                if mover.current_path.is_empty()                                                    // No path
                    || mover.path_index >= mover.current_path.len()                                 // Run out of path but still thinks need to move
                    || mover.current_path.last().unwrap().distance(mover.target_position) > 60.0    // Last point in path is stale 
                    // Safe to unwrap last since previous check covers the empty case
                { 
                    mover.path_index = 0;
                    // path is stale or non-existent, need to request a new one
                    if let Some(path) = level.get_path(transform.translation.xy(), mover.target_position) {
                        mover.current_path = path;
                        
                    }
                    else {
                        mover.current_path.clear();
                        mover.move_to_target = false;
                        return;
                    }
                }
    
                let vec_to_target =  mover.target_position - transform.translation.xy();
                let distance_to_target = vec_to_target.length();
    
                if distance_to_target < 60.0 {
                    mover.move_to_target = false;
                }
                else {
                    // Move along path
                    let next_point = mover.current_path[mover.path_index];
                    if transform.translation.xy().distance(next_point) < 10.0 {
                        mover.path_index += 1;
                    }
    
                    let to_next_point = (next_point - transform.translation.xy()).normalize();
                    //facing.set_forward(to_next_point);
                    facing.turn_towards_direction(to_next_point, time.delta_seconds());
                    let target_factor = to_next_point.normalize().dot(facing.forward()).clamp(0.0, 1.0).powi(3);
    
    
                    let movement = facing.forward() * target_factor * (1.0 / rapier_parameters.scale) * mover.move_speed;
                    rb_vel.linvel = vector![movement.x, movement.y];
                }
            });
        }
    }
}

pub fn ai_chase_behavior_system (
    time: Res<Time>,
    mut query: Query<(&mut AiMovement, &AiPerception, &mut Facing)>,
) {
    let mut rng = rand::thread_rng();
    for(mut mover, perciever, mut facing) in query.iter_mut() {
        if perciever.can_see_target {
            mover.move_to(perciever.target_position);
            mover.move_speed = rng.gen_range(200.0..300.0);
            facing.turn_rate = std::f32::consts::FRAC_PI_2;
            
        } 
        else if !mover.is_moving(){
            let time_since_seen = time.seconds_since_startup() - perciever.last_seen_time;
            let search_rad_t = (time_since_seen / 90.0) as f32;
            let search_rad = (search_rad_t * 1000.0) + 50.0;
            mover.move_to(perciever.target_position + Vec2::new(rng.gen_range(-search_rad..search_rad), rng.gen_range(-search_rad..search_rad)));
            mover.move_speed = rng.gen_range(50.0..120.0);
            facing.turn_rate = std::f32::consts::FRAC_PI_3;
        }
    }
}

pub fn ai_perception_debug_system (
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&AiPerception, &AiPerceptionDebugIndicator, &mut Handle<ColorMaterial>)>,
    mut light_query: Query<(&Parent, &mut lighting::SpotLight)>
) {
    let see_color =Color::rgb(0.8,0.35,0.2);
    let cant_color = Color::rgb(0.2,0.7,0.8);

    for (perciever, _indicator, mat_handle) in query.iter_mut() {
        if let Some(mut color_mat) = materials.get_mut(mat_handle.id) {
            color_mat.color = if perciever.can_see_target {see_color} else {cant_color};
        }
    } 

    for (parent, mut spotlight) in light_query.iter_mut() {
        if let Ok((perciever, _indicator, _mat_handle)) = query.get_mut(parent.0) {
            spotlight.color = if perciever.can_see_target {see_color} else {cant_color};
        }
    }
}