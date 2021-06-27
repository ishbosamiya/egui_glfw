#version 330 core

uniform mat4 projection;
uniform vec2 screen_size;       // (width, height)

in vec2 in_pos;
// in vec2 in_uv;
in vec4 in_color;

out vec4 finalColor;

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
  vec2 pos = vec2(in_pos.x / screen_size.x, 1.0 - in_pos.y / screen_size.y) * screen_size;
  gl_Position = projection * vec4(pos, -10.0, 1.0);
  finalColor = linear_from_srgba(in_color);
}
