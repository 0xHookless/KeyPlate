import "@fontsource-variable/public-sans";
import "@fontsource-variable/martian-mono";
import "./lib/tokens.css";
import "./lib/base.css";
import { mount } from "svelte";
import { applyTheme, getTheme } from "./lib/theme";
import App from "./App.svelte";

applyTheme(getTheme());
mount(App, { target: document.getElementById("app")! });
