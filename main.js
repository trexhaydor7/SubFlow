import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import init, { FluidSim } from './fluid_physics/pkg/fluid_physics.js';

const canvas = document.getElementById('c');
const view   = document.getElementById('view');

const renderer = new THREE.WebGLRenderer({ antialias: true, canvas });
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setSize(view.clientWidth, view.clientHeight);
renderer.setClearColor(0x020d1a);
renderer.setPixelRatio(window.devicePixelRatio);
renderer.shadowMap.enabled = true;

const scene = new THREE.Scene();

await init();

// Grid: 20 x 16 x 20
const sim = new FluidSim(40, 32, 36);
window.sim = sim;

const CELL_SIZE = 0.5;
const raw0 = sim.raw_3d_matrix();
const NX = raw0[0];
const NY = raw0[1];
const NZ = raw0[2];
console.log(`Grid: ${NX} x ${NY} x ${NZ}`);

const gridCenterX = (NX * CELL_SIZE) / 2;
const gridCenterY = (NY * CELL_SIZE) / 2;
const gridCenterZ = (NZ * CELL_SIZE) / 2;

// Camera — positioned to the left and slightly above to watch water flow in
const camera = new THREE.PerspectiveCamera(50, view.clientWidth / view.clientHeight, 0.1, 1000);
camera.position.set(-3, gridCenterY + 4, gridCenterZ);
camera.lookAt(gridCenterX, CELL_SIZE * 3, gridCenterZ);

const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.enablePan     = false;
controls.minDistance   = 1;
controls.maxDistance   = 60;
controls.minPolarAngle = 0.1;
controls.maxPolarAngle = 1.5;
controls.target.set(gridCenterX, CELL_SIZE * 3, gridCenterZ);
controls.update();

// Lights
const spot = new THREE.SpotLight(0xffffff, 2000, 120, 0.3, 1);
spot.position.set(gridCenterX, 20, gridCenterZ);
spot.castShadow  = true;
spot.shadow.bias = -0.0001;
scene.add(spot);
scene.add(new THREE.AmbientLight(0xffffff, 0.6));

window.addEventListener('resize', () => {
  camera.aspect = view.clientWidth / view.clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(view.clientWidth, view.clientHeight);
});

// Floor
const FLOOR_Y = 2;

// Fully scattered buildings: [bx, bz, wx, wz, bh]
//   bx/bz = grid cell origin (top-left corner)
//   wx/wz = footprint width in cells (x and z axes)
//   bh    = height in cells
// Grid is 40(x) × 36(z), water enters from x=0 side. Buildings kept off x<5 and x>36
// to leave entry space and exit room.
const BLDG_DEFS = [
  //  bx   bz  wx  wz  bh
  [  6,  1,  3,  5, 10],
  [  6, 10,  5,  3,  7],
  [  6, 18,  4,  6, 13],
  [  6, 28,  6,  3,  8],

  [ 12,  4,  4,  4, 11],
  [ 13, 22,  3,  7,  9],
  [ 11, 14,  6,  3,  6],
  [ 12, 30,  4,  4, 14],

  [ 19,  2,  5,  5,  8],
  [ 20, 12,  3,  4, 12],
  [ 18, 20,  6,  3, 10],
  [ 19, 27,  4,  6,  7],

  [ 26,  5,  3,  6, 13],
  [ 27, 15,  5,  3,  9],
  [ 25, 23,  4,  5, 11],
  [ 26, 31,  6,  3,  8],

  [ 32,  2,  4,  4,  9],
  [ 33, 11,  3,  6, 14],
  [ 32, 21,  5,  4,  7],
  [ 33, 29,  4,  5, 11],
];

const BLDG_COLORS = [
  0x7a8899, 0x8a7766, 0x667799, 0x998877,
  0x778899, 0x887766, 0x6688aa, 0x996655,
  0x7799aa, 0x886677, 0x99887a, 0x667788,
  0xaa8866, 0x778866, 0x8899aa, 0x996677,
  0x6677aa, 0x887799, 0xaa7766, 0x668899,
];

function buildCity() {
  // Solid floor across entire grid
  for (let x = 0; x < NX; x++)
    for (let z = 0; z < NZ; z++)
      sim.set_active(x, FLOOR_Y, z, false);

  // Mark building cells as solid using per-building wx/wz footprint
  for (const [bx, bz, wx, wz, bh] of BLDG_DEFS) {
    for (let x = bx; x < bx + wx; x++)
      for (let z = bz; z < bz + wz; z++)
        for (let y = FLOOR_Y; y <= FLOOR_Y + bh; y++)
          sim.set_active(x, y, z, false);
  }
}

function addCityVisuals() {
  // Pavement slab
  const paveMat = new THREE.MeshPhongMaterial({ color: 0x1a1a1a });
  const paveGeo = new THREE.BoxGeometry(NX * CELL_SIZE, CELL_SIZE * 0.3, NZ * CELL_SIZE);
  const pave    = new THREE.Mesh(paveGeo, paveMat);
  pave.receiveShadow = true;
  pave.position.set(gridCenterX, FLOOR_Y * CELL_SIZE - 0.05, gridCenterZ);
  scene.add(pave);

  BLDG_DEFS.forEach(([bx, bz, wx, wz, bh], idx) => {
    const w  = wx * CELL_SIZE;
    const d  = wz * CELL_SIZE;
    const h  = bh * CELL_SIZE;
    const cx = (bx + wx / 2) * CELL_SIZE;
    const cz = (bz + wz / 2) * CELL_SIZE;

    // Main building body
    const geo  = new THREE.BoxGeometry(w, h, d);
    const mat  = new THREE.MeshPhongMaterial({ color: BLDG_COLORS[idx], shininess: 20 });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow    = true;
    mesh.receiveShadow = true;
    mesh.position.set(cx, (FLOOR_Y + bh / 2) * CELL_SIZE, cz);
    scene.add(mesh);

    // Roof cap
    const roofGeo = new THREE.BoxGeometry(w + 0.05, 0.06, d + 0.05);
    const roofMat = new THREE.MeshPhongMaterial({ color: 0x0a0a0a });
    const roof    = new THREE.Mesh(roofGeo, roofMat);
    roof.position.set(cx, (FLOOR_Y + bh) * CELL_SIZE + 0.003, cz);
    scene.add(roof);

    // Windows — scale count to building size
    const winMat  = new THREE.MeshBasicMaterial({ color: 0xffffcc });
    const winGeo  = new THREE.PlaneGeometry(0.1, 0.12);
    const cols_xz = Math.max(1, Math.floor(wx * 0.8));
    const cols_zx = Math.max(1, Math.floor(wz * 0.8));

    for (let floor = 0; floor < bh - 1; floor++) {
      const wy = (FLOOR_Y + 0.7 + floor) * CELL_SIZE;

      // Front and back faces (z-facing), windows spread across x width
      for (let c = 0; c < cols_xz; c++) {
        const wx_off = (c - (cols_xz - 1) / 2) * (w / cols_xz) * 0.6;
        const mf = new THREE.Mesh(winGeo, winMat);
        mf.position.set(cx + wx_off, wy, cz + d / 2 + 0.01);
        scene.add(mf);
        const mb = new THREE.Mesh(winGeo, winMat);
        mb.position.set(cx + wx_off, wy, cz - d / 2 - 0.01);
        mb.rotation.y = Math.PI;
        scene.add(mb);
      }

      // Left and right faces (x-facing), windows spread across z depth
      for (let c = 0; c < cols_zx; c++) {
        const wz_off = (c - (cols_zx - 1) / 2) * (d / cols_zx) * 0.6;
        const ml = new THREE.Mesh(winGeo, winMat);
        ml.position.set(cx - w / 2 - 0.01, wy, cz + wz_off);
        ml.rotation.y = -Math.PI / 2;
        scene.add(ml);
        const mr = new THREE.Mesh(winGeo, winMat);
        mr.position.set(cx + w / 2 + 0.01, wy, cz + wz_off);
        mr.rotation.y = Math.PI / 2;
        scene.add(mr);
      }
    }
  });
}

buildCity();
addCityVisuals();

// Fluid rendering
// Fluid rendering — InstancedMesh: one mesh per density bucket, massive perf win
// over creating/destroying thousands of individual meshes each frame
const BUCKETS = 8;
const voxelGeo = new THREE.BoxGeometry(CELL_SIZE * 0.98, CELL_SIZE * 0.55, CELL_SIZE * 0.98);
const MAX_INSTANCES = 40 * 32 * 36;

const bucketMeshes = Array.from({ length: BUCKETS }, (_, bi) => {
  const t = (bi + 0.5) / BUCKETS;
  const mat = new THREE.MeshPhongMaterial({
    color:       new THREE.Color(0.05, 0.35 + t * 0.2, 0.95),
    transparent: true,
    opacity:     0.18 + t * 0.45,
    depthWrite:  false,
    shininess:   120,
    specular:    new THREE.Color(0.6, 0.8, 1.0),
  });
  const im = new THREE.InstancedMesh(voxelGeo, mat, MAX_INSTANCES);
  im.count = 0;
  im.frustumCulled = false;
  scene.add(im);
  return im;
});

const _dummy = new THREE.Object3D();

function rebuildScene() {
  const raw = sim.raw_3d_matrix();
  const bucketPos = Array.from({ length: BUCKETS }, () => []);

  for (let i = 3; i < raw.length; i += 4) {
    const d = raw[i + 3];
    if (d < 0.04) continue;
    const bi = Math.min(Math.floor(d * BUCKETS), BUCKETS - 1);
    bucketPos[bi].push(raw[i] * CELL_SIZE, raw[i+1] * CELL_SIZE, raw[i+2] * CELL_SIZE);
  }

  for (let b = 0; b < BUCKETS; b++) {
    const pos = bucketPos[b];
    const im  = bucketMeshes[b];
    let n = 0;
    for (let j = 0; j < pos.length; j += 3) {
      _dummy.position.set(pos[j], pos[j+1], pos[j+2]);
      _dummy.updateMatrix();
      im.setMatrixAt(n++, _dummy.matrix);
    }
    im.count = n;
    im.instanceMatrix.needsUpdate = true;
  }
}


// Water source: full left wall (x=1), all Z columns, single cell height
const POUR_X  = 1;
const POUR_Y0 = FLOOR_Y + 1;
const POUR_Y1 = FLOOR_Y + 1;  // single cell height — flat sheet

function registerInlets() {
  sim.clear_inlets();
  for (let z = 1; z < NZ - 1; z++) {
    for (let y = POUR_Y0; y <= POUR_Y1; y++) {
      sim.add_inlet(POUR_X,     y, z, 18.0, 0.0, 0.0);
      sim.add_inlet(POUR_X + 1, y, z, 18.0, 0.0, 0.0);
      sim.add_inlet(POUR_X + 2, y, z, 18.0, 0.0, 0.0);
      sim.add_inlet(POUR_X + 3, y, z, 18.0, 0.0, 0.0);
    }
  }
}
registerInlets();

// RGB colour controls
const sldR = document.getElementById('sld-r');
const sldG = document.getElementById('sld-g');
const sldB = document.getElementById('sld-b');
const lblR = document.getElementById('lbl-r');
const lblG = document.getElementById('lbl-g');
const lblB = document.getElementById('lbl-b');

function updateWaterColor() {
  const r = parseInt(sldR.value) / 255;
  const g = parseInt(sldG.value) / 255;
  const b = parseInt(sldB.value) / 255;
  lblR.textContent = sldR.value;
  lblG.textContent = sldG.value;
  lblB.textContent = sldB.value;
  bucketMeshes.forEach((im, bi) => {
    const t = (bi + 0.5) / BUCKETS;
    // Brighten toward full color at high density buckets
    im.material.color.setRGB(
      r * (0.4 + t * 0.6),
      g * (0.4 + t * 0.6),
      b * (0.4 + t * 0.6),
    );
    im.material.needsUpdate = true;
  });
}

sldR.addEventListener('input', updateWaterColor);
sldG.addEventListener('input', updateWaterColor);
sldB.addEventListener('input', updateWaterColor);

// Speed controls
let simSpeed  = 1.0;   // multiplier applied to dt each frame
let simPaused = false;

const btnHalf   = document.getElementById('btn-half');
const btnPlay   = document.getElementById('btn-play');
const btnDouble = document.getElementById('btn-double');

function setActive(btn) {
  [btnHalf, btnDouble].forEach(b => {
    b.style.background = '#0a2a44';
    b.style.borderColor = '#1a5a8a';
    b.style.color = '#7ec8e3';
  });
  if (btn) {
    btn.style.background = '#0d4f7c';
    btn.style.borderColor = '#3ab0d8';
    btn.style.color = '#e0f6ff';
  }
}

btnHalf.addEventListener('click', () => {
  simSpeed = simSpeed === 0.5 ? 1.0 : 0.5;
  setActive(simSpeed === 0.5 ? btnHalf : null);
});

btnDouble.addEventListener('click', () => {
  simSpeed = simSpeed === 2.0 ? 1.0 : 2.0;
  setActive(simSpeed === 2.0 ? btnDouble : null);
});

btnPlay.addEventListener('click', () => {
  simPaused = !simPaused;
  btnPlay.textContent = simPaused ? '▶' : '⏸';
  btnPlay.style.background = simPaused ? '#0d4f7c' : '#0a2a44';
  btnPlay.style.borderColor = simPaused ? '#3ab0d8' : '#1a5a8a';
  btnPlay.style.color       = simPaused ? '#e0f6ff'  : '#7ec8e3';
});

// Animation
let lastTime = performance.now();

function animate() {
  requestAnimationFrame(animate);
  const now = performance.now();
  const dt  = Math.min((now - lastTime) / 1000, 0.016);
  lastTime  = now;

  if (!simPaused) {
    sim.step(dt * simSpeed);
    rebuildScene();
  }
  controls.update();
  renderer.render(scene, camera);
}

animate();