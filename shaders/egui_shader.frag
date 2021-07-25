#version 330 core

uniform sampler2D egui_texture;

in vec4 finalColor;
in vec2 v_uv;
out vec4 fragColor;

void main()
{
  fragColor = finalColor * texture(egui_texture, v_uv);
}
