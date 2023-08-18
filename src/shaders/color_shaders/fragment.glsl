#version 140

in vec4 v_color;
in float v_style;
in vec2 v_tex_coord;
in float v_variator;

out vec4 f_color;

void main() {
  if (v_style == 1) {
    vec2 uv = v_tex_coord;
    
    float t = distance(uv, vec2(0.5));
    
    if (t <= 0.5) {
      f_color = v_color;
    }
    else {
      f_color = vec4(uv,1.0,0.0);
    }
  }
  else if (v_style == 2) {
    vec2 uv = v_tex_coord;
    
    if (uv.x + uv.y > 1 - v_variator) {
      if (uv.x + uv.y < 1 + v_variator) {
        f_color = v_color;
      }
      else {
        f_color = vec4(uv,1.0,0.0);
      }
    }
    
    else {
      f_color = vec4(uv,1.0,0.0);
    }
  }
  else {
    f_color = v_color;
  }
}
