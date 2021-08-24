use bevy::{
    prelude::*, 
    math::Vec3Swizzles,
    render::pipeline::{PipelineDescriptor, RenderPipeline},
};
use bevy_rapier2d::prelude::*;
use nalgebra::{point, vector};

use crate::player;
use crate::lighting;

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
        let change_amt = (target_angle - self.angle).max(self.turn_rate * turn_rate_mult);
        self.angle += change_amt;
    }
    pub fn turn(&mut self, direction: f32, turn_rate_mult: f32) {
        let change_amt = direction.signum() * self.turn_rate * turn_rate_mult;
        self.angle += change_amt;
    }
}

pub struct AiPerception {
    pub visual_range: f32,
    pub vision_cone_angle: f32,
    can_see_target: bool,
    target_position: Vec2,
    target_direction: f32,
}

impl AiPerception {
    pub fn new(visual_range: f32, vision_cone_angle: f32) -> AiPerception {
        AiPerception {
            visual_range,
            vision_cone_angle,
            can_see_target: false,
            target_position: Vec2::new(0.0, 0.0),
            target_direction: 0.0,
        }
    }
}

pub struct AiMovement {
    pub move_speed: f32,
    move_to_target: bool,
    target_position: Vec2,
}

impl AiMovement {
    pub fn new(move_speed: f32) -> AiMovement {
        AiMovement {
            move_speed,
            move_to_target: true,
            target_position: Vec2::new(0.0, 0.0)
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

pub fn setup_test_ai_perception(mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: Res<RapierConfiguration>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    render_data: ResMut<lighting::LightRenderData>,
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
        position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
        ..Default::default()
    })
    .insert(ColliderPositionSync::Discrete)
    .insert(Facing{angle: std::f32::consts::PI, turn_rate: std::f32::consts::FRAC_PI_4})//Facing::new(std::f32::consts::FRAC_PI_4))
    .insert(AiPerception::new(250.0, f32::to_radians(30.0)))
    .insert(AiMovement::new(150.0))
    .insert(AiChaseBehavior{})
    .insert(AiPerceptionDebugIndicator{})
    .id();

    let mesh = meshes.add(render_data.base_mesh.clone().unwrap());
    let vision_spotlight = commands.spawn_bundle(MeshBundle {
        mesh: mesh.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            render_data.pipeline_handle.clone().unwrap(),
        )]),
        transform: Transform::from_xyz(0.0, 0.0, 0.1),
        visible: Visible { is_transparent: true, is_visible: true },
        ..Default::default()
    })
    .insert(lighting::SpotLight::new(f32::to_radians(30.0), Color::RED, 250.0))
    .id();

    commands.entity(test_enemy).push_children(&[vision_spotlight]);
}

pub fn ai_perception_system (
    query_pipeline: Res<QueryPipeline>,
    collider_query: QueryPipelineColliderComponentsQuery,
    rapier_config: Res<RapierConfiguration>,
    mut query: Query<(Entity, &mut AiPerception, &Transform, &Facing)>,
    player_query: Query<(&player::PlayerMovement, &Transform)>
) {
    let (_player_movement, player_transform) = player_query.single().expect("There should be exactly 1 player");
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
                if let Ok((_entity, _coll_pos, coll_shape, _coll_flags)) = collider_query.get(handle.entity()) {
                    // Bad way of telling if this is the player for now, since the player is the only ball
                    if coll_shape.shape_type() == ShapeType::Ball {
                        perciever.can_see_target = true;
                        perciever.target_position = rapier_config.scale * Vec2::new(hit_point.x, hit_point.y);
                        perciever.target_direction = Vec2::angle_between(Vec2::new(0.0, 0.0), dir_to_player);
                        continue;
                    }
                }
            }
        }

        // If can see player we continued out of this iteration so if reached here we cannot see
        perciever.can_see_target = false;
    }
}

pub fn ai_movement_system(
    rapier_parameters: Res<RapierConfiguration>,
    mut query: Query<(&mut AiMovement, &mut RigidBodyVelocity, & Transform)>
) {
    for(mut mover, mut rb_vel, transform) in query.iter_mut() {
        let vec_to_target =  mover.target_position - transform.translation.xy();
        let distance_to_target = vec_to_target.length();

        if !mover.move_to_target { 
            rb_vel.linvel = vector![0.0, 0.0];
            continue; 
        }

        if distance_to_target < 50.0 {
            mover.move_to_target = false;
        }
        else {
            // For now, this just dumb zombie moves towards the target
            let move_dir = vec_to_target
                .try_normalize()
                .unwrap_or(Vec2::new(0.0,0.0));
            let movement = move_dir * (1.0 / rapier_parameters.scale) * mover.move_speed;
            rb_vel.linvel = vector![movement.x, movement.y];
        }
    }
}

pub fn ai_chase_behavior_system (
    time: Res<Time>,
    mut query: Query<(&mut AiMovement, &AiPerception, &mut Facing)>
) {
    for(mut mover, perciever, mut facing) in query.iter_mut() {
        if perciever.can_see_target {
            mover.move_to(perciever.target_position);
            facing.turn_towards(perciever.target_direction, time.delta_seconds());
        } 
        else if !mover.is_moving(){
            facing.turn(1.0, time.delta_seconds());
        }
    }
}

pub fn ai_perception_debug_system (
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&AiPerception, &AiPerceptionDebugIndicator, &mut Handle<ColorMaterial>)>,
    mut light_query: Query<(&Parent, &mut lighting::SpotLight)>
) {
    let see_color =Color::rgb(1.0, 0.0, 0.0);
    let cant_color = Color::rgb(0.0,1.0,0.0);

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