"Vulkan separates the concept of _physical_ and _logical_ devices. A physical device usually represents a single complete implementation of Vulkan (excluding instance-level functionality) available to the host, of which there are a finite number."
A Physical Device is a Vulkan representation of a piece of hardware such as a GPU or IGPU that is capable of performing various Vulkan operations.

Do not get this confused with a Vulkan [[Logical Device]]

Connections:
- Contains [[Queue Families]] used for Vulkan operations such as graphics, data transfers, and presentation
- 
