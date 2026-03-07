use wasm_bindgen::prelude::*;

mod fluid_grid;
use fluid_grid::FluidGrid;  

#[wasm_bindgen]
pub struct FluidSim {
    grid: FluidGrid,
}
    
#[wasm_bindgen]
impl FluidSim {   

    #[wasm_bindgen(constructor)]
    pub fn new(nx: usize, ny: usize, nz: usize) -> FluidSim {
        FluidSim {
            grid: FluidGrid::new(nx, ny, nz),
        }
    }

    pub fn get_dt(&self) -> f32 {
        self.grid.get_dt()
    }

    pub fn dt_step(&self) -> f32 {
        self.grid.step(self.grid.get_dt());
    }
    pub fn raw_3d_matrix(&self) -> Vec<f32> {
        self.grid.raw_3d_matrix()
    }
}