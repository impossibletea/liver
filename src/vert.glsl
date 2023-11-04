#version 330

in vec2 position;
in vec2 texture_uv;
out vec2 frag_uv;

uniform vec2 size;
uniform vec2 origin;
uniform float scale;
uniform vec2 aspect;

void main() {
	frag_uv = texture_uv;
	vec2 pos = 1 + 2 * (scale * position - origin) / size;
	//pos -= origin;
	pos *= aspect;
	gl_Position = vec4(pos, 0.0, 1.0);
}

