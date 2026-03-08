import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
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
const sim = new FluidSim(12, 12, 12);
window.sim = sim;

const CELL_SIZE = 0.5;
const geometry = new THREE.BoxGeometry(CELL_SIZE, CELL_SIZE, CELL_SIZE);

const rawMatrixInitial = Array.from(sim.raw_3d_matrix());
const xLength = rawMatrixInitial[0];
const yLength = rawMatrixInitial[1];
const zLength = rawMatrixInitial[2];
console.log(`Grid size: ${xLength} x ${yLength} x ${zLength}`);

const gridCenterX = (xLength * CELL_SIZE) / 2;
const gridCenterY = (yLength * CELL_SIZE) / 2;
const gridCenterZ = (zLength * CELL_SIZE) / 2;

const camera = new THREE.PerspectiveCamera(45, view.clientWidth / view.clientHeight, 0.1, 1000);
camera.position.set(gridCenterX + 12, gridCenterY + 8, gridCenterZ + 12);
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
const groundGeometry = new THREE.PlaneGeometry(xLength * CELL_SIZE + 4, zLength * CELL_SIZE + 4, 32, 32);
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

// Build solid alley walls using set_active(false)
// Channel is x=4..7 (4 wide), walls at x=3 and x=8, floor at y=1
function buildAlley() {
  const nx = xLength, ny = yLength, nz = zLength;

  // Left wall
  for (let y = 0; y < ny; y++)
    for (let z = 0; z < nz; z++)
      sim.set_active(3, y, z, false);

  // Right wall
  for (let y = 0; y < ny; y++)
    for (let z = 0; z < nz; z++)
      sim.set_active(8, y, z, false);

  // Floor
  for (let x = 3; x <= 8; x++)
    for (let z = 0; z < nz; z++)
      sim.set_active(x, 1, z, false);
}

// Visible Three.js alley geometry
function addAlleyVisuals() {
  const wallMat = new THREE.MeshPhongMaterial({ color: 0x885533 });
  const floorMat = new THREE.MeshPhongMaterial({ color: 0x554433 });

  // Left wall
  const lwGeo = new THREE.BoxGeometry(CELL_SIZE, yLength * CELL_SIZE, zLength * CELL_SIZE);
  const lw = new THREE.Mesh(lwGeo, wallMat);
  lw.position.set(3 * CELL_SIZE, (yLength / 2) * CELL_SIZE, (zLength / 2) * CELL_SIZE);
  scene.add(lw);

  // Right wall
  const rw = new THREE.Mesh(lwGeo.clone(), wallMat);
  rw.position.set(8 * CELL_SIZE, (yLength / 2) * CELL_SIZE, (zLength / 2) * CELL_SIZE);
  scene.add(rw);

  // Floor
  const fGeo = new THREE.BoxGeometry(5 * CELL_SIZE, CELL_SIZE, zLength * CELL_SIZE);
  const fl = new THREE.Mesh(fGeo, floorMat);
  fl.position.set(5.5 * CELL_SIZE, 1 * CELL_SIZE, (zLength / 2) * CELL_SIZE);
  scene.add(fl);
}

buildAlley();
addAlleyVisuals();

// Fluid voxel mesh pool
const meshPool = [];

function rebuildScene() {
  for (const m of meshPool) scene.remove(m);
  meshPool.length = 0;

  const rawMatrix = Array.from(sim.raw_3d_matrix());
  rawMatrix.splice(0, 3);

  for (let i = 0; i < rawMatrix.length; i += 4) {
    const x = rawMatrix[i], y = rawMatrix[i+1], z = rawMatrix[i+2], d = rawMatrix[i+3];
    if (d < 0.01) continue;

    const mat = new THREE.MeshPhongMaterial({
      color: new THREE.Color(0.0, 0.3 + d * 0.4, 0.8 + d * 0.2),
      transparent: true,
      opacity: Math.min(d * 0.85, 0.9),
      depthWrite: false,
    });

    const mesh = new THREE.Mesh(geometry, mat);
    mesh.position.set(x * CELL_SIZE, y * CELL_SIZE, z * CELL_SIZE);
    scene.add(mesh);
    meshPool.push(mesh);
  }
}

let lastTime = performance.now();
function animate() {
  requestAnimationFrame(animate);

  const now = performance.now();
  const dt = Math.min((now - lastTime) / 1000, 0.016);
  lastTime = now;

  // Pour water into top of channel each frame
  for (let x = 4; x <= 7; x++) {
    sim.set_density(x, 9, 5, 1.0);
    sim.set_density(x, 9, 6, 1.0);
    sim.set_velocity(x, 9, 5, 0, -2.0, 0.3);
    sim.set_velocity(x, 9, 6, 0, -2.0, 0.3);
  }

  sim.step(dt);
  rebuildScene();
  controls.update();
  renderer.render(scene, camera);
}

animate();