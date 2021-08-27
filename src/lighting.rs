use bevy::{math::Vec3Swizzles, prelude::*, render::{camera::OrthographicProjection, pipeline::{BlendOperation, PipelineDescriptor}, shader::{ShaderStage, ShaderStages}}, tasks::ComputeTaskPool};
use geo::coords_iter::CoordsIter;
use geo::{Polygon,};

use crate::{MainCam, level};
use crate::ai::Facing;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(LightRenderData {
                pipeline_handle: None,
                base_mesh: None
            })
            .add_startup_system(light_setup_system.system().label("graphics_init"))
            .add_system(point_light_mesh_builder.system().label("light_build").after("light_setup"))
            .add_system(spotlight_mesh_builder.system().after("light_setup"))
            .add_system(test_spin_system.system())
            .add_system(dynamic_light_blocking_system.system().label("light_setup"))
            .add_system(light_mesh_applicator.system().after("light_build"))
        ;
    }
}

pub struct LightRenderData {
    pub pipeline_handle: Option<Handle<PipelineDescriptor>>,
    pub base_mesh: Option<Mesh>
}

pub struct PointLight {
    mesh_built: bool,
    pub color: Color,
    pub reach: f32
}

impl PointLight {
    pub fn _new(color: Color, reach: f32) -> PointLight {
        PointLight{mesh_built: false, color, reach}
    }
}

pub struct SpotLight {
    mesh_built: bool,
    pub color: Color,
    pub angle: f32,
    pub reach: f32
}


#[derive(Default)]
pub struct LightMeshData {
    v_pos: Vec<[f32; 3]>,
    v_color: Vec<[f32; 3]>,
    v_lightpos: Vec<[f32; 3]>,
    v_lightpower: Vec<f32>,
    v_lightfacing: Vec<f32>,
    v_lightangle: Vec<f32>,
    indices: Vec<u32>,
}

impl SpotLight {
    pub fn new(angle: f32, color: Color, reach: f32) -> SpotLight {
        SpotLight{mesh_built: false, color, angle, reach}
    }
}

pub struct TestSpin {}

pub struct DynamicLightBlocker {
    pub size: f32
}

impl DynamicLightBlocker {
    fn get_poly(&self, position: Vec2) -> Polygon<f64> {
        geo::Rect::new(
            level::bevy_vec2_to_geo_coord(position + Vec2::new(-0.5 * self.size,-0.5 * self.size)),
            level::bevy_vec2_to_geo_coord(position + Vec2::new(0.5 * self.size,0.5 * self.size)),
        ).into()
    }
}

pub fn dynamic_light_blocking_system(
    mut level_query: Query<&mut level::LevelGeo>,
    blocker_query: Query<(&DynamicLightBlocker, &Transform)>
) {
    if let Ok(mut level) = level_query.single_mut() {
        level.reset_temps_for_next_frame();
        for (blocker, transform) in blocker_query.iter() {
            level.temp_block(blocker.get_poly(transform.translation.xy()));
        }
    }
}

fn build_mesh_for_vis_poly(poly: &geo::Polygon<f64>, mesh: &mut LightMeshData, center: Vec2, z: f32, color: Color, reach: f32) {
    build_mesh_for_vis_poly_cone(poly, mesh, center, z, color, reach, 0.0, 4.0);
}

fn build_mesh_for_vis_poly_cone(poly: &geo::Polygon<f64>, mesh: &mut LightMeshData, center: Vec2, z: f32, color: Color, reach: f32, facing: f32, angle: f32) {
    let mut v_pos = vec![[center.x, center.y, z]];
    let mut v_color = vec![[color.r(), color.g(), color.b()]];
    let mut v_lightpos = vec![[center.x, center.y, z]];
    let mut v_lightpower = vec![reach];
    let mut v_lightfacing = vec![facing];
    let mut v_lightangle = vec![angle];
    let mut indices = vec![0, 1, 2];
    

    let mut point_index = 1;
    for point in poly.exterior_coords_iter()
    {
        let vecpoint = Vec2::new(point.x as f32, point.y as f32);

        v_pos.push([vecpoint.x, vecpoint.y, z]);
        v_color.push([color.r(), color.g(), color.b()]);
        v_lightpos.push([center.x, center.y, z]);
        v_lightpower.push(reach);
        v_lightfacing.push(facing);
        v_lightangle.push(angle);

        if point_index != 1 {
            indices.push(0);
            indices.push(point_index);
            indices.push(point_index - 1);
        }

        point_index += 1;
    }

    mesh.v_pos = v_pos;
    mesh.v_color = v_color;
    mesh.v_lightpos = v_lightpos;
    mesh.v_lightpower = v_lightpower;
    mesh.v_lightfacing = v_lightfacing;
    mesh.v_lightangle = v_lightangle;
    mesh.indices = indices;
}

pub fn light_mesh_applicator(
    mut query: Query<(&LightMeshData, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mesh_data, mesh_handle) in query.iter_mut() {
        if let Some(mesh) = meshes.get_mut(mesh_handle.id) {
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.v_pos.clone());
            mesh.set_attribute("light_Color", mesh_data.v_color.clone());
            mesh.set_attribute("light_Position", mesh_data.v_lightpos.clone());
            mesh.set_attribute("light_Power", mesh_data.v_lightpower.clone());
            mesh.set_attribute("light_Facing", mesh_data.v_lightfacing.clone());
            mesh.set_attribute("light_Angle", mesh_data.v_lightangle.clone());
            mesh.set_indices(Some(bevy::render::mesh::Indices::U32(mesh_data.indices.clone())));
        }
    }
}

pub fn point_light_mesh_builder(
    mut query: Query<(&mut PointLight, &GlobalTransform, &mut LightMeshData)>,
    mut level_query: Query<&mut level::LevelGeo>
) {
    if let Ok(mut level_geo) = level_query.single_mut() {
        for (mut light, transform, mut mesh_data) in query.iter_mut() {
            let center: Vec2 = transform.translation.xy();
            let vis_polygon = level::get_visibility_polygon(&mut level_geo, center);
            build_mesh_for_vis_poly(&vis_polygon, &mut mesh_data, center, transform.translation.z, light.color, light.reach);
            light.mesh_built = true;
        }
    }
}

fn circle_intersect_rect(r: f32, center: Vec2, corner_a: Vec2, corner_b: Vec2) -> bool{
    let closest_point_to_circle_in_rect = Vec2::new(
        corner_a.x.max(corner_b.x.min(center.x)),
        corner_a.y.max(corner_b.y.min(center.y)),
    );
    let dist_sqd = center.distance_squared(closest_point_to_circle_in_rect);
    return dist_sqd < r.powi(2);
}

pub fn spotlight_mesh_builder(
    mut query: Query<(&mut SpotLight, &GlobalTransform, &Parent, &mut LightMeshData)>,
    cam_query: Query<(&Transform, &OrthographicProjection), With<MainCam>>,
    parent_query: Query<&Facing, With<Children>>,
    level_query: Query<&level::LevelGeo>,
    task_pool: Res<ComputeTaskPool>,
) {
    if let Ok((cam_transform, cam_ortho)) = cam_query.single() {
        let cam_upper_left = cam_transform.translation.xy() + Vec2::new(cam_ortho.left, cam_ortho.top);
        let cam_lower_right = cam_transform.translation.xy() + Vec2::new(cam_ortho.right, cam_ortho.bottom);
        if let Ok(level_geo) = level_query.single() {
            query.par_for_each_mut(&task_pool, 1, |(mut light, transform, parent, mut mesh_data)| {
                if let Ok(facing) = parent_query.get(parent.0) {
                    let center: Vec2 = transform.translation.xy() + facing.forward() * 20.0;
                    // Check if there is any chance of it being visible at all
                    if circle_intersect_rect(light.reach, center, cam_upper_left, cam_lower_right) {
                        let vis_polygon = level::get_visibility_polygon(&level_geo, center);
                        build_mesh_for_vis_poly_cone(&vis_polygon, &mut mesh_data, center, transform.translation.z, light.color, light.reach, facing.angle, light.angle);
                        light.mesh_built = true;
                    }
                }
            });
        }
    }
}


pub fn test_spin_system(
    mut query: Query<(&mut crate::ai::Facing, &TestSpin)>,
    time: Res<Time>,
) {
    for (mut face, _spin ) in query.iter_mut() {
        face.angle += face.turn_rate * time.delta_seconds();
    }
}


pub fn light_setup_system(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_data: ResMut<LightRenderData>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    
    let mut pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    });

    for color_state in &mut pipeline.color_target_states {
        color_state.alpha_blend.operation = BlendOperation::Add;
        color_state.color_blend.operation = BlendOperation::Add;
    }

    render_data.pipeline_handle = Some(pipelines.add(pipeline));
    let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    

    let v_pos = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]];
    let v_color = vec![[1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 0.0]];
    let v_lightpos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let v_lightpower = vec![1.0, 1.0, 1.0];
    let v_lightfacing = vec![0.0, 0.0, 0.0];
    let v_lightangle = vec![1.0, 1.0, 1.0];
    let indices = vec![0, 1, 2];

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    mesh.set_attribute("light_Color", v_color);
    mesh.set_attribute("light_Position", v_lightpos);
    mesh.set_attribute("light_Power", v_lightpower);
    mesh.set_attribute("light_Facing", v_lightfacing);
    mesh.set_attribute("light_Angle", v_lightangle);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    render_data.base_mesh = Some(mesh);
}

pub const VERTEX_SHADER: &str = r"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 3) out vec3 position;
layout(location = 1) in vec3 light_Color;
layout(location = 1) out vec3 l_Color;
layout(location = 2) in vec3 light_Position;
layout(location = 2) out vec3 l_Position;
layout(location = 4) in float light_Power;
layout(location = 4) out float l_power;
layout(location = 5) in float light_Facing;
layout(location = 5) out float l_Facing;
layout(location = 6) in float light_Angle;
layout(location = 6) out float l_Angle;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
void main() {
    l_Color = light_Color;
    l_Position = light_Position;
    l_power = light_Power;
    l_Facing = light_Facing;
    l_Angle = light_Angle;
    gl_Position = ViewProj * vec4(Vertex_Position, 1.0);
    position = Vertex_Position;
}
";

pub const FRAGMENT_SHADER: &str = r"
#version 450
layout(location = 1) in vec3 l_Color;
layout(location = 2) in vec3 l_Position;
layout(location = 3) in vec3 position;
layout(location = 4) in float l_power;
layout(location = 5) in float l_Facing;
layout(location = 6) in float l_Angle;
layout(location = 0) out vec4 o_Target;
void main() {
    vec3 to_source = l_Position - position;
    float distance = length(to_source);
    to_source = to_source / distance;
    vec3 facing = vec3(-cos(l_Facing), -sin(l_Facing), to_source.z);
    float angle = acos(dot(to_source, facing));
    float angle_falloff = l_Angle <= angle ? 0 : 1;//clamp(1 - (angle / l_Angle), 0.0, 1.0);
    float light_power = clamp(1 - (distance / l_power), 0.0, 1.0);
    light_power = pow(light_power, 3) * angle_falloff;
    o_Target = vec4(l_Color.x, l_Color.y, l_Color.z, light_power);
}
";