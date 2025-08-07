import { createWebHistory, createRouter } from 'vue-router'

import Welcome from './Welcome.vue'

import Home from './Home.vue'
import World from './World.vue'
import Export from './Export.vue'
import Decrypting from './Decrypting.vue'
import Overview from './Overview.vue'

const routes = [
  {
    name: 'home', path: '/', component: Home,
    children: [
      { name: 'overview', path: '', component: Overview },
      { name: 'world', path: 'world', component: World },
      { name: 'export', path: 'export', component: Export }
    ]
  },
  { name: 'welcome', path: '/welcome', component: Welcome },
  { name: 'decrypting', path: '/decrypting', component: Decrypting },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})