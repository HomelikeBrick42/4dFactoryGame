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
    pub ground_view: bool,
    pub ground_view_percentage: f32,
    pub screen_door_enabled: bool,
    pub screen_door_percentage: f32,
}

impl Camera {
    pub fn update(&mut self, input_state: &InputState, dt: f32) {
        self.ground_view ^= input_state.key_just_pressed(KeyCode::KeyG);
        if self.ground_view {
            self.ground_view_percentage += dt;
        } else {
            self.ground_view_percentage -= dt;
        }
        self.ground_view_percentage = self.ground_view_percentage.clamp(0.0, 1.0);
        if self.ground_view_percentage == 1.0 {
            self.xy_rotation = 0.0;
        }

        let screen_door_speed = 0.2;
        self.screen_door_enabled ^= input_state.key_just_pressed(KeyCode::KeyT);
        if input_state.key_pressed(KeyCode::KeyY) {
            self.screen_door_percentage += screen_door_speed * dt;
        }
        if input_state.key_pressed(KeyCode::KeyH) {
            self.screen_door_percentage -= screen_door_speed * dt;
        }
        self.screen_door_percentage = self.screen_door_percentage.clamp(0.0, 1.0);

        let rotation = self.no_xy_rotation();
        let forward = rotation.x();
        let up = rotation.y();
        let right = rotation.z();
        let ana = rotation.w();

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

        self.base_rotation = self.base_rotation.normalised();
        self.xy_rotation %= TAU;
    }

    pub fn no_xy_rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation)
            .then(Rotor::rotate_yw(0.25 * TAU * self.ground_view_percentage))
    }

    pub fn rotation(&self) -> Rotor {
        self.no_xy_rotation().then(Rotor::rotate_xy(
            self.xy_rotation * (1.0 - self.ground_view_percentage),
        ))
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
            ana: rotation.w(),
            hovered_tile,
            fov: self.fov,
            screen_door_percentage: self.screen_door_percentage,
        }
    }
}

pub struct Ray {
    pub origin: Vector4<f32>,
    pub direction: Vector4<f32>,
}
