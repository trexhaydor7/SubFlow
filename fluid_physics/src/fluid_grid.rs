#![allow(dead_code, unused_variables)]

const DT: f32 = 0.016; //for 60 FPS
const GRAVITY: f32 = 9.81;
const ITERATION_AMOUNT: usize = 20;
const OVERRELAXATION: f32 = 1.9; //Overrelaxation factor, voodoo magic atp

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
        self.integrate(); // apply gravity
        self.project();          // fix incompressibility
        self.extrapolate();      // fix boundaries
        self.advect(dt);         // move density according to velocity
    }

    pub fn advect(&mut self, dt: f32) {

    }

    //Fix the velocity field to be divergence-free. Only applies to active cells.
    pub fn project(&mut self) {
    for i in 0..ITERATION_AMOUNT {
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

                            let sx : f32 = self.vecx[self.idx(x + 1, y, z)] - self.vecx[self.idx(x - 1, y, z)];
                            let sy : f32 = self.vecy[self.idx(x, y + 1, z)] - self.vecy[self.idx(x, y - 1, z)];
                            let sz : f32 = self.vecz[self.idx(x, y, z + 1)] - self.vecz[self.idx(x, y, z - 1)];
                            let p = (sx + sy + sz) * OVERRELAXATION;

                            println!("Divergence at ({}, {}, {}) at {} iteration: {}", x, y, z, ITERATION_AMOUNT, p);         

                            
                            
                            let i = self.idx(x, y, z);
                            let i_xp1 = self.idx(x+1, y, z);
                            let i_xm1 = self.idx(x-1, y, z);
                            let i_yp1 = self.idx(x, y+1, z);
                            let i_ym1 = self.idx(x, y-1, z);
                            let i_zp1 = self.idx(x, y, z+1);
                            let i_zm1 = self.idx(x, y, z-1);

                            
                            
                            self.vecx[i_xm1] -= sx0 * p;
                            self.vecx[i_xp1] += sx1 * p;
                            self.vecy[i_ym1] -= sy0 * p;
                            self.vecy[i_yp1] += sy1 * p;
                            self.vecz[i_zm1] -= sz0 * p;
                            self.vecz[i_zp1] += sz1 * p;
                        }
                    }
                }
            }
        }
    }

    //Add gravity to the velocity field. Only applies to active cells.
    pub fn integrate(&mut self) {
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    if self.active[i] {
                    self.vecy[i] -= GRAVITY * self.get_dt();
                    }
                }
            }
        }
    }

    //Ensure that advect doesn't cause unecessary drag when approaching boundaries.
    pub fn extrapolate(&mut self) {

        //x faces
        for z in 0..self.nz {
            for x in 0..self.nx {
                let bar1 = self.idx(x, 0, z);
                let active1 = self.idx(x, 1, z);
                let bar2 = self.idx(x, self.ny-1, z);
                let active2 = self.idx(x, self.ny-2, z);

                self.vecx[bar1] = self.vecx[active1];
                self.vecx[bar2] = self.vecx[active2];
                }
            }
        //y faces
        for z in 0..self.nz {
            for y in 0..self.ny {
                let bar1 = self.idx(0, y, z);
                let active1 = self.idx(1, y, z);
                let bar2 = self.idx(self.nx-1, y, z);
                let active2 = self.idx(self.nx-2, y, z);

                self.vecy[bar1] = self.vecx[active1];
                self.vecy[bar2] = self.vecx[active2];
            }
        }
         //z faces
        for y in 0..self.ny {
            for x in 0..self.nx {
                let bar1 = self.idx(x, y, 0);
                let active1 = self.idx(x, y, 1);
                let bar2 = self.idx(x, y, self.nz-1);
                let active2 = self.idx(x, y, self.nz-2);

                self.vecx[bar1] = self.vecx[active1];
                self.vecx[bar2] = self.vecx[active2];
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
