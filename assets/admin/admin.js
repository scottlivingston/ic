const PRESET_PHRASES = [
  "I AM THE I C.",
  "Hello, Lunar traveler.",
  "Welcome to the Mall Station!",
  "Follow me to the check-in terminal and get ready for an adventure in The Mall!",
  "Hello again, why are you still here?",
  "You need to sign in using this terminal!",
  "Umm.. there seems to be a problem.",
  "Not to worry mister, I know another way, follow me.",
  "Oh my, What a day!",
  "I love helping people who have no clue what they are doing.",
  "Everyone comes here, Union Plaza is the best, Ha ha..",
  "Have you seen Jackie recently? I miss her.",
  "Did you know that four legs are better than two. he he he.",
  "I am happy to stay here.. yes.",
  "Good by fellow Moon traveler, Good byy!",
];

// Effect state - defaults to 50% to match ic-bevy
const effectState = {
  glow: true,
  glow_intensity: 0.5,
  scanlines: true,
  scanline_opacity: 0.5,
  flicker: true,
  flicker_amount: 0.3,
  curvature: true,
  curvature_amount: 50,
  grid: false,
};

// Audio state
let audioVolume = 0.2;

// WiFi state
let wifiNetworks = [];
let wifiScanning = false;
let wifiConnecting = false;
let selectedNetwork = null;

// HUD state
let hudState = {
  show_ip: false,
  ip_address: null,
};

async function sendSpeak(msg) {
  try {
    const response = await fetch("/api/speak", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ msg }),
    });
    if (!response.ok) {
      console.error("Failed to send speak:", response.statusText);
    }
  } catch (err) {
    console.error("Error sending speak:", err);
  }
}

async function sendEffects() {
  try {
    const response = await fetch("/api/effects", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(effectState),
    });
    if (!response.ok) {
      console.error("Failed to send effects:", response.statusText);
    }
  } catch (err) {
    console.error("Error sending effects:", err);
  }
}

async function sendVolume(volume) {
  // Scale volume: 100% on slider = 20% actual (new speaker is much louder)
  const scaledVolume = volume * 0.2;
  try {
    const response = await fetch("/api/volume", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ volume: scaledVolume }),
    });
    if (!response.ok) {
      console.error("Failed to send volume:", response.statusText);
    }
  } catch (err) {
    console.error("Error sending volume:", err);
  }
}

async function scanWifi() {
  wifiScanning = true;
  render();
  try {
    const response = await fetch("/api/wifi/scan");
    if (response.ok) {
      const data = await response.json();
      wifiNetworks = data.networks || [];
    } else {
      console.error("Failed to scan WiFi:", response.statusText);
      wifiNetworks = [];
    }
  } catch (err) {
    console.error("Error scanning WiFi:", err);
    wifiNetworks = [];
  }
  wifiScanning = false;
  render();
}

async function connectWifi(ssid, password) {
  wifiConnecting = true;
  render();
  try {
    const response = await fetch("/api/wifi/connect", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ssid, password }),
    });
    const data = await response.json();
    if (data.success) {
      alert(`Connected to ${ssid}!\nIP: ${data.ip_address || "unknown"}`);
      selectedNetwork = null;
      await fetchHudStatus();
    } else {
      alert(`Failed to connect: ${data.error || "Unknown error"}`);
    }
  } catch (err) {
    console.error("Error connecting to WiFi:", err);
    alert("Error connecting to WiFi");
  }
  wifiConnecting = false;
  render();
}

async function forgetWifi(ssid) {
  try {
    const response = await fetch("/api/wifi/forget", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ssid }),
    });
    const data = await response.json();
    if (data.success) {
      await scanWifi();
    } else {
      alert(`Failed to forget network: ${data.error || "Unknown error"}`);
    }
  } catch (err) {
    console.error("Error forgetting WiFi:", err);
    alert("Error forgetting WiFi");
  }
}

async function fetchHudStatus() {
  try {
    const response = await fetch("/api/hud/status");
    if (response.ok) {
      hudState = await response.json();
    }
  } catch (err) {
    console.error("Error fetching HUD status:", err);
  }
}

async function toggleIpHud(show) {
  try {
    const response = await fetch("/api/hud/ip", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ show }),
    });
    if (response.ok) {
      hudState.show_ip = show;
      render();
    }
  } catch (err) {
    console.error("Error toggling IP HUD:", err);
  }
}

function render() {
  const app = document.querySelector("#app");

  app.innerHTML = `
    <h1>IC Admin</h1>

    <div class="section">
      <h2 class="section-header" data-section="effects">CRT Effects <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="effects">
        <div class="effects-controls">
        <div class="effect-row">
          <button class="toggle-btn ${effectState.glow ? "active" : ""}" data-effect="glow">Glow</button>
          <div class="slider-group">
            <label class="slider-label">Intensity</label>
            <input type="range" class="slider" data-slider="glow_intensity" min="0" max="1" step="0.05" value="${effectState.glow_intensity}" />
            <span class="slider-value">${Math.round(effectState.glow_intensity * 100)}%</span>
          </div>
        </div>
        <div class="effect-row">
          <button class="toggle-btn ${effectState.scanlines ? "active" : ""}" data-effect="scanlines">Scanlines</button>
          <div class="slider-group">
            <label class="slider-label">Opacity</label>
            <input type="range" class="slider" data-slider="scanline_opacity" min="0" max="1" step="0.05" value="${effectState.scanline_opacity}" />
            <span class="slider-value">${Math.round(effectState.scanline_opacity * 100)}%</span>
          </div>
        </div>
        <div class="effect-row">
          <button class="toggle-btn ${effectState.flicker ? "active" : ""}" data-effect="flicker">Flicker</button>
          <div class="slider-group">
            <label class="slider-label">Amount</label>
            <input type="range" class="slider" data-slider="flicker_amount" min="0" max="1" step="0.01" value="${effectState.flicker_amount}" />
            <span class="slider-value">${Math.round(effectState.flicker_amount * 100)}%</span>
          </div>
        </div>
        <div class="effect-row">
          <button class="toggle-btn ${effectState.curvature ? "active" : ""}" data-effect="curvature">Curvature</button>
          <div class="slider-group">
            <label class="slider-label">Amount</label>
            <input type="range" class="slider" data-slider="curvature_amount" min="0" max="100" step="5" value="${effectState.curvature_amount}" />
            <span class="slider-value">${effectState.curvature_amount}%</span>
          </div>
        </div>
        <div class="effect-row">
          <button class="toggle-btn ${effectState.grid ? "active" : ""}" data-effect="grid">Grid</button>
        </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-header" data-section="audio">Audio <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="audio">
        <div class="audio-controls">
          <div class="effect-row">
            <div class="slider-group" style="flex: 1;">
              <label class="slider-label">Volume</label>
              <input type="range" class="slider" data-slider="volume" min="0" max="1" step="0.05" value="${audioVolume}" />
              <span class="slider-value">${Math.round(audioVolume * 100)}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-header" data-section="wifi">WiFi <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="wifi">
        <div class="wifi-controls">
          <button class="scan-btn" id="scan-wifi" ${wifiScanning ? "disabled" : ""}>
            ${wifiScanning ? "Scanning..." : "Scan Networks"}
          </button>
          <div class="wifi-networks">
            ${wifiNetworks.length === 0 ? '<div class="wifi-empty">No networks found. Click Scan to search.</div>' : ""}
            ${wifiNetworks
              .map(
                (net) => `
              <div class="wifi-network ${net.in_use ? "wifi-connected" : ""}" data-ssid="${net.ssid}">
                <div class="wifi-info">
                  <span class="wifi-ssid">${net.ssid}${net.in_use ? " (connected)" : ""}</span>
                  <span class="wifi-signal">${net.signal}% ${net.security}</span>
                </div>
                ${net.in_use ? `<button class="wifi-forget-btn" data-ssid="${net.ssid}">Forget</button>` : ""}
              </div>
            `
              )
              .join("")}
          </div>
          ${
            selectedNetwork
              ? `
            <div class="wifi-connect-form">
              <div class="wifi-selected">Connecting to: ${selectedNetwork}</div>
              <input type="password" class="wifi-password" placeholder="Password" />
              <div class="wifi-form-buttons">
                <button class="wifi-connect-btn" ${wifiConnecting ? "disabled" : ""}>
                  ${wifiConnecting ? "Connecting..." : "Connect"}
                </button>
                <button class="wifi-cancel-btn">Cancel</button>
              </div>
            </div>
          `
              : ""
          }
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-header" data-section="display">Display <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="display">
        <div class="display-controls">
          <div class="effect-row">
            <button class="toggle-btn ${hudState.show_ip ? "active" : ""}" id="ip-toggle">Show IP Address</button>
            <span class="ip-display">${hudState.ip_address || "Not connected"}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-header" data-section="presets">Preset Phrases <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="presets">
        <div class="preset-buttons">
          ${PRESET_PHRASES.map(
            (phrase) =>
              `<button class="preset-btn" data-phrase="${phrase}">${phrase}</button>`
          ).join("")}
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-header" data-section="custom">Custom Message <span class="collapse-icon">-</span></h2>
      <div class="section-content" data-section="custom">
        <div class="free-form">
          <input type="text" class="text-input" placeholder="Enter text to speak..." />
          <button class="submit-btn">Send</button>
        </div>
      </div>
    </div>
  `;

  // Toggle button handlers
  const toggleBtns = app.querySelectorAll(".toggle-btn");
  toggleBtns.forEach((btn) => {
    btn.addEventListener("click", () => {
      const effect = btn.dataset.effect;
      if (effect && typeof effectState[effect] === "boolean") {
        effectState[effect] = !effectState[effect];
        btn.classList.toggle("active");
        sendEffects();
      }
    });
  });

  // Slider handlers
  const sliders = app.querySelectorAll(".slider");
  sliders.forEach((slider) => {
    slider.addEventListener("input", () => {
      const key = slider.dataset.slider;
      if (key === "glow_intensity") {
        effectState.glow_intensity = parseFloat(slider.value);
        const valueSpan = slider.nextElementSibling;
        valueSpan.textContent = `${Math.round(parseFloat(slider.value) * 100)}%`;
        sendEffects();
      } else if (key === "scanline_opacity") {
        effectState.scanline_opacity = parseFloat(slider.value);
        const valueSpan = slider.nextElementSibling;
        valueSpan.textContent = `${Math.round(parseFloat(slider.value) * 100)}%`;
        sendEffects();
      } else if (key === "flicker_amount") {
        effectState.flicker_amount = parseFloat(slider.value);
        const valueSpan = slider.nextElementSibling;
        valueSpan.textContent = `${Math.round(parseFloat(slider.value) * 100)}%`;
        sendEffects();
      } else if (key === "curvature_amount") {
        effectState.curvature_amount = parseFloat(slider.value);
        const valueSpan = slider.nextElementSibling;
        valueSpan.textContent = `${slider.value}%`;
        sendEffects();
      } else if (key === "volume") {
        audioVolume = parseFloat(slider.value);
        const valueSpan = slider.nextElementSibling;
        valueSpan.textContent = `${Math.round(audioVolume * 100)}%`;
        sendVolume(audioVolume);
      }
    });
  });

  // Preset button handlers
  const presetBtns = app.querySelectorAll(".preset-btn");
  presetBtns.forEach((btn) => {
    btn.addEventListener("click", () => {
      const phrase = btn.dataset.phrase;
      if (phrase) {
        sendSpeak(phrase);
      }
    });
  });

  // Free-form input handlers
  const textInput = app.querySelector(".text-input");
  const submitBtn = app.querySelector(".submit-btn");

  submitBtn.addEventListener("click", () => {
    const msg = textInput.value.trim();
    if (msg) {
      sendSpeak(msg);
      textInput.value = "";
    }
  });

  textInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      const msg = textInput.value.trim();
      if (msg) {
        sendSpeak(msg);
        textInput.value = "";
      }
    }
  });

  // WiFi handlers
  const scanWifiBtn = app.querySelector("#scan-wifi");
  if (scanWifiBtn) {
    scanWifiBtn.addEventListener("click", scanWifi);
  }

  const wifiNetworkElements = app.querySelectorAll(".wifi-network");
  wifiNetworkElements.forEach((el) => {
    el.addEventListener("click", () => {
      const ssid = el.dataset.ssid;
      if (ssid && !el.classList.contains("wifi-connected")) {
        selectedNetwork = ssid;
        render();
      }
    });
  });

  const wifiForgetBtns = app.querySelectorAll(".wifi-forget-btn");
  wifiForgetBtns.forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      const ssid = btn.dataset.ssid;
      if (ssid && confirm(`Forget network "${ssid}"?`)) {
        forgetWifi(ssid);
      }
    });
  });

  const wifiConnectBtn = app.querySelector(".wifi-connect-btn");
  if (wifiConnectBtn) {
    wifiConnectBtn.addEventListener("click", () => {
      const passwordInput = app.querySelector(".wifi-password");
      const password = passwordInput ? passwordInput.value : "";
      if (selectedNetwork) {
        connectWifi(selectedNetwork, password);
      }
    });
  }

  const wifiCancelBtn = app.querySelector(".wifi-cancel-btn");
  if (wifiCancelBtn) {
    wifiCancelBtn.addEventListener("click", () => {
      selectedNetwork = null;
      render();
    });
  }

  // HUD handlers
  const ipToggleBtn = app.querySelector("#ip-toggle");
  if (ipToggleBtn) {
    ipToggleBtn.addEventListener("click", () => {
      toggleIpHud(!hudState.show_ip);
    });
  }

  // Section collapse handlers
  const sectionHeaders = app.querySelectorAll(".section-header");
  sectionHeaders.forEach((header) => {
    header.addEventListener("click", () => {
      const section = header.dataset.section;
      const content = app.querySelector(`.section-content[data-section="${section}"]`);
      const icon = header.querySelector(".collapse-icon");
      const parentSection = header.closest(".section");
      if (content && icon && parentSection) {
        const isCollapsed = content.classList.toggle("collapsed");
        parentSection.classList.toggle("collapsed", isCollapsed);
        icon.textContent = isCollapsed ? "+" : "-";
      }
    });
  });
}

// Initialize: fetch HUD status then render
fetchHudStatus().then(render);
