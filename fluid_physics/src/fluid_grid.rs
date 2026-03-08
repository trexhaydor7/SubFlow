const DT: f32 = 0.016; //for 60 FPS
const GRAVITY: f32 = 4.0;
const ITERATION_AMOUNT: usize = 20;
const OVERRELAXATION: f32 = 1.7; //Overrelaxation factor, voodoo magic atp

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

    pub fn nx(&self) -> usize { self.nx }
    pub fn ny(&self) -> usize { self.ny }
    pub fn nz(&self) -> usize { self.nz }
    pub fn get_dt(&self) -> f32 { self.dt }

    pub fn increment_dt(&mut self) {
        self.dt += DT;
    }

    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.nx + z * self.nx * self.ny
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
        // FIX 1: Don't set velocity on inactive (wall) cells
        if self.active[i] {
            self.vecx[i] = vx;
            self.vecy[i] = vy;
            self.vecz[i] = vz;
        }
    }

    pub fn set_density(&mut self, x: usize, y: usize, z: usize, density: f32) {
        let i = self.idx(x, y, z);
        // FIX 1: Don't set density on inactive (wall) cells
        if self.active[i] {
            self.density[i] = density;
        }
    }

    // FIX 2: When marking a cell inactive, zero out its velocity and density
    // so the pressure solver and advection never see non-zero values leaking from walls
    pub fn set_active(&mut self, x: usize, y: usize, z: usize, active: bool) {
        let i = self.idx(x, y, z);
        self.active[i] = active;
        if !active {
            self.vecx[i] = 0.0;
            self.vecy[i] = 0.0;
            self.vecz[i] = 0.0;
            self.density[i] = 0.0;
            self.vecx0[i] = 0.0;
            self.vecy0[i] = 0.0;
            self.vecz0[i] = 0.0;
            self.density0[i] = 0.0;
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.increment_dt();
        self.integrate(dt);      // 1. apply gravity to velocities
        self.project();          // 2. make velocity field divergence-free
        self.extrapolate();      // 3. copy boundary velocities into ghost cells
        self.advect(dt);         // 4. move density+velocity through the corrected field
        self.enforce_walls();    // 5. zero out any bleed into solid cells

        for i in 0..self.density.len() {
            if self.density[i].is_nan() { self.density[i] = 0.0; }
            if self.vecx[i].is_nan()   { self.vecx[i]   = 0.0; }
            if self.vecy[i].is_nan()   { self.vecy[i]   = 0.0; }
            if self.vecz[i].is_nan()   { self.vecz[i]   = 0.0; }
            self.vecx[i]    = self.vecx[i].clamp(-20.0, 20.0);
            self.vecy[i]    = self.vecy[i].clamp(-20.0, 20.0);
            self.vecz[i]    = self.vecz[i].clamp(-20.0, 20.0);
            self.density[i] = self.density[i].clamp(0.0, 1.0);
        }
    }

    // FIX 3: Zero velocity and density on every inactive cell each frame
    fn enforce_walls(&mut self) {
        for i in 0..self.active.len() {
            if !self.active[i] {
                self.vecx[i]    = 0.0;
                self.vecy[i]    = 0.0;
                self.vecz[i]    = 0.0;
                self.density[i] = 0.0;
            }
        }
    }

    pub fn advect(&mut self, dt: f32) {
        self.vecx0    = self.vecx.clone();
        self.vecy0    = self.vecy.clone();
        self.vecz0    = self.vecz.clone();
        self.density0 = self.density.clone();

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    if !self.active[i] {
                        continue;
                    }

                    let (vx, vy, vz) = self.get_velocity(x, y, z);
                    let src_x = (x as f32 - vx * dt).clamp(0.5, (self.nx as f32) - 1.5);
                    let src_y = (y as f32 - vy * dt).clamp(0.5, (self.ny as f32) - 1.5);
                    let src_z = (z as f32 - vz * dt).clamp(0.5, (self.nz as f32) - 1.5);

                    let x0 = src_x as usize; let x1 = (x0 + 1).min(self.nx - 1);
                    let y0 = src_y as usize; let y1 = (y0 + 1).min(self.ny - 1);
                    let z0 = src_z as usize; let z1 = (z0 + 1).min(self.nz - 1);

                    let tx = src_x - x0 as f32;
                    let ty = src_y - y0 as f32;
                    let tz = src_z - z0 as f32;

                    // FIX 4: Use free functions that borrow the raw slices directly,
                    // avoiding the self-closure borrow conflict.
                    // Returns 0 for inactive (wall) cells so backtrace never pulls through walls.
                    let nx = self.nx; let ny = self.ny;
                    let idx = |cx: usize, cy: usize, cz: usize| cx + cy * nx + cz * nx * ny;

                    macro_rules! sample {
                        ($buf:expr, $cx:expr, $cy:expr, $cz:expr) => {{
                            let ii = idx($cx, $cy, $cz);
                            if self.active[ii] { $buf[ii] } else { 0.0 }
                        }};
                    }

                    // Trilinear interpolation — density
                    self.density[i] =
                        (1.0-tz) * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.density0,x0,y0,z0) + tx*sample!(self.density0,x1,y0,z0)) +
                                ty  * ((1.0-tx)*sample!(self.density0,x0,y1,z0) + tx*sample!(self.density0,x1,y1,z0))
                        ) +
                            tz  * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.density0,x0,y0,z1) + tx*sample!(self.density0,x1,y0,z1)) +
                                ty  * ((1.0-tx)*sample!(self.density0,x0,y1,z1) + tx*sample!(self.density0,x1,y1,z1))
                        );

                    // Velocity X
                    self.vecx[i] =
                        (1.0-tz) * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecx0,x0,y0,z0) + tx*sample!(self.vecx0,x1,y0,z0)) +
                                ty  * ((1.0-tx)*sample!(self.vecx0,x0,y1,z0) + tx*sample!(self.vecx0,x1,y1,z0))
                        ) +
                            tz  * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecx0,x0,y0,z1) + tx*sample!(self.vecx0,x1,y0,z1)) +
                                ty  * ((1.0-tx)*sample!(self.vecx0,x0,y1,z1) + tx*sample!(self.vecx0,x1,y1,z1))
                        );

                    // Velocity Y
                    self.vecy[i] =
                        (1.0-tz) * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecy0,x0,y0,z0) + tx*sample!(self.vecy0,x1,y0,z0)) +
                                ty  * ((1.0-tx)*sample!(self.vecy0,x0,y1,z0) + tx*sample!(self.vecy0,x1,y1,z0))
                        ) +
                            tz  * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecy0,x0,y0,z1) + tx*sample!(self.vecy0,x1,y0,z1)) +
                                ty  * ((1.0-tx)*sample!(self.vecy0,x0,y1,z1) + tx*sample!(self.vecy0,x1,y1,z1))
                        );

                    // Velocity Z
                    self.vecz[i] =
                        (1.0-tz) * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecz0,x0,y0,z0) + tx*sample!(self.vecz0,x1,y0,z0)) +
                                ty  * ((1.0-tx)*sample!(self.vecz0,x0,y1,z0) + tx*sample!(self.vecz0,x1,y1,z0))
                        ) +
                            tz  * (
                            (1.0-ty) * ((1.0-tx)*sample!(self.vecz0,x0,y0,z1) + tx*sample!(self.vecz0,x1,y0,z1)) +
                                ty  * ((1.0-tx)*sample!(self.vecz0,x0,y1,z1) + tx*sample!(self.vecz0,x1,y1,z1))
                        );
                }
            }
        }
    }

    // Fix the velocity field to be divergence-free using Gauss-Seidel relaxation.
    // This uses a cell-centered (non-staggered) velocity grid where vecx[i] is the
    // x-velocity AT cell center i. Divergence at cell (x,y,z) is computed from
    // the 6 face-adjacent neighbors, and pressure pushes velocity away from solid walls.
    pub fn project(&mut self) {
        for _iter in 0..ITERATION_AMOUNT {
            for z in 1..self.nz-1 {
                for y in 1..self.ny-1 {
                    for x in 1..self.nx-1 {
                        let i = self.idx(x, y, z);
                        if !self.active[i] { continue; }

                        // s-values: 1.0 if neighbor is fluid/open, 0.0 if solid wall
                        let sx0 = self.active[self.idx(x-1, y, z)] as i32 as f32;
                        let sx1 = self.active[self.idx(x+1, y, z)] as i32 as f32;
                        let sy0 = self.active[self.idx(x, y-1, z)] as i32 as f32;
                        let sy1 = self.active[self.idx(x, y+1, z)] as i32 as f32;
                        let sz0 = self.active[self.idx(x, y, z-1)] as i32 as f32;
                        let sz1 = self.active[self.idx(x, y, z+1)] as i32 as f32;
                        let s = sx0 + sx1 + sy0 + sy1 + sz0 + sz1;
                        if s == 0.0 { continue; }

                        // Divergence: net outflow from this cell
                        // For cell-centered grid: use neighbor center velocities projected onto face normals
                        let i_xp1 = self.idx(x+1, y, z);
                        let i_xm1 = self.idx(x-1, y, z);
                        let i_yp1 = self.idx(x, y+1, z);
                        let i_ym1 = self.idx(x, y-1, z);
                        let i_zp1 = self.idx(x, y, z+1);
                        let i_zm1 = self.idx(x, y, z-1);

                        let div = (self.vecx[i_xp1] - self.vecx[i_xm1])
                                + (self.vecy[i_yp1] - self.vecy[i_ym1])
                                + (self.vecz[i_zp1] - self.vecz[i_zm1]);

                        let p = -div / s * OVERRELAXATION;

                        // Push velocity of fluid neighbors away from this cell
                        if self.active[i_xm1] { self.vecx[i_xm1] -= sx0 * p; }
                        if self.active[i_xp1] { self.vecx[i_xp1] += sx1 * p; }
                        if self.active[i_ym1] { self.vecy[i_ym1] -= sy0 * p; }
                        if self.active[i_yp1] { self.vecy[i_yp1] += sy1 * p; }
                        if self.active[i_zm1] { self.vecz[i_zm1] -= sz0 * p; }
                        if self.active[i_zp1] { self.vecz[i_zp1] += sz1 * p; }
                        // Also update this cell's own velocity (no-penetration against walls)
                        self.vecx[i] += (sx1 - sx0) * p;
                        self.vecy[i] += (sy1 - sy0) * p;
                        self.vecz[i] += (sz1 - sz0) * p;
                    }
                }
            }
        }
    }

    // Add gravity to the velocity field. Only applies to active cells.
    pub fn integrate(&mut self, dt: f32) {
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.idx(x, y, z);
                    if self.active[i] {
                        self.vecy[i] -= GRAVITY * dt;
                    }
                }
            }
        }
    }

    // Ensure that advect doesn't cause unnecessary drag when approaching boundaries.
    // FIX 7: Removed the active[bar] = false lines — extrapolate must NOT overwrite
    // the active flags that were set by buildAlley / set_active. It only copies
    // velocities into the ghost border layer for smooth boundary conditions.
    pub fn extrapolate(&mut self) {
        // x-axis border (y=0 and y=ny-1 rows)
        for z in 0..self.nz {
            for x in 0..self.nx {
                let bar1    = self.idx(x, 0, z);
                let active1 = self.idx(x, 1, z);
                let bar2    = self.idx(x, self.ny-1, z);
                let active2 = self.idx(x, self.ny-2, z);
                self.vecx[bar1] = self.vecx[active1];
                self.vecx[bar2] = self.vecx[active2];
            }
        }
        // y-axis border (x=0 and x=nx-1 columns)
        for z in 0..self.nz {
            for y in 0..self.ny {
                let bar1    = self.idx(0, y, z);
                let active1 = self.idx(1, y, z);
                let bar2    = self.idx(self.nx-1, y, z);
                let active2 = self.idx(self.nx-2, y, z);
                self.vecy[bar1] = self.vecy[active1];
                self.vecy[bar2] = self.vecy[active2];
            }
        }
        // z-axis border (z=0 and z=nz-1 slices)
        for y in 0..self.ny {
            for x in 0..self.nx {
                let bar1    = self.idx(x, y, 0);
                let active1 = self.idx(x, y, 1);
                let bar2    = self.idx(x, y, self.nz-1);
                let active2 = self.idx(x, y, self.nz-2);
                self.vecz[bar1] = self.vecz[active1];
                self.vecz[bar2] = self.vecz[active2];
            }
        }
    }

    // Vector in the form [nx, ny, nz, x, y, z, density, x, y, z, density, ...]
    pub fn raw_3d_matrix(&self) -> Vec<f32> {
        let mut buf = Vec::new();
        buf.push(self.nx as f32);
        buf.push(self.ny as f32);
        buf.push(self.nz as f32);
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
}