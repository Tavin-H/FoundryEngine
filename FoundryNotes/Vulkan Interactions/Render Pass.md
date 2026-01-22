"A VkRenderPass is a Vulkan object that encapsulates the state needed to set up the 'target' for rendering, and the state of the images you will be rendering to."

In the Render Pass, there are descriptions of all the steps needed to draw a frame.

Note: Render commands must be called in the render pass
```
vkCmdBeginRenderPass(cmd, ...);

//rendering commands go here

vkCmdEndRenderPass(cmd);
```
Example of how the render pass is used in the [[Graphics Pipeline]].

Connections:
- Check out [[Draw Frame function explained]] for a detailed explanation of how this is used.

