import App from "./App.vue";
import { createApp } from "vue";
import { createPinia } from 'pinia';
import { router } from "./router/router.ts";

const components = import.meta.glob("./components/*.vue", { eager: true });
const app = createApp(App);
const pinia = createPinia()


Object.entries(components).forEach(([path, module]) => {
    // deno-lint-ignore no-explicit-any
    const component = (module as { default: any }).default;

    const name = component.name || `sb-${path
        .split("/")
        .pop()
        ?.replace(/\.\w+$/, "")
        .toLowerCase() }`;
    if (typeof module === "object" && module !== null && "default" in module) {
        app.component(name, component);
    }
});

app.use(pinia);
app.use(router);
app.mount("#app");
