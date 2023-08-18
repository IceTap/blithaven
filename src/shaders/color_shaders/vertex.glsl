#version 140

in vec2 position;
in vec4 color;
in int style;
in vec2 tex_coord;
in float variator;

out vec4 v_color;
out float v_style;
out vec2 v_tex_coord;
out float v_variator;

uniform mat4 matrix;

void main() {
  v_color = color;
  v_style = style;
  v_tex_coord = tex_coord;
  v_variator = variator;
  gl_Position = matrix * vec4(position, 0.0, 1.0);
}
