
A collection of [[Image]]s in the GPU memory that are used to render to and present to the screen.

Different types:
- Triple buffer swapchain (what I use) - contains 3 images
- Double buffer swapchain - contains 2 images

Different Modes:
- Immediate - Images that are finished being rendered are presented to the screen right away (Warning: this may result in tearing).
- FIFO - presents the image at the front of the queue and adds rendered images to the back of the queue. (How V-sync works) However, if the queue is full, then the program has to wait for an available image to render to.
- FIFO relaxed - Same as FIFO, however, if the queue is empty during the vertical blank, then the next image is presented as soon as it's done rendering.
- Mailbox (what I use) - Similar to FIFO, however, if the queue is full, then the images in the queue are replaced with the newer ones. (Warning: this has a high energy usage)

