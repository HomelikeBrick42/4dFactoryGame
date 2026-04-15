use math::{NoE2Rotor, Rotor, Vector2, Vector3, Vector4};
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
        let forward = self.base_rotation.x();
        let up = self.base_rotation.y();
        let right = self.base_rotation.z();
        let ana = self.base_rotation.w();

        let move_speed = 4.0;
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

        if input_state.key_released(KeyCode::ShiftLeft) {
            if input_state.key_pressed(KeyCode::ArrowUp) {
                self.xy_rotation += rotation_speed * dt;
            }
            if input_state.key_pressed(KeyCode::ArrowDown) {
                self.xy_rotation -= rotation_speed * dt;
            }
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
            if input_state.key_pressed(KeyCode::ArrowUp) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(rotation_speed * dt));
            }
            if input_state.key_pressed(KeyCode::ArrowDown) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(-rotation_speed * dt));
            }
            if input_state.key_pressed(KeyCode::ArrowLeft) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_zw(-rotation_speed * dt));
            }
            if input_state.key_pressed(KeyCode::ArrowRight) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_zw(rotation_speed * dt));
            }
        }
    }

    pub fn rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation).then(Rotor::rotate_xy(self.xy_rotation))
    }

    pub fn get_ray(&self, mouse_position: Vector2<f32>, width: u32, height: u32) -> Ray {
        let aspect = width as f32 / height as f32;
        let mut uv = Vector2 {
            x: (mouse_position.x + 0.5) / width as f32,
            y: (mouse_position.y + 0.5) / height as f32,
        } * 2.0
            - 1.0;
        uv *= f32::tan(self.fov * 0.5);
        uv.x *= aspect;
        uv.y *= -1.0;

        let rotation = self.rotation();
        Ray {
            origin: self.position,
            direction: (rotation.x() + rotation.y() * uv.y + rotation.z() * uv.x).normalised(),
        }
    }

    pub fn to_gpu(&self, hovered_tile: Option<Vector3<i32>>) -> ray_tracing::Camera {
        let rotation = self.rotation();
        ray_tracing::Camera {
            position: self.position,
            forward: rotation.x(),
            up: rotation.y(),
            right: rotation.z(),
            hovered_tile,
            fov: self.fov,
        }
    }
}

pub struct Ray {
    pub origin: Vector4<f32>,
    pub direction: Vector4<f32>,
}
