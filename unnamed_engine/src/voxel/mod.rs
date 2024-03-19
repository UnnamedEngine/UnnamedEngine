use wgpu::Color;

pub const CHUNK_SIZE: usize = 8;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_SIZE;
pub const PALETTE_SIZE: usize = 16;

// Access
// (X * AREA) + (y * SIZE) + z

#[repr(C)]
pub struct Chunk {
  pub palette: [Color; PALETTE_SIZE],
  pub voxels: [u8; CHUNK_VOLUME],
}

impl Chunk {
  pub fn new(
    palette: [Color; PALETTE_SIZE],
    voxels: [u8; CHUNK_VOLUME],
  ) -> Self {
    Self {
      palette,
      voxels,
    }
  }

  pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: u8) {
    self.voxels[(x as usize * CHUNK_AREA) + (y as usize * CHUNK_SIZE) + z as usize] = voxel;
  }

  pub fn get(&self, x: u32, y: u32, z: u32) -> u8 {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkVoxelPosition {
  pub x: usize,
  pub y: usize,
  pub z: usize,
}

impl ChunkVoxelPosition {
  pub fn get_1d(&self) -> usize {
    (self.x * CHUNK_AREA) + (self.y * CHUNK_SIZE) + self.z
  }
}

#[repr(C)]
pub struct ChunkIterator<'a> {
  chunk: &'a Chunk,
  current: ChunkVoxelPosition,
}

impl<'a> Iterator for ChunkIterator<'a> {
  type Item = (ChunkVoxelPosition, u8);

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
