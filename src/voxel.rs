use bytemuck::{Pod, Zeroable};
use derive_new::new;
use palette::{LinSrgb, Srgb};

#[derive(new, Copy, Clone, Debug, PartialEq, Default)]
pub struct VoxelType {
    pub color: Srgb,
}

impl VoxelType {
    pub fn to_internal(self) -> VoxelTypeInternal {
        VoxelTypeInternal {
            color: self.color.into_linear(),
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelTypeInternal {
    color: LinSrgb,
    _padding: f32,
}

impl Default for VoxelTypeInternal {
    fn default() -> Self {
        VoxelTypeInternal {
            color: LinSrgb::new(0.0, 0.0, 0.0),
            _padding: 0.0,
        }
    }
}

unsafe impl Zeroable for VoxelTypeInternal {}
unsafe impl Pod for VoxelTypeInternal {}

#[repr(transparent)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxelId(pub u8);
