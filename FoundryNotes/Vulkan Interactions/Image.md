A VkImage is the object that stores the individual pixels of a target that is rendered or to be rendered by the [[Swapchain]].

It's stored in the GPU memory
Cannot be accessed directly; must be used through an [[Image View]]

Connections:
- An [[Image View]] provides additional information about the image.
- 