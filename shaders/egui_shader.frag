#version 330 core

uniform sampler2D u_texture;

in vec2 f_uv;
in vec4 f_colour;

out vec4 o_frag_colour;

void main()
{
  o_frag_colour = f_colour * texture(u_texture, f_uv);
}
