#version 450
/*
layout(location = 0) out vec3 frag_color;

const vec2 positions[3] = vec2[] (
		vec2(0.0, -0.5),
		vec2(0.5, 0.5),
		vec2(-0.5, 0.5)
		);

const vec3 colors[3] = vec3[] (
		vec3(1.0, 0.0, 0.0),
		vec3(0.0, 1.0, 0.0),
		vec3(0.0, 0.0, 1.0)
		);

void main() {
	gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
	frag_color = colors[gl_VertexIndex];
}
*/
//Input from buffers
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec3 in_colour;

//Output to buffer
layout(location = 0) out vec3 frag_color;

void main() {
	gl_Position = vec4(in_position, 1.0, 0.0);
	frag_color = in_colour;
}

