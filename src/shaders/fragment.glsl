#version 140

in vec4 v_color;
in float v_style;
in vec2 v_tex_coord;

out vec4 f_color;

void main() {
  if (v_style == 1) {
    vec2 uv = v_tex_coord;
    
    float t = distance(uv, vec2(0.5));
    
    if (t <= 0.5) {
      f_color = v_color;
    }
    else {
      f_color = vec4(1.0,1.0,1.0,0.0);
    }
}
  else {
    f_color = v_color;
  }
}
