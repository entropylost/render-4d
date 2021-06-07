use bytemuck::Pod;
use bytemuck::Zeroable;
use derive_new::new;
use palette::LinSrgba;
use palette::Srgba;

#[derive(new, Copy, Clone, Debug, PartialEq, Default)]
pub struct VoxelType {
    pub color: Srgba,
}

impl VoxelType {
    pub fn to_internal(self) -> VoxelTypeInternal {
        VoxelTypeInternal {
            color: self.color.into_linear(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct VoxelTypeInternal {
    color: LinSrgba,
}

unsafe impl Zeroable for VoxelTypeInternal {}
unsafe impl Pod for VoxelTypeInternal {}

#[repr(transparent)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxelId(pub u8);
