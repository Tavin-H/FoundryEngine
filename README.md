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

Recent milestone: Made a rotating 3d square thats off-center of the origin with a transformation matrix
<img width="790" height="622" alt="Screenshot 2026-02-10 at 7 17 31 PM" src="https://github.com/user-attachments/assets/874bf82d-f00c-49e6-8159-48ba24f8f0c0" />

Past milestone: Drawing a shaded triangle on the screen as seen below (Jan 15, 2026).
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

For those of you wanting to create a similar project, or just want to know how vulkan works, I've created some documentation of this project with obsidian. So far I've explained all the interactions between the Vulkan Objects and the significance of each one, but I will keep these notes updated as I progress.
<img width="957" height="743" alt="Screenshot 2026-01-21 at 10 33 15 PM" src="https://github.com/user-attachments/assets/6bd7971d-d0c5-4c4a-91a5-f2ad3629e3c5" />
Example of how all interactions are linked
