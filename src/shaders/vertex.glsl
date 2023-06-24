#version 140

in vec2 position;
in vec3 color;
in int style;
in vec2 tex_coord;

out vec4 v_color;
out float v_style;
out vec2 v_tex_coord;

uniform mat4 matrix;

void main() {
  v_color = vec4(color, 1.0);
  v_style = style;
  v_tex_coord = tex_coord;
  gl_Position = matrix * vec4(position, 0.0, 1.0);
}
