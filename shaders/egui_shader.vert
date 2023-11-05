#version 330 core

uniform vec2 u_screen_size_in_points; // (width, height)

in vec2 v_pos;
in vec2 v_uv;
in vec4 v_colour;

out vec2 f_uv;
out vec4 f_colour;

// 0-1 linear  from  0-255 sRGB
// from egui_glium
vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, cutoff);
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main()
{
  vec2 pos = vec2(2.0 * v_pos.x / u_screen_size_in_points.x - 1.0,
                  1.0 - 2.0 * v_pos.y / u_screen_size_in_points.y);
  gl_Position = vec4(pos, 0.01, 1.0);
  f_uv = v_uv;
  f_colour = linear_from_srgba(v_colour);
}
