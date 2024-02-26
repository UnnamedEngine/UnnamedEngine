use std::collections::HashMap;

pub enum VoxelState {
  None,
  Solid,
  Liquid,
  Gas,
  Plasma,
}

pub struct VoxelDescritpion {
  pub name: String,
  pub state: VoxelState,
}

pub struct Voxel {
  pub id: u32,
  pub desc: VoxelDescritpion,
}

impl Voxel {
  pub fn new(id: u32, desc: VoxelDescritpion) -> Voxel {
    Self {
      id,
      desc,
    }
  }
}

pub struct VoxelManager {
  map_id: HashMap<u32, Voxel>,
  last_id: u32,
}

impl VoxelManager {
  pub fn new() -> VoxelManager {
    let mut map_id = HashMap::new();

    let mut last_id = 0;

    // Add void to serve as an initial voxel for all chunks
    last_id += 1;
    map_id.insert(last_id, Voxel::new(last_id, VoxelDescritpion {
      name: "Void".to_string(),
      state: VoxelState::None,
    }));

    Self {
      map_id,
      last_id,
    }
  }

  pub fn register_voxel(&mut self, desc: VoxelDescritpion) {
    self.last_id += 1;

    let voxel = Voxel::new(self.last_id, desc);
    self.map_id.insert(self.last_id, voxel);
  }

  pub fn get_voxel(&self, id: u32) -> Option<&Voxel> {
    self.map_id.get(&id)
  }
}


