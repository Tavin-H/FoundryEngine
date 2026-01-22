Used for controlling the order of operations on a GPU.
This is helpful since the GPU will perform operations in parallel, leading to chaotic and unpredictable behaviour when one function relies on another one to finish before it

usage:
```
VkCommandBuffer A, B = ... // record command buffers

VkSemaphore S = ... // create a semaphore

// enqueue A, signal S when done - starts executing immediately

vkQueueSubmit(work: A, signal: S, wait: None)

// enqueue B, wait on S to start

vkQueueSubmit(work: B, signal: None, wait: S)
```
(taken from the Vulkan Tutorial book)

Connections: 
- Similar to a [[Fence]] but for the GPU
