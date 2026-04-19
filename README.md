
<img width="1481" height="500" alt="Foundry Banner" src="https://github.com/user-attachments/assets/64d3b090-c955-423a-b90a-34f8d6db500d" />

# FoundryEngine
<p align="center">
  <img src="https://img.shields.io/badge/Rust-red?style=for-the-badge&logo=rust" />
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="https://img.shields.io/badge/Vulkan-DC143C?style=for-the-badge&logo=Vulkan" />
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="https://img.shields.io/badge/Version-pre--alpha-white?style=for-the-badge&logo=Git&logoColor=white&labelColor=2d3146&color=%234cb083" />
</br>
  <img src="https://img.shields.io/badge/Downloads-89-white?style=for-the-badge&labelColor=2d3146&color=blue" />
   &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="https://img.shields.io/badge/License-MIT-white?style=for-the-badge&labelColor=2d3146&color=yellow" />
  &nbsp;&nbsp;&nbsp;&nbsp;
    <a href="https://foundry-engine.gitbook.io/foundry-engine-docs/">
      <img src="https://img.shields.io/badge/Documentation-WIP-white?style=for-the-badge&logo=Gitbook&labelColor=2d3146&color=9370db" />
    </a>
</p>


## 📝Description
A lightweight, open-source game engine written in Rust and using Vulkan API. 

The main goal of this project was to teach myself computer graphics and give myself a way to work on game ideas without relying on Unity/Unreal.

---
## ⚡ Game Engine Features:

### General
* Custom 3D graphics engine
* Input handling with Winit
* UI system

### ECS (Entity Component System)
* Data-oriented design to allow for super-fast and efficient system lookups
* Add components to game objects (represented as 'Game Entities')
* Get components from game objects with turbofish syntax

### Scene Manager
* Instantiate game objects
* Set the position of game objects

---
## 🖥️ Current supported Operating Systems:
- Windows 10/11
- MacOS (only intel tested)
> Linux support coming soon

---
## 🚀 Usage

Clone the repo
```
git clone https://github.com/Tavin-H/FoundryEngine.git
```
Make sure to cd into foundry
```
cd your-clone-location/FoundryEngine/Foundry
```

Run the demo scene
```
cargo run --release
```
> Note: Foundry is still in pre-beta and isn't ready to make full games with. Check out the roadmap to see when new features are coming.

---
## 🛠️ Tools
I've chosen each of my tools for specific reasons as stated below.

| Tool | Choice | Why |
| :--- | :--- | :--- |
| **Programming Language** | Rust |• As fast as the competitors (like C & C++) <br> • Superior memory management <br> • Fast to develop in comparison to competitors |
| **Graphics API** | Vulkan | • Newer, more modern API compared to others like OpenGLNewer, more modern API compared to others like OpenGL <br> • Platform agnostic <br> • Extremely customizable <br> • Lets me get my hands dirty and learn as much as possible|
| **Window Manager** | Winit |• Cross platform <br> • Written in Rust <br> • Easy window events to use for input handling <br> • Easy integration with ImGUI|
---
## 🗺️ Roadmap
### Features currently in development
* Custom components

### Features to come
* Drag and drop mesh files to use for game objects
* Scene graph in scene manager
* Adjusting game object positions from scene graph
* Attaching scripts to game objects
* Lighting engine
* UI manager to use in scripts
* Sound manager to use in scripts

---
## 👷 Contributing
First off, thank you for considering contributing! I'm happy to have others involved in the project.

#### How to get started
1. Check the Issues: See if what you want to work on is already being discussed.

2. Start a Conversation: Open a new issue or comment on an existing one to pitch your idea.

3. Fork & Branch: Once we've touched base, feel free to fork the repo and create a descriptive branch.

4. Submit a PR: Reference your issue in the Pull Request description.

> Small fixes (like typos or minor bugs) are always welcome as direct Pull Requests!

---
## 🆘 Support
### Options
1. Open a Github issue and I will work to fix it as soon as possible.
2. Use Discussions to get my attention

---
## 📜 History
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

For those of you wanting to create a similar project, or just want to know how vulkan works, I've created some documentation of this project with obsidian. So far I've explained all the interactions between the Vulkan Objects and the significance of each one, but I will keep these notes updated as I progress.
- Documentation is pretty outdated but I plan to update it ASAP - 
<img width="957" height="743" alt="Screenshot 2026-01-21 at 10 33 15 PM" src="https://github.com/user-attachments/assets/6bd7971d-d0c5-4c4a-91a5-f2ad3629e3c5" />
Example of how all interactions are linked
Last updated: Jan 21, 2026
