#version 330

in vec2 position;

void main() {
    vec2 pos = position.xy / 10.0;
    gl_Position = vec4(pos, 0.0, 1.0);
}

