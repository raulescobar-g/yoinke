use std::cell::RefCell;
use std::rc::Rc;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_inspector_egui::egui::epaint::textures;
use noise::{NoiseFn, Perlin, Seedable};
use crate::GameState;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Inspectable, Debug)]
struct PlanetOptions {
    #[inspectable(min = 1.0, max = 1000.0)]
    radius: f32,
    layers: u32,
    persistence: f32,
    base_roughness: f32,
    resolution: u32,
    seed: u32,
    pos: Vec3,
    strength: f32,
    roughness: f32,
    centre: Vec3,
    minimum: f32,
}

impl Default for PlanetOptions {
    fn default() -> Self {
        PlanetOptions {
            radius: 1.0,
            layers: 1,
            persistence: 0.5,
            base_roughness: 1.0,
            resolution: 10,
            seed: 1,
            pos: Vec3::new(0.0,0.0,0.0),
            strength: 1.0,
            roughness: 2.0,
            centre: Vec3::new(0.0,0.0,0.0),
            minimum: 0.0,
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
                .with_system(spawn_ball)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(planet_respawn)
                .with_system(gravity)
        );
    }
}

fn planet_respawn(data: Res<PlanetOptions>, mut command: Commands, mut planet_entities: Query<(Entity,&Planet)>, meshes: ResMut<Assets<Mesh>>, materials: ResMut<Assets<StandardMaterial>>) {
    if data.is_changed() {
        for (entity,_) in planet_entities.iter_mut() {
            command.entity(entity).despawn();
        }

        spawn_planet(command, meshes, materials, data);
    }
}

#[derive(Component)]
struct Planet;


fn spawn_planet(mut command: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, data: Res<PlanetOptions>) {

    let planet_mesh = create_planet(&data);

    let scale = Vec3::new(data.radius, data.radius, data.radius);
    let transform = Transform::from_xyz(data.pos.x, data.pos.y, data.pos.z).with_scale(scale);
    let collider = Collider::from_bevy_mesh(&planet_mesh, &ComputedColliderShape::TriMesh).unwrap();

    command.spawn_bundle(PbrBundle {
        mesh: meshes.add(planet_mesh ),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.0,
            ..default()
        }),
        transform: transform,
        ..default()
    }).insert(Planet)
    .insert(collider)
    .insert(ExternalForce {
        force: Vec3::new(0.0, 0.0, 0.0),
        torque: Vec3::new(0.0, 0.0, 0.0),
    })
    .insert(ColliderMassProperties::Density(1.0))
    .insert(RigidBody::Dynamic)
    .insert(GravityScale(0.0));
    
}

fn gravity(mut body_query: Query<(&mut ExternalForce, & MassProperties, & Transform)>) {

    let mut net_force = Vec3::ZERO;

    let mut iter = body_query.iter_mut();

    while let Some((mut force, body , pos)) = iter.next() {

        for (_ ,ext_body, ext_pos) in iter.by_ref() {

            if pos.translation != ext_pos.translation {
                let r = ext_pos.translation - pos.translation;
                net_force += r.normalize() * ((body.mass * ext_body.mass) / r.length_squared());
            }
        }
        force.force = net_force;
    } 
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

fn spawn_ball(mut command : Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials : ResMut<Assets<StandardMaterial>>) {

    let sphere_mesh = Mesh::from(shape::Icosphere {
        radius: 0.45,
        subdivisions: 10,
    });

    let transform = Transform::from_xyz(3.0, 3.0, 3.0);

    let mut collider = Collider::from_bevy_mesh(&sphere_mesh, &ComputedColliderShape::TriMesh).unwrap();
    collider.set_scale(Vec3::new(0.45,0.45,0.45), 32);

    command.spawn_bundle(PbrBundle {
        mesh: meshes.add(sphere_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("ffd891").unwrap(),
            // vary key PBR parameters on a grid of spheres to show the effect
            metallic: 0.5,
            perceptual_roughness: 0.5,
            ..default()
        }),
        transform: transform,
        ..default()
    }).insert(collider)
    .insert(ExternalForce {
        force: Vec3::new(0.0, 0.0, 0.0),
        torque: Vec3::new(0.0, 0.0, 0.0),
    })
    .insert(ColliderMassProperties::Density(1.0))
    .insert(RigidBody::Dynamic)
    .insert(GravityScale(0.0));
}

struct TerrainFace {
    local_up: Vec3,
    axis_a: Vec3,
    axis_b: Vec3,
    order: u8,
}

impl TerrainFace {
    pub fn allocate(local_up: Vec3, order: u8) -> TerrainFace {
        let axis_a = Vec3::new(local_up.y, local_up.z , local_up.x);
        let axis_b = local_up.cross(axis_a);
        TerrainFace {
            local_up,
            axis_a,
            axis_b,
            order,
        }
    }
}

struct MeshData {
    vertices: Vec<[f32;3]>,
    normals : Vec<[f32;3]>,
    textures : Vec<[f32;2]>,
    indices : Vec<u32>,
}

fn generate_mesh(face: TerrainFace, data: &Res<PlanetOptions>) -> MeshData {
    let noise = Perlin::new();
    let noise = noise.set_seed(data.seed);

    let vert_count = data.resolution * data.resolution;
    let tile_count = (data.resolution-1) * (data.resolution-1) * 6;

    let mut vertices: Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut normals:Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut textures: Vec<[f32;2]> = Vec::with_capacity(vert_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(tile_count as usize);

    for z in 0..data.resolution {
        for x in 0..data.resolution {
            let i = z * data.resolution + x;
            let x_scaled = ((x as f32 /(data.resolution-1) as f32) - 0.5) * 2.0;
            let z_scaled = ((z as f32 /(data.resolution-1) as f32) - 0.5) * 2.0;

            let mut h: f32 = 0.0;
            let mut frequency = data.base_roughness;
            let mut amplitude = 1.0;

            let mut position = (face.local_up + face.axis_a * x_scaled + face.axis_b * z_scaled).normalize();

            for _ in 0..data.layers {
                let pos = ((face.local_up + face.axis_a * x_scaled + face.axis_b * z_scaled).normalize() * frequency) + data.centre;
                let v = noise.get(pos.as_dvec3().to_array()) as f32;
                h += (v+1.0) * 0.5 * amplitude;
                frequency *= data.roughness;
                amplitude *= data.persistence;
            }

            position *= 1. + ((h - data.minimum).min(0.) * data.strength);

            vertices.push(position.to_array());
            textures.push([position[0], position[1]]);
            normals.push(position.to_array());

            if x != data.resolution-1 && z != data.resolution-1 {
                indices.push(face.order as u32 * vert_count + i);
                indices.push(face.order as u32 * vert_count + i + data.resolution + 1);
                indices.push(face.order as u32 * vert_count + i + data.resolution);

                indices.push(face.order as u32 * vert_count + i);
                indices.push(face.order as u32 * vert_count + i+1);
                indices.push(face.order as u32 * vert_count + i + data.resolution + 1);
            }
        }
    }

    MeshData {
        vertices,
        normals,
        textures,
        indices,
    }
}

fn create_planet(data: &Res<PlanetOptions>) -> Mesh {
   
    let mut meshes: Mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vert_count = data.resolution * data.resolution * 6;
    let tile_count = (data.resolution-1) * (data.resolution-1) * 6 * 6;

    let mut vertices: Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut normals:Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut textures: Vec<[f32;2]> = Vec::with_capacity(vert_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(tile_count as usize);

    let dirs = vec![Vec3::new(1.0,0.0,0.0),
                            Vec3::new(0.0,1.0,0.0), 
                            Vec3::new(0.0,0.0,1.0), 
                            Vec3::new(0.0,0.0,-1.0),
                            Vec3::new(0.0,-1.0,0.0),
                            Vec3::new(-1.0,0.0,0.0)];
    let mut order: u8 = 0;
    for face in dirs.iter() {
        let terrain_face = TerrainFace::allocate(*face, order);
        let mut mesh_data = generate_mesh(terrain_face, data);

        vertices.append(&mut mesh_data.vertices);
        normals.append(&mut mesh_data.normals);
        textures.append(&mut mesh_data.textures);
        indices.append(&mut mesh_data.indices);

        order += 1;
    }

    meshes.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    meshes.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    meshes.insert_attribute(Mesh::ATTRIBUTE_UV_0, textures);
    meshes.set_indices(Some(Indices::U32( indices )));

    meshes
}
