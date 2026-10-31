import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TopBar from "./TopBar";
import Dock from "./Dock";
import SettingsWindow from "./SettingsWindow";
import Hud from "./Hud";
import "./design/tokens.css";
import "./styles.css";

const label = getCurrentWindow().label;
const View = label === "dock" ? Dock : label === "settings" ? SettingsWindow : label.startsWith("hud") ? Hud : TopBar;
document.body.dataset.view = label;
createRoot(document.getElementById("root")!).render(<View />);
