Because [egui](https://github.com/emilk/egui) is an immediate-mode GUI library, its architectural limits are deeply structural. While it is unmatched for building developer tools, game overlays, and quick dash-style desktop applications, it faces hard roadblocks when pushed to build high-end commercial products. [1, 2]
The main limits of egui stem from its core paradigm:
## 1. The Layout "Blind Spot" (First-Frame Jitter)
Because immediate-mode re-renders everything from scratch every frame, egui doesn't know the size of a widget until it actually draws it. [1, 3]

*
* The Limit: Creating complex, responsive layouts (like a dynamic wrap-around flexbox or auto-sizing tables) is highly friction-filled.
* The Symptom: Features like egui::Grid rely on tracking sizing data from previous frames. The first time a grid renders, it has to guess the size. If it guesses wrong, it creates a one-frame "pop" or visual jitter before correcting itself (which it attempts to hide using a multi-pass discard request). [3, 4, 5]
*

## 2. High CPU Power Consumption
In traditional GUI toolkits (Retained Mode like Qt, Electron, or Tauri), the interface sits completely idle and consumes 0% CPU until you move the mouse or input data.

*
* The Limit: By default, if something is animating or continuously repainting, egui runs your entire UI layout logic block at 60+ frames per second. [1]
* The Symptom: While egui does feature a "reactive mode" (only repainting when it receives input), interacting with it heavily or running background tasks can draw significant battery and CPU resources compared to light retained-mode alternatives. [6]
*

## 3. Severe Web Platform Realities (WASM Canvas)
When compiling egui for the web via WebAssembly, it doesn't use HTML elements or the standard DOM. Instead, it paints raw pixels onto a single WebGL/WebGPU <canvas>. This introduces massive web-specific constraints: [7, 8]

*
* No Native Browser Integration: Users cannot press Ctrl+F to search text on the page, nor can they natively highlight text unless a custom widget implements it. [9]
* Broken Text Interactions: Browser extensions (like password managers or translation tools) cannot read the UI. Right-clicking doesn't reveal standard browser menus, and opening links in background tabs via mouse clicks behaves unnaturally. [8]
* Mobile Friction: Faking an on-screen keyboard, supporting mobile pinch-to-zoom, and smooth touch-scrolling require explicit library-level workarounds rather than inheriting native browser mechanics. [8, 9]
*

## 4. Poor Typography & Complex Text Rendering
egui handles its own font rasterization and shaping, rather than offloading it to the host operating system. [8]

*
* The Limit: It has poor out-of-the-box support for complex Unicode scripts. Cursive layout joins, right-to-left flowing scripts (like Arabic), or context-dependent grapheme clusters (like Devanagari/Hindi) frequently render incorrectly or fail to shape properly.
* Blurry Text: Because it struggles to accurately align text to the exact physical pixel grid on varying monitor display scales, text can look fuzzy or lack subpixel antialiasing at 1x desktop scaling. [6, 7, 10]
*

## 5. Multi-Window Isolation
If your app architecture requires multi-tasking across several independent desktop windows, egui makes this notoriously difficult. [11]

*
* The Limit: The default native integration framework, eframe, natively hosts only one window per application.
* The Symptom: Context menus, dropdown tooltips, and dialog popups are strictly clipped to the boundaries of that single outer window. They cannot overflow outside the application frame like native OS menus do. [11]
*

## Summary: When should you avoid it?
Avoid egui if you are building an SEO-focused public website, an application prioritizing long-lasting mobile battery life, a highly custom text document editor, or a consumer-facing app requiring a perfectly native look and feel on Windows or macOS. Use it freely if you value instant developer productivity, rust-native speed, and internal application tools. [2, 4]

[1] [https://github.com](https://github.com/emilk/egui)
[2] [https://www.youtube.com](https://www.youtube.com/watch?v=YNvfqmEgUFQ)
[3] [https://news.ycombinator.com](https://news.ycombinator.com/item?id=39009542)
[4] [https://medium.com](https://medium.com/@build_break_learn/rust-gui-framework-benchmark-egui-iced-slint-gtk-electron-d88596c042fb)
[5] [https://docs.rs](https://docs.rs/egui/latest/egui/)
[6] [https://www.reddit.com](https://www.reddit.com/r/rust/comments/12slfw2/considerations_for_power_draw_with_egui/)
[7] [https://github.com](https://github.com/emilk/egui/issues/516)
[8] [https://news.ycombinator.com](https://news.ycombinator.com/item?id=33861831)
[9] [https://crates.io](https://crates.io/crates/egui_web)
[10] [https://github.com](https://github.com/emilk/egui/issues/2517)
[11] [https://www.reddit.com](https://www.reddit.com/r/rust/comments/wippxd/before_learning_egui_is_there_anything_i_should/)
