import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import {
  exists,
  BaseDirectory,
  writeFile,
  readTextFile,
  readDir,
  createDir,
} from "@tauri-apps/api/fs";
import { appConfigDir } from "@tauri-apps/api/path";
import { enable } from "tauri-plugin-autostart-api";

await enable();

interface Event {
  payload: any;
}

let urlInputEl: HTMLInputElement | null;
let tokenInputEl: HTMLInputElement | null;
let actionMessageEl: HTMLElement | null;
let logsEl: HTMLElement | null;

let data;

try {
  await readDir(await appConfigDir());
} catch (e) {
  await createDir(await appConfigDir());
}

if (await exists("settings.json", { dir: BaseDirectory.AppConfig })) {
  console.log("Settings file exists");
  data = JSON.parse(
    await readTextFile("settings.json", { dir: BaseDirectory.AppConfig })
  );
} else {
  console.log("Settings file does not exist");
  await writeFile("settings.json", "{}", { dir: BaseDirectory.AppConfig });
  data = {};
}

let url = data.url ?? "";
let token = data.token ?? "";

urlInputEl = document.querySelector("#url-input");
tokenInputEl = document.querySelector("#token-input");
if (urlInputEl && tokenInputEl) {
  urlInputEl.value = url;
  tokenInputEl.value = token;
}
actionMessageEl = document.querySelector("#action-msg");
logsEl = document.querySelector("#logs");

await listen("online-check", (event: Event) => {
  console.log("Received event: " + JSON.stringify(event));
  if (logsEl) {
    logsEl.innerHTML = event.payload.data + logsEl.innerHTML;
  }
});

async function storeUserData() {
  console.log("Storing user data...");
  console.log("URL: " + urlInputEl);
  console.log("Token: " + tokenInputEl);
  if (urlInputEl && tokenInputEl && actionMessageEl) {
    await writeFile(
      "settings.json",
      JSON.stringify({ url: urlInputEl.value, token: tokenInputEl.value }),
      { dir: BaseDirectory.AppConfig }
    );

    url = urlInputEl.value;
    token = tokenInputEl.value;

    actionMessageEl.innerText = "Settings saved!";
    console.log("Settings saved!");
  }
}

document.querySelector("#submit-button")?.addEventListener("click", () => {
  storeUserData();
});

// every 5 seconds, invoke the function
setInterval(() => {
  console.log("Invoking online_check...");
  invoke("online_check", { url: url, token: token });
}, 60000);
