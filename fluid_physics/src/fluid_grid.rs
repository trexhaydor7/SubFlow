#![allow(dead_code, unused_variables)]

const DT: f32 = 0.016; //for 60 FPS
const gravity: f32 = 9.81;
const iteration_project_amount: usize = 20;
const o :f32 = 1.9; //Overrelaxation factor, voodoo magic atp

pub struct FluidGrid {
    //Size of grid
    nx: usize,
    ny: usize,
    nz: usize,
    active: Vec<bool>,
    dt: f32,

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
            active: vec![true; size],
            dt: 0.0,
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

    pub fn get_dt(&self) -> f32 {
        self.dt
    }
    //CARGO ITS RIGHTTTT HERREEE!!!!!
    pub fn increment_dt(&mut self) {
        self.dt += DT;
    }
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        return x + y * self.nx + z * self.nx * self.ny;
    }

    pub fn get_velocity(&self, x: usize, y: usize, z: usize) -> (f32, f32, f32) {
        let i = self.idx(x, y, z);
        (self.vecx[i], self.vecy[i], self.vecz[i])
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

    pub fn step(&mut self, dt: f32) {
        self.increment_dt();
        self.integrate(gravity); // apply gravity
        self.project();          // fix incompressibility
        self.extrapolate();      // fix boundaries
        self.advect(dt);         // move density according to velocity
    }

    pub fn advect(&mut self, dt: f32) {

    }

    //Fix the velocity field to be divergence-free. Only applies to active cells.
    pub fn project(&mut self) {
    for i in 0..iteration_project_amount {
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    if !self.active[i] {
                        continue;
                    }
                    if self.active[i] {
                        // Compute divergence and solve for pressure
                        // Adjust velocity to be divergence-free
                        
                            // count fluid neighbors
                            let sx0 = self.active[self.idx(x-1, y, z)] as i32 as f32;
                            let sx1 = self.active[self.idx(x+1, y, z)] as i32 as f32;
                            let sy0 = self.active[self.idx(x, y-1, z)] as i32 as f32;
                            let sy1 = self.active[self.idx(x, y+1, z)] as i32 as f32;
                            let sz0 = self.active[self.idx(x, y, z-1)] as i32 as f32;
                            let sz1 = self.active[self.idx(x, y, z+1)] as i32 as f32;
                            let s = sx0 + sx1 + sy0 + sy1 + sz0 + sz1;

                            if s == 0.0 {
                                continue; // No fluid neighbors, skip
                            }

                            // Iterative solver for pressure
                            let j = self.idx(x, y, z);
                            f32 sx = self.vecx[self.idx(x + 1, y, z)] - self.vecx[self.idx(x - 1, y, z)];
                            f32 sy = self.vecy[self.idx(x, y + 1, z)] - self.vecy[self.idx(x, y - 1, z)];
                            f32 sz = self.vecz[self.idx(x, y, z + 1)] - self.vecz[self.idx(x, y, z - 1)];
                            let divergence = (sx + sy + sz) * 1.9;

                            println!("Divergence at ({}, {}, {}) at {} iteration: {}", x, y, z, iteration_project_amount, divergence);         

                            self.vecx[self.idx(x-1, y, z)] -= sx0 * p;
                            self.vecx[self.idx(x+1, y, z)] += sx1 * p;
                            self.vecy[self.idx(x, y-1, z)] -= sy0 * p;
                            self.vecy[self.idx(x, y+1, z)] += sy1 * p;
                            self.vecz[self.idx(x, y, z-1)] -= sz0 * p;
                            self.vecz[self.idx(x, y, z+1)] += sz1 * p;
                        }
                    }
                }
            }
        }
    }

    //Add gravity to the velocity field. Only applies to active cells.
    pub fn integrate(&mut self, gravity: f32) {
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    if self.active[i] {
                    self.vecy[i] -= gravity * dt;
                    }
                }
            }
        }
    }

    //Ensure that advect doesn't cause unecessary drag when approaching boundaries.
    pub fn extrapolate(&mut self) {

        //x faces
        for z in 0..self.nz {
            for y in 0..self.ny {
                self.vecx[self.idx(0, y, z)] = self.vecx[self.idx(1, y, z)];
                self.vecx[self.idx(self.nx - 1, y, z)] = self.vecx[self.idx(self.nx - 2, y, z)];
                }
            }
        //y faces
        for z in 0..self.nz {
            for y in 0..self.ny {
                self.vecx[self.idx(x, 0, z)] = self.vecx[self.idx(x, 1, z)];
                self.vecx[self.idx(x, self.ny-1, z)] = self.vecx[self.idx(x, self.ny-2, z)];
                }
            }
         //z faces
        for z in 0..self.nz {
            for y in 0..self.ny {
                self.vecx[self.idx(x, y, 0)] = self.vecx[self.idx(x, y, 1)];
                self.vecx[self.idx(x, y, self.nz-1)] = self.vecx[self.idx(x, y, self.nz-2)];
                }
            }
        }
    //Vector in the form [nx, ny, nz, x, y, z, density. x, y, z, density, ...]
    pub fn raw_3d_matrix(&self) -> Vec<f32> {
        let mut buf = Vec::new();

        //Return total matrix size
        buf.push(self.nx as f32);
        buf.push(self.ny as f32);
        buf.push(self.nz as f32);

        //Then return total information of the 3D matrix
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    buf.push(x as f32);
                    buf.push(y as f32);
                    buf.push(z as f32);
                    buf.push(self.density[i]);
                }
            }
        }
        buf
    }

    /*
     pub fn diffuse(&self) {

     }
    */
}
