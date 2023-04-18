use gltf::texture::{MagFilter, MinFilter, WrappingMode};

pub struct Sampler {
    pub mag_filter: Option<MagFilter>,
    pub min_filter: Option<MinFilter>,
    pub wrap_s: WrappingMode,
    pub wrap_t: WrappingMode,
}

impl From<gltf::texture::Sampler<'_>> for Sampler {
    fn from(sampler: gltf::texture::Sampler) -> Self {
        Self {
            mag_filter: sampler.mag_filter(),
            min_filter: sampler.min_filter(),
            wrap_s: sampler.wrap_s(),
            wrap_t: sampler.wrap_t(),
        }
    }
}

pub struct Texture {
    pub name: Option<String>,
    pub source_index: usize,
    pub sampler: Sampler,
}

impl Texture {
    pub fn new(name: Option<String>, source_index: usize, sampler: Sampler) -> Self {
        Self {
            name,
            source_index,
            sampler,
        }
    }
}
