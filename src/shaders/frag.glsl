#version 330

in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;
uniform vec4 screen;
uniform vec4 mult;

void main()
{
	vec4 color = texture(tex, frag_uv);
	color.rgb *= mult.rgb;
	// This is taken from the Live2D Framework, I can not figure out where the
	// fuck does this come from
	color.rgb = color.rgb + screen.rgb - (color.rgb * screen.rgb);
	color.a *= opacity;
	gl_FragColor = color;
}

