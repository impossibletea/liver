#version 330

in vec2 position;
in vec2 texture_uv;

uniform vec2 size;
uniform vec2 origin;
uniform float scale;
uniform vec2 aspect;

void main()
{
	vec2 pos = position;
	pos *= scale;  // canvas scale
	pos += origin; // move to canvas origin
	pos /= size;   // [0; s] -> [ 0; 1]
	pos *= 2;      //        -> [ 0; 2]
	pos -= 1;      //        -> [-1; 1]
	pos *= aspect; // preserve aspect ratio
	gl_Position = vec4(pos, 0.0, 1.0);
}

