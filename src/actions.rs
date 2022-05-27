use crate::GameState;
use bevy::prelude::*;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>().add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(set_actions),
        );
    }
}

#[derive(Default)]
pub struct Actions {
    pub moving: Vec2,
    pub jumping: bool,
    pub running: bool,
    pub crouching: bool,
}

fn set_actions(mut actions: ResMut<Actions>, keyboard_input: Res<Input<KeyCode>>) {
    let mut velocity = Vec2::ZERO;
    actions.jumping = false;
    actions.running = false;
    actions.crouching = false;
    for key in keyboard_input.get_pressed() {
        match key {
            KeyCode::W => velocity[1] += 1.,
            KeyCode::S => velocity[1] -= 1.,
            KeyCode::A => velocity[0] -= 1.,
            KeyCode::D => velocity[0] += 1.,
            KeyCode::Space => actions.jumping = true,
            KeyCode::LShift => actions.running = true,
            KeyCode::LControl => actions.crouching = true,
            _ => (),
        }
    }
    actions.moving = velocity.normalize_or_zero();

}