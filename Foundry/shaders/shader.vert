#version 450
//Input from buffers

layout(set = 0, binding = 0) uniform UniformBufferObject {
	mat4 model;
	mat4 view;
	mat4 proj;
} ubo;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_colour;
layout(location = 2) in vec2 in_tex_coord;

//Output to buffer
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec2 frag_tex_coord;

void main() {
	//gl_Position = vec4(in_position, 1.0, 0.0);
	gl_Position = ubo.proj * ubo.view * ubo.model * vec4(in_position, 1.0);
	frag_color = in_colour;
	frag_tex_coord = in_tex_coord;
}

