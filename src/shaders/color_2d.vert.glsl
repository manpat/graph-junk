#version 450

layout(std140, row_major, binding = 0) uniform UniformData {
	mat4 u_projection_view;
};


layout(location=0) in vec2 a_pos;
layout(location=1) in vec4 a_color;

out vec4 v_color;

void main() {
	gl_PointSize = 3.0;
	gl_Position = u_projection_view * vec4(a_pos, 0.0, 1.0);
	v_color = a_color;
}