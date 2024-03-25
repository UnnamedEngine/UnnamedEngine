pub mod rendering;

use wgpu::Color;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_SIZE;
pub const PALETTE_SIZE: usize = 16;

// Access
// (X * AREA) + (y * SIZE) + z

/// Used to access the palette inside of a chunk
pub type VoxelIndex = u8;

/// A `Chunk` contains data
#[repr(C)]
pub struct Chunk {
  pub palette: [Color; PALETTE_SIZE],
  pub voxels: [VoxelIndex; CHUNK_VOLUME],
}

impl Chunk {
  pub fn new(
    palette: [Color; PALETTE_SIZE],
    voxels: [VoxelIndex; CHUNK_VOLUME],
  ) -> Self {
    Self {
      palette,
      voxels,
    }
  }

  pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: VoxelIndex) {
    self.voxels[(x as usize * CHUNK_AREA) + (y as usize * CHUNK_SIZE) + z as usize] = voxel;
  }

  pub fn get(&self, x: u32, y: u32, z: u32) -> VoxelIndex {
    self.voxels[(x as usize * CHUNK_AREA) + (y as usize * CHUNK_SIZE) + z as usize]
  }

  pub fn iter(&self) -> ChunkIterator {
    ChunkIterator {
      chunk: self,
      current: Default::default(),
    }
  }
}

impl Default for Chunk {
  fn default() -> Self {
    Self {
      palette: Default::default(),
      voxels: [0; CHUNK_VOLUME],
    }
  }
}

/// Represents a voxel position inside of a chunk
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkVoxelPosition {
  pub x: usize,
  pub y: usize,
  pub z: usize,
}

impl ChunkVoxelPosition {
  /// Returns the index that represents the 3d position inside of a 1d array
  pub fn get_1d(&self) -> usize {
    (self.x * CHUNK_AREA) + (self.y * CHUNK_SIZE) + self.z
  }
}

/// Iterator for a chunk, it returns a `(ChunkVoxelPosition, VoxelIndex)` tuple
/// to represent the voxel, the `VoxelIndex` being an index to the current chunk
/// palette
#[repr(C)]
pub struct ChunkIterator<'a> {
  chunk: &'a Chunk,
  current: ChunkVoxelPosition,
}

impl<'a> Iterator for ChunkIterator<'a> {
  type Item = (ChunkVoxelPosition, VoxelIndex);

  fn next(&mut self) -> Option<Self::Item> {
    if self.current.x >= CHUNK_SIZE {
      return None;
    }

    let position = self.current;

    let value = self.chunk.voxels[self.current.get_1d()];

    self.current.y += 1;
    if self.current.y >= CHUNK_SIZE {
      self.current.y = 0;
      self.current.z += 1;
      if self.current.z >= CHUNK_SIZE {
        self.current.z = 0;
        self.current.x += 1;
      }
    }

    Some((position, value))
  }
}
