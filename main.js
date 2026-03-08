import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const canvas = document.getElementById('c');
const view = document.getElementById('view');

const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: canvas });
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setSize(view.clientWidth, view.clientHeight); 
renderer.setClearColor(0x000000);
renderer.setPixelRatio(window.devicePixelRatio);
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;

const scene = new THREE.Scene();

const rawMatrix = raw_3d_matrix();
const cityGrid = [];
const xLength = rawMatrix.splice(1, 0);
const yLength = rawMatrix.splice(1, 1);
const zLength = rawLength.splice(1, 2);
for(let i = 0; i < (xLength * yLength * zLength); i = i + 4)
{
  cityGrid.push(new cell(rawMatrix[i], rawMatrix[i + 1], rawMatrix[i + 2], rawMatrix[i + 3]));
}

let xLocation = 0;
let yLocation = 0;
let zLocation = 0;
let cDensity = 0;
const geometry = new THREE.BoxGeometry(.1, .1, .1);
let nothing = color('white');
let blue = color('blue');
let solid = color('gray');
let material = new THREE.MeshPhongMaterial({nothing});

const cubeGrid = [];

for(let i = 0; i < cityGrid.length; i++){
  xLocation = cityGrid[i].getX();
  yLocation = cityGrid[i].getY();
  zLocation = cityGrid[i].getZ();
  cDensity = cityGrid[i].getD();

  let color, opacity;

  if(cDensity==0){
    continue;
  }
  else if(cDensity==1){
    color = 0x888888;
    opacity = 1.0;
  }
  else{
    color = 0x0044ff;
    opacity = cDensity;
  }

  const material = new THREE.MeshPhongMaterial({
    color: color,
    transparent: true,
    opacity: opacity,
    depthWrite: opacity < 1
    });

  const cube = new THREE.Mesh(geometry, material);
  cube.position.set(xLocation, yLocation, zLocation);
  scene.add(cube);
  cubeGrid.push(cube);
}

const camera = new THREE.PerspectiveCamera(45, view.clientWidth / view.clientHeight, 1, 1000);
camera.position.set(4, 5, 11);

const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.enablePan = false;
controls.minDistance = 5;
controls.maxDistance = 20;
controls.minPolarAngle = 0.5;
controls.maxPolarAngle = 1.5;
controls.autoRotate = false;
controls.target = new THREE.Vector3(0, 1, 0);
controls.update();

const groundGeometry = new THREE.PlaneGeometry(20, 20, 32, 32);
groundGeometry.rotateX(-Math.PI / 2);
const groundMaterial = new THREE.MeshStandardMaterial({
  color: 0x555555,
  side: THREE.DoubleSide
});
const groundMesh = new THREE.Mesh(groundGeometry, groundMaterial);
groundMesh.castShadow = false;
groundMesh.receiveShadow = true;
scene.add(groundMesh);

const spotLight = new THREE.SpotLight(0xffffff, 3000, 100, 0.22, 1);
spotLight.position.set(0, 25, 0);
spotLight.castShadow = true;
spotLight.shadow.bias = -0.0001;
scene.add(spotLight);

const loader = new GLTFLoader().setPath('/millennium_falcon');
loader.load('scene.gltf', (gltf) => {
  console.log('loading model');
  const mesh = gltf.scene;
  mesh.traverse((child) => {
    if (child.isMesh) {
      child.castShadow = true;
      child.receiveShadow = true;
    }
  });
  mesh.position.set(0, 1.05, -1);
  scene.add(mesh);

  const el = document.getElementById('progress-container');
  if (el) el.style.display = 'none';

}, (xhr) => {
  console.log(`loading ${xhr.loaded / xhr.total * 100}%`);
}, (error) => {
  console.error(error);
});

window.addEventListener('resize', () => {
  camera.aspect = view.clientWidth / view.clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(view.clientWidth, view.clientHeight);
});

function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}
animate();

class cell 
{
  constructor(x, y, z, d)
  {
    this.x = x;
    this.y = y;
    this.z = z;
    this.d = d;
  }
  
  getX()
  {
    return this.x;
  }

  getY()
  {
    return this.y;
  }

  getZ()
  {
    return this.z;
  }

  getD()
  {
    return this.d;
  }
  
  setX(x)
  {
    this.x = x;
  }

  setY(y)
  {
    this.y = y;
  }

  setZ(z)
  {
    this.z = z;
  }

  setD(d)
  {
    this.d = d;
  }
}