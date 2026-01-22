Used to halt the main thread of the cpu.
For Vulkan, this is very helpful for synchronizing the CPU and GPU

usage:
```
VkCommandBuffer A = ... // record command buffer with the transfer

VkFence F = ... // create the fence

// enqueue A, start work immediately, signal F when done

vkQueueSubmit(work: A, fence: F)

vkWaitForFence(F) // blocks execution until A has finished executing

save_screenshot_to_disk() // can't run until the transfer has finished
```
(taken from the Vulkan Tutorial book)

Connections:
- Similar to a [[Semaphore]] but for the CPU.