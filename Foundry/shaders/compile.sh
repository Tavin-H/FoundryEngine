
~/VulkanSDK/1.4.335.1/macOS/bin/glslc \
    --target-env=vulkan1.0 \
    -fshader-stage=vert \
    shader.vert \
    -o vert.spv
#~/VulkanSDK/1.4.335.1/macOS/bin/glslc shader.frag -o frag.spv
~/VulkanSDK/1.4.335.1/macOS/bin/glslc \
    --target-env=vulkan1.0 \
    -fshader-stage=frag \
    shader.frag \
    -o frag.spv
