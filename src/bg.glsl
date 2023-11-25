#version 330

in vec2 position;
in vec2 texture_uv;

uniform vec2 aspect;

out vec2 frag_uv;

void main()
{
	frag_uv = texture_uv;
	vec2 pos = position;
	pos *= aspect;
	gl_Position = vec4(pos, 0.0, 1.0);
}
