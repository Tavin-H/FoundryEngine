           ______                    _               ______             _
          |  ____|                  | |             |  ____|           (_)
          | |__ ___  _   _ _ __   __| |_ __ _   _   | |__   _ __   __ _ _ _ __   ___
          |  __/ _ \| | | | '_ \ / _` | '__| | | |  |  __| | '_ \ / _` | | '_ \ / _ \
          | | | (_) | |_| | | | | (_| | |  | |_| |  | |____| | | | (_| | | | | |  __/
          |_|  \___/ \__,_|_| |_|\__,_|_|   \__, |  |______|_| |_|\__, |_|_| |_|\___|
                                             __/ |                 __/ |
                                            |___/                 |___/
A lightweight graphics engine using Rust and VulkanAPI. In the future I plan to expand this into a full game engine.

The main goal of this project was to teach myself about computer graphics, as well as give myself a way to work on game ideas without relying on Unity/Unreal.

Current supported OS:
- Windows 10/11
- MacOS (only intel tested)
- Linux support coming soon

The biggest recent change was drawing a shaded triangle on the screen as seen below (Jan 15, 2026).
<img width="995" height="778" alt="foundry" src="https://github.com/user-attachments/assets/f8d3799e-a5ca-4a49-83db-b6776b08c7fc" />

I've chosen each of my tools for specific reasons as stated below.
* winit (window manager)
  - Cross platform
  - Easy window events which will allow me to expand this project into a game engine
* Rust (programming language)
  - As fast as the competitors (like C & C++)
  - Superior memory management
  - Fast to develop in compared to competitors
* Vulkan
  - Newer, more modern API compared to others like OpenGL
  - Extremely customizable
  - Lets me get my hands dirty and learn as much as possible
