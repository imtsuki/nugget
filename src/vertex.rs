pub enum VertexAttribute {
    Position,
    TexCoord,
    Normal,
    Tangent,
}

impl VertexAttribute {
    pub const fn format(&self) -> wgpu::VertexFormat {
        use wgpu::VertexFormat::*;
        match self {
            VertexAttribute::Position => Float32x3,
            VertexAttribute::TexCoord => Float32x2,
            VertexAttribute::Normal => Float32x3,
            VertexAttribute::Tangent => Float32x4,
        }
    }

    pub const fn size(&self) -> wgpu::BufferAddress {
        self.format().size()
    }

    /// Corresponds to the `@location(n)` in the vertex shader
    pub const fn location(&self) -> wgpu::ShaderLocation {
        match self {
            VertexAttribute::Position => 0,
            VertexAttribute::TexCoord => 1,
            VertexAttribute::Normal => 2,
            VertexAttribute::Tangent => 3,
        }
    }
}

type Position = [f32; 3];
type TexCoord = [f32; 2];
type Normal = [f32; 3];
type Tangent = [f32; 4];

#[repr(C)]
pub struct VertexIn {
    position: Position,
    tex_coord: TexCoord,
    normal: Normal,
    tangent: Tangent,
}

impl VertexIn {
    /// Use separate buffers for each attribute for now
    pub const BUFFER_LAYOUTS: [wgpu::VertexBufferLayout<'static>; 4] = [
        wgpu::VertexBufferLayout {
            array_stride: VertexAttribute::Position.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![VertexAttribute::Position.location() => Float32x3],
        },
        wgpu::VertexBufferLayout {
            array_stride: VertexAttribute::TexCoord.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![VertexAttribute::TexCoord.location() => Float32x2],
        },
        wgpu::VertexBufferLayout {
            array_stride: VertexAttribute::Normal.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![VertexAttribute::Normal.location() => Float32x3],
        },
        wgpu::VertexBufferLayout {
            array_stride: VertexAttribute::Tangent.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![VertexAttribute::Tangent.location() => Float32x4],
        },
    ];
}
