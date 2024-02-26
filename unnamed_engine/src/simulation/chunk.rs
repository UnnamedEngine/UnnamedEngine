use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_SIZE;

#[derive(PartialEq, std::cmp::Eq, Hash, Clone, Copy)]
pub struct ChunkPos {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}

pub struct Chunk {
  pub position: ChunkPos,
  voxels: [u32; CHUNK_VOLUME],
}

impl Chunk {
  pub fn new(position: ChunkPos) -> Self {
    // Initializes
    let voxels = [1; CHUNK_VOLUME];

    Self {
      position,
      voxels,
    }
  }
}

struct ChunkManager {
  chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkManager {
  pub fn new() -> Self {
    let chunks = HashMap::new();

    Self {
      chunks,
    }
  }

  pub fn create_chunk(&mut self, position: ChunkPos) -> &Chunk {
    let chunk = Chunk::new(position);
    self.chunks.insert(position, chunk);
    self.chunks.get(&position).unwrap()
  }

  pub fn get_chunk(&self, position: ChunkPos) -> Option<&Chunk> {
    self.chunks.get(&position)
  }
}
