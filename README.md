<img width="5679" height="2227" alt="Foundry Banner" src="https://github.com/user-attachments/assets/428e4a52-ee6b-44eb-9215-e08f3b5d802f" />

A lightweight graphics engine using Rust and VulkanAPI. In the future I plan to expand this into a full game engine.

The main goal of this project was to teach myself about computer graphics, as well as give myself a way to work on game ideas without relying on Unity/Unreal.

Current supported OS:
- Windows 10/11
- MacOS (only intel tested)
- Linux support coming soon

Recent Milestone: Multiple objects in a scene as well as UI element for displaying fps (March 1)
<img width="1000" height="1031" alt="fps counter" src="https://github.com/user-attachments/assets/1bcda4dc-9d07-4b2c-8fce-d876c8505c5f" />

Past Milestone: Loaded a 3D objec from a .obj file (Feb 21)
<img width="747" height="1028" alt="Object Example" src="https://github.com/user-attachments/assets/e00c756a-17e6-420e-9180-1c8acbac72e2" />

Past Milestone: Added a texture image to the rotating plane (Feb 17)
<img width="1003" height="784" alt="Texture Image Example" src="https://github.com/user-attachments/assets/84a99227-8666-4e55-b28d-171da7501a54" />

Past Milestone: Made a rotating 3d square thats off-center of the origin with a transformation matrix (Feb 4, 2025)
<img width="790" height="622" alt="Screenshot 2026-02-10 at 7 17 31 PM" src="https://github.com/user-attachments/assets/874bf82d-f00c-49e6-8159-48ba24f8f0c0" />

Past milestone: Drawing a shaded triangle on the screen as seen below (Jan 15, 2026).
<img width="995" height="778" alt="foundry" src="https://github.com/user-attachments/assets/f8d3799e-a5ca-4a49-83db-b6776b08c7fc" />

<u>Game Engine Features:</u>
You can:
* Instantiate game objects
* Set the position of game objects

<u>Tools</u>
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
- Documentation is pretty outdated but I plan to update it ASAP - 
<img width="957" height="743" alt="Screenshot 2026-01-21 at 10 33 15 PM" src="https://github.com/user-attachments/assets/6bd7971d-d0c5-4c4a-91a5-f2ad3629e3c5" />
Example of how all interactions are linked
Last updated: Jan 21, 2026
