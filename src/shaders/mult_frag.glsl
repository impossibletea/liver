#version 330

in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

void main()
{
	vec4 texture = texture(tex, frag_uv);
	vec4 color = vec4(1.0, 1.0, 1.0, 1.0);
	gl_FragColor = color;
}

