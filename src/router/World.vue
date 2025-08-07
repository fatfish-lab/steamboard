<template>
  <div class="world-container">
    <div ref="container" class="globe-container">
      <div v-for="label in visibleLabels" :key="label.code"
        class="country-label"
        :style="{ left: label.left + 'px', top: label.top + 'px', opacity: label.opacity, '--color': `var(--pico-color-${label.color})` }">
        <div class="content">
          <span class="flag">{{ flagFromCountryCode(label.code) }}</span>
          <h4 class="country-name">{{ label.name }}</h4>
          <h3 class="revenue">{{ label.revenue.toLocaleString('en-US', {
            style: "currency", currency: "USD",
            maximumFractionDigits: 0
          }) }}</h3>
          <small class="percentage">{{ label.percentage.toLocaleString('en-US', {
            style: "percent",
            maximumFractionDigits: 2
          }) }}</small>
        </div>
      </div>
    </div>
    <div class="pico toolbar">
      <sb-icon
        data-tooltip="Toggle auto-rotate"
        data-placement="right"
        :icon="rotating ? 'pause' : 'play_arrow'"
        @click="toggleAutoRotate"
        fill />
    </div>
    <div class="top-countries">
      <h5>Top 10 Countries</h5>
      <div class="country" v-for="country in top10Countries" :key="country.code">
        <span class="flag">{{ flagFromCountryCode(country.code) }}</span>
        <span class="country-name">{{ country.name }}</span>
        <span class="revenue">{{ country.revenue.toLocaleString('en-US', {
          style: "currency", currency: "USD",
          maximumFractionDigits: 0 }) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, reactive, useTemplateRef, watch } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';

import { useOptionsStore } from '@/stores/options.ts';
import { flagFromCountryCode } from '@/utils.js';

import { DateTime } from "luxon";

import countries from '@/countries.json';
import earthModelUrl from '@/assets/earth.glb';

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';
import { Line2 } from 'three/addons/lines/Line2.js';
import { LineGeometry } from 'three/addons/lines/LineGeometry.js';
import { LineMaterial } from 'three/addons/lines/LineMaterial.js';

const sales = ref([]);
const now = DateTime.now().minus({ days: 1 }); // Use minus one day to match Steam (ignoring uncompleted sales today)
const container = useTemplateRef('container')
const labels = reactive([]);
const beams = [];
const gltfloader = new GLTFLoader();
const loader = new GLTFLoader();

const controls: OrbitControls | null = ref(null);

const rotating = computed(() => {
  return controls.value?.autoRotate;
})
function toggleAutoRotate() {
  if (controls.value) {
    controls.value.autoRotate = !controls.value.autoRotate;
  }
}

let renderer, scene, camera, animationId, composer;

const options = useOptionsStore();
const period = computed(() => options.period);
const saleType = computed(() => options.salesType);

const saleProperty = computed(() => {
  return saleType.value === 'gross' ? 'gross_sales_usd' : 'net_sales_usd';
});

watch(period, (newPeriod) => {
  get_detailed_sales();
});

listen("sync-data", (event) => {
  const results = event.payload
  if (results) {
    results.forEach((result) => {
      if (options.isInPeriod(result.date)) {
        sales.value.push(result);
      }
    })
  }
});

async function get_detailed_sales() {
  let fromDate = options.from;
  let toDate = options.now;
  sales.value = await invoke("get_detailed_sales_command", { fromDate, toDate });
}

const maximumRevenue = computed(() => {
  return Object.values(salesByCountry.value).reduce((max, sale) => {
    return Math.max(max, sale.revenue || 0);
  }, 0);
});

const visibleLabels = computed(() => {
  return labels.filter(label => label.revenue > 0.01);
});

const salesByCountry = computed(() => {
  const countries = sales.value.reduce((countries, sale) => {
    const country = sale.country_code;
    if (!country) return countries; // Skip if no country code
    if ((sale[saleProperty.value] || 0) == 0) return countries; // Skip if no revenue
    if (!countries[country]) countries[country] = { revenue: 0, name: sale.country_name, code: country };
    countries[country].revenue += sale[saleProperty.value] || 0;
    return countries;
  }, {});
  return countries; // Filter out countries with no revenue
});

const top10Countries = computed(() => {
  return Object.entries(salesByCountry.value)
    .sort((a, b) => b[1].revenue - a[1].revenue)
    .slice(0, 10)
    .map(([code, data]) => ({
      code,
      ...data
    }));
});

const allRevenue = computed(() => {
  return sales.value.reduce((revenue, sale) => {
    return revenue + (sale[saleProperty.value] || 0);
  }, 0);
});

function latLngToVector3(lat, lng, radius) {
  const phi = (90 - lat) * (Math.PI / 180);
  const theta = (lng + 180) * (Math.PI / 180);
  return new THREE.Vector3(
    -radius * Math.sin(phi) * Math.cos(theta),
    radius * Math.cos(phi),
    radius * Math.sin(phi) * Math.sin(theta)
  );
}

function setupLabels() {
  for (const [key, country] of Object.entries(countries)) {
    const sales_by_country = salesByCountry.value[key];
    const code = key;
    const name = sales_by_country?.name || code;
    const revenue = sales_by_country?.revenue || 0;
    const percentage = sales_by_country?.revenue / allRevenue.value;

    // #RAY
    const pos3D = latLngToVector3(country.latitude, country.longitude, 2);
    const positions = [pos3D.x, pos3D.y, pos3D.z, pos3D.x, pos3D.y, pos3D.z];
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));

    let color = 'blue-500'
    const colors = [
      0, 0, 0
    ];
    if (percentage > 0.2) {
      colors[0] = 178 / 255;
      colors[1] = 229 / 255;
      colors[2] = 26 / 255;
      color = 'lime-150';

    } else if (percentage >= 0.01) {
      colors[0] = 255 / 255;
      colors[1] = 191 / 255;
      colors[2] = 0 / 255;
      color = 'amber-300';
    } else {
      colors[0] = 231 / 255;
      colors[1] = 75 / 255;
      colors[2] = 26 / 255;
      color = 'sand-300';
    }

    colors[3] = colors[0];
    colors[4] = colors[1];
    colors[5] = colors[2];
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    const beamMaterial = new THREE.LineBasicMaterial({
      vertexColors: true,
      linecap: 'round',
      fog: false,
    });
    const beam = new THREE.Line(geometry, beamMaterial);
    scene.add(beam);

    beams.push(beam);
    labels.push({
      code,
      name,
      color,
      percentage,
      revenue,
      left: 0,
      top: 0,
      opacity: 1
    });
  }
}

function updateLabels() {
  const globeCenter = new THREE.Vector3(0, 0, 0);
  const cameraDirection = new THREE.Vector3();
  camera.getWorldDirection(cameraDirection);
  for (const [i, label] of labels.entries()) {
    const beam = beams[i];
    const country = countries[label.code];
    const pos3D = latLngToVector3(country.latitude, country.longitude, 2);
    const normal = pos3D.clone().normalize();
    const visible = normal.dot(cameraDirection) < -0.4;
    const revenue = salesByCountry.value[label.code]?.revenue || 0.0;
    const pos = pos3D.clone().project(camera);
    const minHeight = 0.0;
    const maxHeight = 10.0;
    const maxRevenue = maximumRevenue.value || 1;
    const beamHeight = minHeight + (maxHeight - minHeight) * (revenue / maxRevenue);
    const positionAttribute = beam.geometry.getAttribute('position');
    pos3D.add(normal.multiplyScalar(beamHeight));

    label.left = (pos.x * 0.5 + 0.5) * container.value.clientWidth;
    label.top = (-pos.y * 0.5 + 0.5) * container.value.clientHeight;
    label.revenue = revenue;
    label.opacity = visible ? 1 : 0;

    positionAttribute.setXYZ(1, pos3D.x, pos3D.y, pos3D.z);
    positionAttribute.needsUpdate = true;
  }
}


onMounted(async () => {
  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true });
  renderer.setSize(container.value.offsetWidth, container.value.offsetHeight, false);
  renderer.domElement.style.width = '100%';
  renderer.domElement.style.height = '100%';
  renderer.domElement.style.display = 'block';
  container.value.appendChild(renderer.domElement);

  const pmremGenerator = new THREE.PMREMGenerator(renderer);

  scene = new THREE.Scene();
  scene.environment = pmremGenerator.fromScene(new RoomEnvironment(), 0.04).texture;
  // scene.fog = new THREE.Fog(0x0495f4, 4, 9);

  camera = new THREE.PerspectiveCamera(45, container.value.clientWidth / container.value.clientHeight, 1, 50);
  camera.position.set(0, 0, 7);

  const meshPhysicalMaterialOptions = {
    color: 0x049ef4,
    vertexColors: true,
    emissive: 0x265fba,
    roughness: 0.7,
    metalness: 0,
    ior: 1.5,
    reflectivity: 0.5,
    iridescence: 0,
    iridescenceIOR: 1.3,
    sheen: 0.5,
    sheenRoughness: 1,
    sheenColor: new THREE.Color(0xffde0a),
    clearcoat: 0,
    clearcoatRoughness: 0,
    specularIntensity: 1.2,
    specularColor: new THREE.Color(0xffffff),
    side: THREE.FrontSide,
    transparent: false,
    opacity: 1,
    fog: false
  };

  loader.load(earthModelUrl, (gltf) => {
    const mesh = gltf.scene.children[0];
    scene.add(gltf.scene);

    const material = new THREE.MeshPhysicalMaterial(meshPhysicalMaterialOptions);
    mesh.material = material;
    mesh.castShadow = false;
    mesh.receiveShadow = false;
    // globe = new THREE.Mesh(geometry, material);
    // globe.castShadow = true;
    // globe.receiveShadow = true;
    // scene.add(globe);
  });

  scene.add(new THREE.AmbientLight(0x1cc4b9));

  composer = new EffectComposer(renderer);
  composer.addPass(new RenderPass(scene, camera));

  controls.value = new OrbitControls(camera, renderer.domElement);
  controls.value.autoRotate = true;
  controls.value.autoRotateSpeed = -0.5;
  controls.value.enableDamping = true;
  controls.value.dampingFactor = 0.1;
  controls.value.enablePan = false;
  controls.value.minDistance = 4;
  controls.value.maxDistance = 10;
  controls.value.enableZoom = true;
  controls.value.rotateSpeed = 0.7;
  controls.value.zoomSpeed = 1;

  controls.value.addEventListener('start', () => {
    if (controls.value.autoRotate) controls.value.autoRotate = false;
  });

  await get_detailed_sales();
  setupLabels();

  // Animation loop
  function animate() {
    updateLabels();
    controls.value.update();
    composer.render();
    animationId = requestAnimationFrame(animate);
  }
  animate();

  window.addEventListener('resize', onResize);
});

function onResize() {
  if (!container.value || !renderer || !camera) return;
  const $parent = container.value.parentElement;
  if (!$parent) return;
  const width = $parent.offsetWidth;
  const height = $parent.offsetHeight;
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  if (composer) composer.setSize(width, height);
}

onBeforeUnmount(() => {
  cancelAnimationFrame(animationId);
  window.removeEventListener('resize', onResize);
  if (renderer) {
    renderer.dispose();
    renderer.domElement.remove();
  }
  if (controls.value) controls.value.dispose();
});
</script>

<style scoped lang="scss">
.world-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
  width: 100%;
  height: 100%;
  overflow: hidden;
  position: relative;

  .globe-container {
    border-radius: 50%;
    background: radial-gradient(closest-side,
        #1c2a3b 30%,
        color-mix(in oklab, #1c2a3b, var(--pico-background-color) 90%),
        var(--pico-background-color));
  }

  .top-countries {
    position: absolute;
    bottom: 16px;
    right: 16px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    font-size: 0.8rem;
    padding: 8px;
    border-radius: 8px;
    background-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);

    .country {
      display: inline-grid;
      grid-template-columns: min-content 1fr max-content;
      gap: 8px;
      padding: 4px 8px;
      border-radius: 4px;
      cursor: default;

      &:hover {
        background-color: var(--pico-muted-border-color);
      }
    }
  }

  .toolbar {
    position: absolute;
    bottom: 16px;
    left: 16px;
    z-index: 10;

    >span {
      background-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);
      ;
      padding: 4px;
      border-radius: 4px;

      &:hover {
        background-color: var(--pico-muted-border-color);
      }
    }
  }
}

.country-label {
  position: absolute;
  left: 0px;
  top: 0px;
  color: #fff;
  font-size: 12px;
  // pointer-events: none;
  // text-shadow: 0 0 4px #000;
  // white-space: nowrap;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-family: sans-serif;
  font-weight: bold;
  transition: opacity 0.3s ease-in-out;
  --color: var(--pico-color-blue-500);

  height: 12px;
  width: 12px;
  background-color: var(--color);
  border-radius: 50%;
  border: 1px solid color-mix(in oklab, var(--color), var(--pico-muted-border-color) 50%);
  // z-index: 1000;

  .content {
    display: none;
    grid-template-columns: min-content 1fr 1fr;
    grid-template-rows: min-content;
    align-items: center;
    gap: 0 4px;
    padding: 8px;
    background-color: var(--pico-background-color);
    position: absolute;
    top: 12px;
    left: 12px;
    border-radius: 8px;
    min-width: 120px;

    .country-name {
      grid-column: span 2;
    }

    .revenue {
      grid-column: span 2;
    }
  }

  &:hover {
    z-index: 1;

    .content {
      display: grid;
    }
  }
}
</style>
