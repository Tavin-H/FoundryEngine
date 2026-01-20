"A VkRenderPass is a Vulkan object that encapsulates the state needed to set up the 'target' for rendering, and the state of the images you will be rendering to."

Note: Render commands must be called in the render pass
```
vkCmdBeginRenderPass(cmd, ...);

//rendering commands go here

vkCmdEndRenderPass(cmd);
```
Example of how the render pass is used in the [[Graphics Pipeline]].

Connections:
- 

