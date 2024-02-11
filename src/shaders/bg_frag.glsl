#version 330

in vec2 frag_uv;

uniform sampler2D tex;

void main() {gl_FragColor = texture(tex, frag_uv);}

