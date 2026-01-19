A queue is where you submit all your [[Command Buffer]]s to. The queue will then execute each command buffer in the order it was submitted.

Each queue can only perform certain operations

Kinds of Queues nd what they do: 
- Graphics Queue - This queue can run a [[Graphics Pipeline]]  by the Vulkan draw command
- Compute Queue - This queue can compute pipelines (I have not used this yet)
- Presentation Queue - This queue can present images from the [[Swapchain]] to the [[Surface]]