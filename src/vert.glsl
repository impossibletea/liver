#version 330

in vec2 position;
in vec2 texture_uv;
out vec2 frag_uv;

uniform vec2 size;
uniform vec2 origin;
uniform float scale;

void main() {
	frag_uv = texture_uv;
	vec2 pos = 2 * scale * position / size;
	gl_Position = vec4(pos, 0.0, 1.0);
}

