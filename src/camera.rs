use crate::uniform::{CameraBinding, Uniforms};

pub struct ArcCamera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub width: u32,
    pub height: u32,
    pub uniforms: Uniforms<CameraBinding>,
}

impl ArcCamera {
    const FOV: f32 = 45.0;
    const Z_NEAR: f32 = 0.1;
    const Z_FAR: f32 = 100.0;

    pub fn new(
        width: u32,
        height: u32,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let eye = glam::Vec3::new(2.0, 0.0, 0.0);
        let target = glam::Vec3::new(0.0, 0.0, 0.0);
        let up = glam::Vec3::new(0.0, 1.0, 0.0);

        let view_matrix = Self::calculate_view_matrix(eye, target, up);
        let projection_matrix = Self::calculate_projection_matrix(width, height);

        let uniforms = Uniforms::new(
            CameraBinding {
                view_matrix,
                projection_matrix,
            },
            device,
            layout,
        );

        Self {
            eye,
            target,
            up,
            width,
            height,
            uniforms,
        }
    }

    pub fn view_matrix(&self) -> glam::Mat4 {
        Self::calculate_view_matrix(self.eye, self.target, self.up)
    }

    fn calculate_view_matrix(eye: glam::Vec3, target: glam::Vec3, up: glam::Vec3) -> glam::Mat4 {
        glam::Mat4::look_at_lh(eye, target, up)
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        Self::calculate_projection_matrix(self.width, self.height)
    }

    fn calculate_projection_matrix(width: u32, height: u32) -> glam::Mat4 {
        glam::Mat4::perspective_lh(
            Self::FOV.to_radians(),
            width as f32 / height as f32,
            Self::Z_NEAR,
            Self::Z_FAR,
        )
    }

    fn uniforms_data(&self) -> CameraBinding {
        CameraBinding {
            view_matrix: self.view_matrix(),
            projection_matrix: self.projection_matrix(),
        }
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.width = width;
        self.height = height;
        self.uniforms.update(self.uniforms_data(), queue);
    }

    pub fn rotate(&mut self, delta: glam::Vec2, queue: &wgpu::Queue) {
        // calculate perpendicular axis to eye and up
        let axis = self.eye.cross(self.up).normalize();

        // calculate rotation from delta's x and y
        let rotation = glam::Quat::from_axis_angle(axis, delta.y * 0.01)
            * glam::Quat::from_axis_angle(self.up, delta.x * 0.01);
        // * glam::Quat::from_rotation_y(delta.x * 0.01);

        let eye = rotation * (self.eye - self.target);
        self.eye = eye + self.target;

        self.up = (rotation * self.up).normalize();
        tracing::debug!("eye: {:?}", self.eye);

        self.uniforms.update(self.uniforms_data(), queue);
    }
}
