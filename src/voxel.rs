use std::num::NonZeroU32;
use wgpu::ImageDataLayout;
use palette::Srgba;
use arrayvec::ArrayVec;
use ndarray::Array3;

pub struct WorldSize(pub u32);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VoxelType {
    color: Srgba,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxelId(u8);

pub struct World3d {
    voxels: Array3<VoxelId>,
    palette: ArrayVec<VoxelType, 256>,
}

impl World3d {
    pub fn new(size: u32) -> World3d {
        let mut palette = ArrayVec::new();
        let size = size as usize;
        palette.push(VoxelType {
            color: Srgba::new(0.0, 0.0, 0.0, 0.0),
        });
        World3d {
            voxels: Array3::from_elem((size, size, size), VoxelId(0)),
            palette,
        }
    }

    pub fn insert_type(&mut self, ty: VoxelType) -> VoxelId {
        let id = self.palette.len();
        self.palette.push(ty);
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
        unsafe { std::mem::transmute(self.voxels.as_slice().unwrap()) }
    }

    pub fn palette_bytes(&self) -> &[u8] {
        &[0]
    }
}
