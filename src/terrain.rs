use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use noise::{NoiseFn, Perlin, Seedable};
use crate::GameState;


pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_terrain)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                // .with_system(dynamic_load_terrain)
        );
    }
}

// fn dynamic_load_terrain(mut commands: Commands, player_query : Query<&Transform, With<Player>>) {
//     let player = player_query.single();

// }


fn spawn_terrain(mut command: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let size_of_chunk = 30.0;
    let render_dist = 3;
    let var = 0.2;
    let seed = 10;
    let resolution = 100;

    for x_chunk in -render_dist..(render_dist-1) {
        for z_chunk in -render_dist..(render_dist-1) {
            let x_loc = size_of_chunk * (x_chunk as f32);
            let z_loc = size_of_chunk * (z_chunk as f32);

            command.spawn_bundle(PbrBundle {
                mesh: meshes.add(create_floor(resolution, size_of_chunk, Vec2::new(x_loc,z_loc), seed, var)),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 1.0,
                    ..default()
                }),
                transform: Transform::from_xyz(1.5, 1.0, 1.5),
                ..default()
            });
        }
    }
    command.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        point_light: PointLight {
            intensity: 2000.0, // lumens - roughly a 100W non-halogen incandescent bulb
            color: Color::BLUE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

fn create_floor(res : u32, size : f32, pos: Vec2, seed: u32, var: f64) -> Mesh {
    let scalar: f32 = size / (res-1) as f32;

    let vert_count = res * res;
    let tile_count = (res-1) * (res-1);

    let noise = Perlin::new();
    let noise = noise.set_seed(seed);


    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertices: Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut normals:Vec<[f32;3]> = Vec::with_capacity(vert_count as usize);
    let mut textures: Vec<[f32;2]> = Vec::with_capacity(vert_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity((tile_count * 6) as usize);

    let mut noise_mem: Vec<f64> = Vec::with_capacity(vert_count as usize);
    for z in 0..res {
        for x in 0..res {
            noise_mem.push(noise.get([(pos.x as f64 + ((x as f32) * scalar) as f64) * var, (pos.y as f64 + ((z as f32) * scalar) as f64) * var]));
        }
    }


    for z in 0..res {
        for x in 0..res {
            let coord = (x + z * res) as usize;
            let x = x as f32;
            let z = z as f32;
            vertices.push([(x * scalar) + pos.x, noise_mem[coord] as f32, (z * scalar) + pos.y]);
            normals.push([0.0 ,1.0, 0.0]);
            textures.push([x * scalar,  z * scalar]); 
        }
    }

    for z in 0..(res-1) {
        for x in 0..(res-1) {
            indices.push((x+1) + ((z+1)*(res)));
            indices.push((x+1) + z*(res));
            indices.push(x + z*(res));
            
            indices.push(x + ((z+1)*(res)));
            indices.push(x + ((z+1)*(res)) + 1);
            indices.push(x + z*(res));
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, textures);
    mesh.set_indices(Some(Indices::U32( indices )));

    mesh
}