use wasm_bindgen::prelude::*;
mod fluid_grid;
pub use fluid_grid::FluidGrid;

#[wasm_bindgen]
pub struct FluidSim {
    grid: FluidGrid,
}

#[wasm_bindgen]
impl FluidSim {
    #[wasm_bindgen(constructor)]
    pub fn new(nx: usize, ny: usize, nz: usize) -> FluidSim {
        FluidSim { grid: FluidGrid::new(nx, ny, nz) }
    }
    pub fn get_dt(&self) -> f32 { self.grid.get_dt() }
    pub fn increment_dt(&mut self) -> f32 { self.grid.increment_dt(); self.grid.get_dt() }
    pub fn step(&mut self, dt: f32) { self.grid.step(dt); }
    pub fn set_velocity(&mut self, x: usize, y: usize, z: usize, vx: f32, vy: f32, vz: f32) {
        self.grid.set_velocity(x, y, z, vx, vy, vz);
    }
    pub fn raw_3d_matrix(&self) -> Box<[f32]> { self.grid.raw_3d_matrix().into() }
    pub fn get_density(&self, x: usize, y: usize, z: usize) -> f32 { self.grid.get_density(x, y, z) }
    pub fn set_density(&mut self, x: usize, y: usize, z: usize, density: f32) {
        self.grid.set_density(x, y, z, density);
    }
    pub fn set_active(&mut self, x: usize, y: usize, z: usize, active: bool) {
        self.grid.set_active(x, y, z, active);
    }
    pub fn add_inlet(&mut self, x: usize, y: usize, z: usize, vx: f32, vy: f32, vz: f32) {
        self.grid.add_inlet(x, y, z, vx, vy, vz);
    }
    pub fn clear_inlets(&mut self) {
        self.grid.clear_inlets();
    }
}