Desc:
"The instance is the connection between your application and the Vulkan library"
This VkInstance Object is the backbone of your Vulkan application and will be used in pretty much every function you use.

Since there is NO global state in Vulkan, an Instance is responsible for managing all the Vulkan-related information and will pass this information around as needed.

Connections:
- Used to create a [[Surface]] for screen presentation
- Used to enumerate all available potential [[Physical Device]] options
- Used to query for [[Queue Families]] in a given Physical Device
- Used to create a [[Logical Device]]
- Used to create a [[Surface Instance]]
- Used to create a [[Surface Instance]]
- Used as a parameter to create a [[Swapchain Instance]].
