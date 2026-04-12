use math::{NoE2Rotor, Rotor, Vector4};
use renderer::{
    app::{InputState, KeyCode},
    ray_tracing,
};
use std::f32::consts::TAU;

pub struct Camera {
    pub position: Vector4<f32>,
    pub base_rotation: NoE2Rotor,
    pub xy_rotation: f32,
    pub fov: f32,
}

impl Camera {
    pub fn update(&mut self, input_state: &InputState, dt: f32) {
        let rotation = self.rotation();
        let forward = rotation.x();
        let up = rotation.y();
        let right = rotation.z();
        let ana = rotation.w();

        let move_speed = 2.0;
        let rotation_speed = TAU * 0.25;

        if input_state.key_pressed(KeyCode::KeyW) {
            self.position += forward * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyS) {
            self.position -= forward * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyA) {
            self.position -= right * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyD) {
            self.position += right * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyE) {
            self.position += up * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyQ) {
            self.position -= up * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyR) {
            self.position += ana * move_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyF) {
            self.position -= ana * move_speed * dt;
        }

        if input_state.key_pressed(KeyCode::ArrowUp) {
            self.xy_rotation += rotation_speed * dt;
        }
        if input_state.key_pressed(KeyCode::ArrowDown) {
            self.xy_rotation -= rotation_speed * dt;
        }
        if input_state.key_released(KeyCode::ShiftLeft) {
            if input_state.key_pressed(KeyCode::ArrowLeft) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xz(-rotation_speed * dt));
            }
            if input_state.key_pressed(KeyCode::ArrowRight) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xz(rotation_speed * dt));
            }
        } else {
            if input_state.key_pressed(KeyCode::ArrowLeft) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(-rotation_speed * dt));
            }
            if input_state.key_pressed(KeyCode::ArrowRight) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(rotation_speed * dt));
            }
        }
    }

    pub fn rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation).then(Rotor::rotate_xy(self.xy_rotation))
    }

    pub fn to_gpu(&self) -> ray_tracing::Camera {
        let rotation = self.rotation();
        ray_tracing::Camera {
            position: self.position,
            forward: rotation.x(),
            up: rotation.y(),
            right: rotation.z(),
            fov: self.fov,
        }
    }
}
