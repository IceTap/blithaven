#version 140

in vec2 position;
in vec2 tex_coord;
out vec2 v_tex_coord;

uniform mat4 matrix;

void main() {
    v_tex_coord = tex_coord;
    gl_Position = matrix * vec4(position, 0.0, 1.0);
}