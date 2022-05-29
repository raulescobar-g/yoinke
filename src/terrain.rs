use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use noise::{NoiseFn, Perlin, Seedable};
use crate::GameState;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Debug)]
struct PlanetOptions {
    #[inspectable(min = 1.0, max = 1000.0)]
    radius: f32,
    resolution: u32,
    seed: u32,
    pos: Vec3,
    strength: f32,
    roughness: f32,
    centre: Vec3,
}

impl Default for PlanetOptions {
    fn default() -> Self {
        PlanetOptions {
            radius: 1.0,
            resolution: 10,
            seed: 1,
            pos: Vec3::new(0.0,0.0,0.0),
            strength: 1.0,
            roughness: 1.0,
            centre: Vec3::new(0.0,0.0,0.0),
        }
    }
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(InspectorPlugin::<PlanetOptions>::new())
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_planet)
                .with_system(spawn_light)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(planet_respawn)
        );
    }
}

fn planet_respawn(data: Res<PlanetOptions>, mut command: Commands, mut planet_entities: Query<(Entity,&Planet)>,mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !data.is_changed() {
        return;
    } else {
        for (entity,_) in planet_entities.iter_mut() {
            command.entity(entity).despawn();
        }

        for face in create_planet(data.resolution, data.strength, data.roughness, data.seed, data.centre){
            command.spawn_bundle(PbrBundle {
                mesh: meshes.add(face),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.0,
                    ..default()
                }),
                transform: Transform::from_xyz(data.pos.x, data.pos.y, data.pos.z).with_scale(Vec3::new(data.radius, data.radius, data.radius)) ,
                ..default()
            }).insert(Planet);
        };

    }
}

#[derive(Component)]
struct Planet;


fn spawn_planet(mut command: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, data: Res<PlanetOptions>) {

    for face in create_planet(data.resolution, data.strength, data.roughness, data.seed, data.centre){
        command.spawn_bundle(PbrBundle {
            mesh: meshes.add(face),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.0,
                ..default()
            }),
            transform: Transform::from_xyz(data.pos.x, data.pos.y, data.pos.z).with_scale(Vec3::new(data.radius, data.radius, data.radius)) ,
            ..default()
        }).insert(Planet);
    };
}


fn spawn_light(mut command: Commands) {
    command.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        point_light: PointLight {
            intensity: 200000.0, // lumens - roughly a 100W non-halogen incandescent bulb
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        ..default()
    });
    command.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, -20.0, 0.0),
        point_light: PointLight {
            intensity: 200000.0, // lumens - roughly a 100W non-halogen incandescent bulb
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        ..default()
    });
}

struct TerrainFace {
    local_up: Vec3,
    axis_a: Vec3,
    axis_b: Vec3,
}

impl TerrainFace {
    pub fn allocate(local_up: Vec3) -> TerrainFace {
        let axis_a = Vec3::new(local_up.y, local_up.z , local_up.x);
        let axis_b = local_up.cross(axis_a);
        TerrainFace {
            local_up,
            axis_a,
            axis_b
        }
    }
}

fn generate_mesh(face: TerrainFace, resolution: u32, strength: f32, roughness: f32, seed: u32, centre: Vec3) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let noise = Perlin::new();
    let noise = noise.set_seed(seed);

    let vert_count = resolution * resolution;
    let tile_count = (resolution-1) * (resolution-1) * 6;

    let mut vertices: Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut normals:Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut textures: Vec<[f32;2]> = Vec::with_capacity(vert_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(tile_count as usize);

    for z in 0..resolution {
        for x in 0..resolution {
            let i = z * resolution + x;
            let x_scaled = ((x as f32 /(resolution-1) as f32) - 0.5) * 2.0;
            let z_scaled = ((z as f32 /(resolution-1) as f32) - 0.5) * 2.0;

            let mut pos = (((face.local_up + face.axis_a * x_scaled + face.axis_b * z_scaled).normalize() * roughness) + centre);

            let h = 1.0 + (1.0 + noise.get(pos.as_dvec3().to_array()) * 0.5 * strength as f64) as f32;

            pos[0] *= h;
            pos[1] *= h;
            pos[2] *= h;

            vertices.push(pos.to_array());
            textures.push([pos[0], pos[1]]);
            normals.push(pos.to_array());

            if x != resolution-1 && z != resolution-1 {
                indices.push(i);
                indices.push(i + resolution + 1);
                indices.push(i + resolution);

                indices.push(i);
                indices.push(i+1);
                indices.push(i + resolution + 1);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, textures);
    mesh.set_indices(Some(Indices::U32( indices )));

    return mesh;
}

fn create_planet(res: u32, strength: f32, roughness: f32, seed: u32, centre: Vec3) -> Vec<Mesh> {
   
    let mut meshes: Vec<Mesh> = Vec::with_capacity(6);

    let dirs = vec![Vec3::new(1.0,0.0,0.0), Vec3::new(-1.0,0.0,0.0), Vec3::new(0.0,1.0,0.0), Vec3::new(0.0,-1.0,0.0), Vec3::new(0.0,0.0,1.0), Vec3::new(0.0,0.0,-1.0)];
    for face in dirs.iter() {
        let terrain_face = TerrainFace::allocate(face.clone());
        meshes.push(generate_mesh(terrain_face, res, strength, roughness, seed, centre));
    }
    meshes
}
