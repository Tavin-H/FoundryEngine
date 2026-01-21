"An image view is quite literally a view into an image. It describes how to access the image and which part of the image to access, for example if it should be treated as a 2D texture depth texture without any mipmapping levels." 

A required wrapper around an image that gives information about how to interpret the memory data (2D color attachment)
This provides data on how to use the _raw_ data in the [[Image]].

Each [[Image]] can have multiple of these Image views to be used all over the place (In multiple [[Frame Buffer]]s)

Connections:
- An image view that's being used in the [[Render Pass]] is called an [[Attachment]]
