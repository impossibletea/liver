#version 330

in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

vec4 blend_normal(vec4 d, vec4 s);
vec4 blend_add(vec4 d, vec4 s);
vec4 blend_multi(vec4 d, vec4 s);

void main() {
	vec4 texture = texture(tex, frag_uv);
	vec4 color = vec4(
		texture.xyz,
		texture.w * opacity
	);
	gl_FragColor = color;
}

vec4 blend_normal(vec4 d, vec4 s) {
	vec3 rgb = d.xyz * (1 - s.w) + s.xyz;
	float a = d.w * (1 - s.w) + s.w;
	return vec4(rgb, a);
}

vec4 blend_add(vec4 d, vec4 s) {
	vec3 rgb = d.xyz + s.xyz;
	float a = d.w;
	return vec4(rgb, a);
}

vec4 blend_multi(vec4 d, vec4 s) {
	vec3 rgb = d.xyz * (1 - s.w) + s.xyz * d.xyz;
	float a = d.w;
	return vec4(rgb, a);
}
