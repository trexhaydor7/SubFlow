import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import init, { FluidSim } from './fluid_physics/pkg/fluid_physics.js';

const canvas = document.getElementById('c');
const view = document.getElementById('view');

const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: canvas });
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setSize(view.clientWidth, view.clientHeight);
renderer.setClearColor(0x111111);
renderer.setPixelRatio(window.devicePixelRatio);
renderer.shadowMap.enabled = true;

const scene = new THREE.Scene();

await init();
const sim = new FluidSim(16, 16, 16);
window.sim = sim;

const CELL_SIZE = 0.5;

// Read grid dimensions from the sim directly via the first call
const rawMatrixInitial = Array.from(sim.raw_3d_matrix());
const xLength = rawMatrixInitial[0];
const yLength = rawMatrixInitial[1];
const zLength = rawMatrixInitial[2];
console.log(`Grid size: ${xLength} x ${yLength} x ${zLength}`);

const gridCenterX = (xLength * CELL_SIZE) / 2;
const gridCenterY = (yLength * CELL_SIZE) / 2;
const gridCenterZ = (zLength * CELL_SIZE) / 2;

const camera = new THREE.PerspectiveCamera(45, view.clientWidth / view.clientHeight, 0.1, 1000);
camera.position.set(gridCenterX + 14, gridCenterY + 10, gridCenterZ + 14);
camera.lookAt(gridCenterX, gridCenterY, gridCenterZ);

const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.enablePan = false;
controls.minDistance = 2;
controls.maxDistance = 50;
controls.minPolarAngle = 0.3;
controls.maxPolarAngle = 1.5;
controls.autoRotate = false;
controls.target = new THREE.Vector3(gridCenterX, gridCenterY, gridCenterZ);
controls.update();

// Ground plane
const groundGeometry = new THREE.PlaneGeometry(xLength * CELL_SIZE + 4, zLength * CELL_SIZE + 4);
groundGeometry.rotateX(-Math.PI / 2);
const groundMesh = new THREE.Mesh(groundGeometry, new THREE.MeshStandardMaterial({ color: 0x333333 }));
groundMesh.receiveShadow = true;
groundMesh.position.set(gridCenterX, -0.05, gridCenterZ);
scene.add(groundMesh);

const spotLight = new THREE.SpotLight(0xffffff, 3000, 100, 0.22, 1);
spotLight.position.set(gridCenterX, 25, gridCenterZ);
spotLight.castShadow = true;
spotLight.shadow.bias = -0.0001;
scene.add(spotLight);
scene.add(new THREE.AmbientLight(0xffffff, 0.5));

window.addEventListener('resize', () => {
  camera.aspect = view.clientWidth / view.clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(view.clientWidth, view.clientHeight);
});

// ─── Alley config ────────────────────────────────────────────────────────────
// Alley channel: x = WALL_L+1 .. WALL_R-1, floor at y = FLOOR_Y
// Walls and floor are inactive solid cells.
const WALL_L  = 4;   // left wall x
const WALL_R  = 9;   // right wall x
const FLOOR_Y = 2;   // floor y (fluid sits above this)
const NX = xLength;
const NY = yLength;
const NZ = zLength;

function buildAlley() {
  // Left wall — full height, full z depth
  for (let y = 0; y < NY; y++)
    for (let z = 0; z < NZ; z++)
      sim.set_active(WALL_L, y, z, false);

  // Right wall
  for (let y = 0; y < NY; y++)
    for (let z = 0; z < NZ; z++)
      sim.set_active(WALL_R, y, z, false);

  // Floor — spans between walls
  for (let x = WALL_L; x <= WALL_R; x++)
    for (let z = 0; z < NZ; z++)
      sim.set_active(x, FLOOR_Y, z, false);
}

function addAlleyVisuals() {
  const wallMat  = new THREE.MeshPhongMaterial({ color: 0x885533, side: THREE.DoubleSide });
  const floorMat = new THREE.MeshPhongMaterial({ color: 0x554433, side: THREE.DoubleSide });

  // Left wall
  const wallH = NY * CELL_SIZE;
  const wallD = NZ * CELL_SIZE;
  const lwGeo = new THREE.BoxGeometry(CELL_SIZE, wallH, wallD);
  const lw = new THREE.Mesh(lwGeo, wallMat);
  lw.position.set(WALL_L * CELL_SIZE, (NY / 2) * CELL_SIZE, (NZ / 2) * CELL_SIZE);
  scene.add(lw);

  // Right wall
  const rw = new THREE.Mesh(lwGeo.clone(), wallMat);
  rw.position.set(WALL_R * CELL_SIZE, (NY / 2) * CELL_SIZE, (NZ / 2) * CELL_SIZE);
  scene.add(rw);

  // Floor between walls
  const channelW = (WALL_R - WALL_L + 1) * CELL_SIZE;
  const fGeo = new THREE.BoxGeometry(channelW, CELL_SIZE, wallD);
  const fl = new THREE.Mesh(fGeo, floorMat);
  // Center x between WALL_L and WALL_R
  fl.position.set(
    ((WALL_L + WALL_R) / 2) * CELL_SIZE,
    FLOOR_Y * CELL_SIZE,
    (NZ / 2) * CELL_SIZE
  );
  scene.add(fl);
}

buildAlley();
addAlleyVisuals();

// ─── Fluid voxel rendering ────────────────────────────────────────────────────
// Reuse a single geometry; create one mesh per visible cell each frame.
const voxelGeo = new THREE.BoxGeometry(CELL_SIZE * 0.95, CELL_SIZE * 0.95, CELL_SIZE * 0.95);
const meshPool = [];

function rebuildScene() {
  // Remove and dispose old meshes
  for (const m of meshPool) {
    scene.remove(m);
    m.material.dispose();
  }
  meshPool.length = 0;

  const raw = sim.raw_3d_matrix();
  let maxD = 0, visCount = 0; // Float32Array / Box<[f32]>
  // Layout: [nx, ny, nz,  x,y,z,density,  x,y,z,density, ...]
  // So data starts at index 3, stride 4.
  for (let i = 3; i < raw.length; i += 4) {
    const gx = raw[i];       // grid x
    const gy = raw[i + 1];   // grid y
    const gz = raw[i + 2];   // grid z
    const d  = raw[i + 3];   // density 0..1

    if (d < 0.05) continue;

    const opacity = Math.min(0.35 + d * 0.65, 0.95);
    const mat = new THREE.MeshPhongMaterial({
      color: new THREE.Color(0.05, 0.25 + d * 0.4, 0.75 + d * 0.25),
      transparent: true,
      opacity,
      depthWrite: false,
    });

    const mesh = new THREE.Mesh(voxelGeo, mat);
    // Convert grid coords to world coords — CELL_SIZE per cell
    mesh.position.set(gx * CELL_SIZE, gy * CELL_SIZE, gz * CELL_SIZE);
    scene.add(mesh);
    meshPool.push(mesh);
    visCount++;
  }
  // Uncomment to debug: console.log(`max density: ${maxD.toFixed(3)}, visible cells: ${visCount}`);
}

// ─── Pour source ─────────────────────────────────────────────────────────────
// Pour into the top of the channel, centred in z, well within walls.
const POUR_Y  = NY - 3;                        // near top of grid
const POUR_X0 = WALL_L + 1;
const POUR_X1 = WALL_R - 1;
const POUR_Z0 = Math.floor(NZ * 0.3);
const POUR_Z1 = Math.floor(NZ * 0.7);

function pourWater() {
  for (let x = POUR_X0; x <= POUR_X1; x++) {
    for (let z = POUR_Z0; z <= POUR_Z1; z++) {
      sim.set_density(x, POUR_Y, z, 1.0);
      sim.set_velocity(x, POUR_Y, z, 0, -3.0, 0);
    }
  }
}

// ─── Animation loop ───────────────────────────────────────────────────────────
let lastTime = performance.now();

function animate() {
  requestAnimationFrame(animate);

  const now = performance.now();
  const dt  = Math.min((now - lastTime) / 1000, 0.016);
  lastTime  = now;

  pourWater();
  sim.step(dt);
  rebuildScene();
  controls.update();
  renderer.render(scene, camera);
}

animate();