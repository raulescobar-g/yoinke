//use crate::actions::Actions;
use bevy::input::mouse::MouseMotion;
use bevy::ecs::event::{Events, ManualEventReader};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct Collides;

#[derive(Component)]
struct Physics {
    mass : f32
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<InputState>()
        .init_resource::<MovementSettings>()
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_player)
                .with_system(spawn_camera)
                .with_system(initial_grab_cursor)
                
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player)
                .with_system(player_look)
                .with_system(cursor_grab)
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(Camera);
}

fn spawn_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere{
                radius: 0.5,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                emissive: Color::rgba_linear(0.0, 0.0, 100.0, 0.0),
                ..default()
            }),
            transform:Transform::from_xyz(0., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Player)
        .insert(Collides)
        .insert(Physics{
            mass: 1.0f32
        });
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query_player: Query<&mut Transform, With<Player>>,
    mut query_camera: Query<&mut Transform, (With<Camera>,Without<Player>)>
) {
    let window = windows.get_primary().unwrap();
    let mut player_transform = query_player.single_mut();
    let mut camera_transform = query_camera.single_mut();
    
    let mut velocity = Vec3::ZERO;
    let local_z = player_transform.local_z();

    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);

    for key in keys.get_pressed() {
        if window.cursor_locked() {
            match key {
                KeyCode::W => velocity += forward,
                KeyCode::S => velocity -= forward,
                KeyCode::A => velocity -= right,
                KeyCode::D => velocity += right,
                KeyCode::Space => velocity += Vec3::Y,
                KeyCode::LShift => velocity -= Vec3::Y,
                _ => (),
            }
        }
    }

    velocity = velocity.normalize_or_zero();

    player_transform.translation += velocity * time.delta_seconds() * settings.speed;
    camera_transform.translation = player_transform.translation;
    
}

fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query_player: Query<&mut Transform, With<Player>>,
    mut query_camera: Query<&mut Transform, (With<Camera>,Without<Player>)>
) {
    let window = windows.get_primary().unwrap();
    let mut delta_state = state.as_mut();
    let mut camera_transform = query_camera.single_mut();
    let mut player_transform = query_player.single_mut();

    for ev in delta_state.reader_motion.iter(&motion) {
        if window.cursor_locked() {
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());
            delta_state.pitch -=
                (settings.sensitivity * ev.delta.y * window_scale).to_radians();
            delta_state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
        }

        delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

        // Order is important to prevent unintended roll
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
            * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
        
        camera_transform.rotation = player_transform.rotation;
    }
    
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00048,
            speed: 15.,
        }
    }
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}
/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }
}