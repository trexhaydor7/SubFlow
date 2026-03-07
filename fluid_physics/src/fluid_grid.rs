pub struct FluidGrid {
    //Size of grid
    nx: usize,
    ny: usize,
    nz: usize,

    //Updated values
    vecx: Vec<f32>,
    vecy: Vec<f32>,
    vecz: Vec<f32>,
    density: Vec<f32>,

    //Inital values
    vecx0: Vec<f32>,
    vecy0: Vec<f32>,
    vecz0: Vec<f32>,
    density0: Vec<f32>,
}

impl FluidGrid {
    pub fn new(nx: usize, ny: usize, nz: usize) -> FluidGrid {
        let size = nx * ny * nz;
        FluidGrid {
            nx,
            ny,
            nz,
            vecx: vec![0.0; size],
            vecy: vec![0.0; size],
            vecz: vec![0.0; size],
            density: vec![0.0; size],
            vecx0: vec![0.0; size],
            vecy0: vec![0.0; size],
            vecz0: vec![0.0; size],
            density0: vec![0.0; size],
        }
    }
    
    pub fn nx(&self) -> usize {
        self.nx
    }

    pub fn ny(&self) -> usize {
        self.ny
    }

    pub fn nz(&self) -> usize {
        self.nz
    }
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        return x + y * self.nx + z * self.nx * self.ny;
    }

    pub fn get_velocity(&self, x: usize, y: usize, z: usize) -> (f32, f32, f32) {
        let i = self.idx(x, y, z);
        (self.vecx[i],
        self.vecy[i], 
        self.vecz[i])
    }

    pub fn get_density(&self, x: usize, y: usize, z: usize) -> f32 {
        let i = self.idx(x, y, z);
        self.density[i]
    }

    pub fn set_velocity(&mut self, x: usize, y: usize, z: usize, vx: f32, vy: f32, vz: f32) {
        let i = self.idx(x, y, z);
        self.vecx[i] = vx;
        self.vecy[i] = vy;
        self.vecz[i] = vz;
    }

    pub fn set_density(&mut self, x: usize, y: usize, z: usize, density: f32) {
        let i = self.idx(x, y, z);
        self.density[i] = density;
    }
}
