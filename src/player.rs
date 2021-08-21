use bevy::{
    prelude::*, 
};

pub struct PlayerMovement {
    pub speed: f32,
}

pub fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&PlayerMovement, &mut Transform)>
) {
    if let Ok((player, mut transform)) = query.single_mut() {
        let mut y_movement: f32 = 0.0;
        let mut x_movement: f32 = 0.0; 
        if keyboard_input.pressed(KeyCode::W) {
            y_movement += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            y_movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            x_movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            x_movement += 1.0;
        }

        let translation = &mut transform.translation;
        translation.x += time.delta_seconds() * x_movement * player.speed;
        translation.y += time.delta_seconds() * y_movement * player.speed;

        translation.x = translation.x.min(380.0).max(-380.0);
        translation.y = translation.y.min(380.0).max(-380.0);
    }
}