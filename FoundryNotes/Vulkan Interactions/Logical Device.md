A Logical Device is an instance of Vulkan running on a [[Physical Device]] with its own resources. Note: each Physical Device can be exposed to multiple logical devices within the same application.

Connections:
- Can retrieve handles to a [[Queue]] using a queue family index.
- Used as a parameter to create a [[Swapchain Instance]].
- Used to create a [[Render Pass]].