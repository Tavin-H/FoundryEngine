A VkImage is the object that stores the individual pixels of a target that to presented to the screen by the [[Swapchain]].

It's stored in the GPU memory
Cannot be accessed directly; must be used through an [[Image View]]

an 'Image Handle' points to this area in memory and describes the width, height and format.

Connections:
- An [[Image View]] provides additional information about the image.
- 