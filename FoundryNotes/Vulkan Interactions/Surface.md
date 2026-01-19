Desc:
"Since Vulkan is a platform-agnostic API, it is not designed to interface directly with the window system on its own. To establish the connection between Vulkan and the window system to present results to the screen, we need to use the WSI (Window System Integration) extensions"

A surface is linked to a window created by a window library for presentation of what is rendered.

Different kinds of surfaces are created depending on which operating system you are developing for.
OS:
- Mac - VkMetalSurface
- Windows - VkSurface32
- Linux - Not sure lol

Links:
- A surface Influences which [[Physical Device]] you pick