use bevy::{math::Vec3Swizzles, prelude::*, render::{mesh, pipeline::{BlendOperation, PipelineDescriptor, RenderPipeline}, shader::{ShaderStage, ShaderStages}}};
use geo::coords_iter::CoordsIter;
use crate::level::{self, get_visibility_polygon};

pub struct PointLight {
    mesh_built: bool,
    color: Color,
    reach: f32
}

impl PointLight {
    pub fn new(color: Color, reach: f32) -> PointLight {
        PointLight{mesh_built: false, color, reach}
    }
}

pub fn point_light_mesh_builder(
    mut query: Query<(&mut PointLight, &Transform, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    level_query: Query<&level::LevelGeo>
) {
    if let Ok(level_geo) = level_query.single() {
        for (mut light, transform, mesh_handle) in query.iter_mut() {
            //if light.mesh_built { continue; }

            let center: Vec2 = transform.translation.xy();

            let fixed_z = transform.translation.z;

            let mut v_pos = vec![[center.x, center.y, fixed_z]];
            let mut v_color = vec![[light.color.r(), light.color.g(), light.color.b()]];
            let mut v_lightpos = vec![[center.x, center.y, fixed_z]];
            let mut v_lightpower = vec![light.reach];
            let mut indices = vec![0, 1, 2];

            let vis_polygon = get_visibility_polygon(&level_geo, center);
            let mut point_index = 1;
            for point in vis_polygon.exterior_coords_iter()
            {
                let vecpoint = Vec2::new(point.x as f32, point.y as f32);

                v_pos.push([vecpoint.x, vecpoint.y, fixed_z]);
                v_color.push([light.color.r(), light.color.g(), light.color.b()]);
                v_lightpos.push([center.x, center.y, fixed_z]);
                v_lightpower.push(light.reach);

                if point_index != 1 {
                    indices.push(0);
                    
                    indices.push(point_index);
                    indices.push(point_index - 1);
                }

                point_index += 1;
            }

            if let Some(mesh) = meshes.get_mut(mesh_handle.id) {
                mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
                mesh.set_attribute("light_Color", v_color);
                mesh.set_attribute("light_Position", v_lightpos);
                mesh.set_attribute("light_Power", v_lightpower);
                mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

                //println!("Did mesh stuff");

                light.mesh_built = true;
            }
        }
    }
    //let mut v_pos: Vec<[f32;3]> = vec![];
    //let mut indices = vec![];
    
    //light_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
}

pub fn light_setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
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

    let pipeline_handle = pipelines.add(pipeline);


    let mut light_mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    let v_pos = vec![[0.0, 0.0, 0.0], [200.0, 0.0, 0.0], [0.0, 200.0, 0.0]];
    //let v_color = vec![[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0]];
    let indices = vec![0, 1, 2];

    
    light_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    //light_mesh.set_attribute("Vertex_Color", v_color);
    light_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    let mesh = meshes.add(light_mesh);
    
    // We can now spawn the entities for the star and the camera
    commands.spawn_bundle(MeshBundle {
        mesh: mesh,
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_xyz(150.0, 0.0, 0.1),
        visible: Visible { is_transparent: true, is_visible: true },
        ..Default::default()
    })
    .insert(PointLight::new(Color::BLUE, 500.0));
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
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
void main() {
    l_Color = light_Color;
    l_Position = light_Position;
    l_power = light_Power;
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
layout(location = 0) out vec4 o_Target;
void main() {
    float distance = length(l_Position - position);
    float light_power = clamp(1 - (distance / l_power), 0.0, 1.0);
    light_power = pow(light_power, 3);
    o_Target = vec4(l_Color.x, l_Color.y, l_Color.z, light_power);
}
";