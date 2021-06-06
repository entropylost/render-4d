use arrayvec::ArrayVec;
use bytemuck::Pod;
use bytemuck::Zeroable;
use derive_new::new;
use nalgebra::Vector3;
use ndarray::Array3;
use palette::LinSrgba;
use palette::Srgba;
use std::num::NonZeroU32;
use std::ops::Index;
use std::ops::IndexMut;
use wgpu::ImageDataLayout;

pub struct WorldSize(pub u32);

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
pub struct VoxelId(u8);

pub struct World3d {
    voxels: Array3<VoxelId>,
    types: ArrayVec<VoxelType, 256>,
    types_internal: ArrayVec<VoxelTypeInternal, 256>,
}

impl World3d {
    pub fn new(size: u32) -> World3d {
        let size = size as usize;

        let mut types = ArrayVec::new();
        let mut types_internal = ArrayVec::new();
        types.push(VoxelType::default());
        types_internal.push(types[0].to_internal());
        World3d {
            voxels: Array3::from_elem((size, size, size), VoxelId(0)),
            types,
            types_internal,
        }
    }

    pub fn insert_type(&mut self, ty: VoxelType) -> VoxelId {
        let id = self.types.len();
        self.types.push(ty);
        self.types_internal.push(ty.to_internal());
        VoxelId(id as u8)
    }

    pub fn air(&mut self) -> VoxelId {
        VoxelId(0)
    }

    pub fn size(&self) -> u32 {
        self.voxels.shape()[0] as u32
    }

    pub fn layout(&self) -> ImageDataLayout {
        let size = self.size();
        ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(size),
            rows_per_image: NonZeroU32::new(size),
        }
    }

    pub fn voxel_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.voxels.as_slice().unwrap())
    }

    pub fn types_internal(&self) -> [VoxelTypeInternal; 256] {
        let mut out = [Default::default(); 256];
        out[..self.types.len()].clone_from_slice(&self.types_internal);
        out
    }
}

impl Index<Vector3<u32>> for World3d {
    type Output = VoxelId;
    fn index(&self, index: Vector3<u32>) -> &Self::Output {
        let index = index.cast::<usize>();
        &self.voxels[[index.x, index.y, index.z]]
    }
}
impl IndexMut<Vector3<u32>> for World3d {
    fn index_mut(&mut self, index: Vector3<u32>) -> &mut Self::Output {
        let index = index.cast::<usize>();
        &mut self.voxels[[index.x, index.y, index.z]]
    }
}
